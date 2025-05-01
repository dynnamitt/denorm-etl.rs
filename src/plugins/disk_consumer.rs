use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

use crate::common::ResBoxed;
use crate::pipeline::consumer::Consumer;
use crate::pipeline::Item;

const INDENT: &str = " 💾";

#[derive(Debug)]
pub struct DataDir {
    dest_dir: PathBuf,
    // TODO: suffix
    // TODO: impl Display?
}

impl DataDir {
    #[allow(dead_code)]
    pub async fn new(dir_name: impl AsRef<Path>) -> ResBoxed<Self> {
        tokio::fs::create_dir_all(&dir_name).await?;
        Ok(DataDir {
            dest_dir: PathBuf::from(dir_name.as_ref()),
        })
    }

    //TODO:
    // write asysnc
    async fn write_file(&self, prefix: &str, data: impl AsRef<[u8]>) -> ResBoxed<()> {
        let suffix = "_plain.txt";
        let output_file = self.dest_dir.join(format!("{}{}", prefix, suffix));
        tokio::fs::write(&output_file, data).await?;
        Ok(())
    }
}

#[async_trait]
impl<T: Item<Inner = String> + Send + 'static> Consumer<T> for DataDir {
    async fn pull(&self, mut rx: mpsc::Receiver<T>) -> ResBoxed<usize> {
        let mut count = 0;

        while let Some(item) = rx.recv().await {
            let key = item.key();
            let content = item.into_inner(); // Assuming `Item` has this method
            match self.write_file(&key, content).await {
                Ok(_) => {
                    count += 1;
                    println!("{} Plain text saved to {:?}/{}", INDENT, self, key);
                }
                Err(err) => {
                    eprintln!("Disk-save Fail: {}", err);
                }
            };
            count += 1;
        }

        Ok(count)
    }
}
