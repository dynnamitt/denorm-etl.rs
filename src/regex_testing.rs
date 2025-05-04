#[allow(dead_code)]
// TODO: remove
fn strip_base64_images(content: &str, ticket_key: &str) -> String {
    // Create output directory if it doesn't exist
    //fs::create_dir_all(output_dir)?; TAKEN OUT .. for now

    // Regex to find base64 encoded images in the content
    // FIXME: stream from the content and MAKE THIS BETTER
    let re = Regex::new(r"data:image/([a-zA-Z0-9]+);base64,([A-Za-z0-9+/=]+)").unwrap();

    // FIXME: DONT CLONE that str !!
    let mut result = content.to_string();
    let mut image_counter = 0;

    for cap in re.captures_iter(content) {
        let img_type = &cap[1];
        let _base64_data = &cap[2]; // skip scan/trace for now !!!!

        // Decode base64 data
        //let image_data = decode(base64_data)?;

        // Create a file for the image
        image_counter += 1;
        let file_name = format!("{}_{}.{}", ticket_key, image_counter, img_type);
        //let file_path = output_dir.join(&file_name);

        // Write image data to file
        //let mut file = fs::File::create(&file_path)?;
        //file.write_all(&image_data)?;

        // Replace the base64 data with a link to the image file
        let image_reference = format!("!{}!", file_name);
        println!("  Took out an image: {}", file_name);
        result = result.replace(&cap[0], &image_reference);
    }

    content.to_string()
}
