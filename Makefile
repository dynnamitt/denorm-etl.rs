
VER := 3.6.4
ARCH := amd64
TGZ_FILE := pandoc-$(VER)-linux-$(ARCH).tar.gz

cache_dir := pandoc_cache
binary := pandoc
full_bin_path := $(cache_dir)/$(ARCH)/opt/bin/$(binary)
meta_file_path := $(cache_dir)/$(ARCH)/opt/PACK_META.txt # trick!
URL := https://github.com/jgm/pandoc/releases/download/$(VER)/$(TGZ_FILE)

$(cache_dir)/$(ARCH)_${binary}_${VER}_layer.zip: $(meta_file_path)
	cd $(cache_dir)/$(ARCH) ; zip -yr $(notdir $@) .
	mv $(cache_dir)/$(ARCH)/$(notdir $@) ${cache_dir}

$(meta_file_path): $(cache_dir)/$(TGZ_FILE)
	tar -xzf $< -C $(dir $<)
	mkdir -p $(cache_dir)/$(ARCH)
	mv "$(cache_dir)/$$(tar -tf $< | head -n1)" "$(cache_dir)/$(ARCH)/opt"
	echo "Packaged: $$(date)" > $@

$(cache_dir)/$(TGZ_FILE):
	mkdir -p $(dir $@)
	curl -L $(URL) > $@

clean:
	rm -rf pandoc_cache


