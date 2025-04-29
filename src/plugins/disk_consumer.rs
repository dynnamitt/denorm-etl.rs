use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

use crate::common::ResBoxed;
use crate::pipeline::consumer::Consumer;
use crate::pipeline::Item;

pub struct DataDir {
    dest_dir: PathBuf,
}

impl DataDir {
    pub async fn new(dir_name: impl AsRef<Path>) -> ResBoxed<Self> {
        tokio::fs::create_dir_all(&dir_name).await?;
        Ok(DataDir {
            dest_dir: PathBuf::from(dir_name.as_ref()),
        })
    }

    //TODO:
    // write asysnc
    async fn write_file(&self, prefix: impl ToString, data: impl AsRef<[u8]>) -> ResBoxed<()> {
        let output_file = self
            .dest_dir
            .join(format!("{}_plain.txt", prefix.to_string()));
        tokio::fs::write(&output_file, data).await?;
        println!("Plain text saved to {}", output_file.display());
        Ok(())
    }
}

// #[async_trait]
// // skip Transformer (test) version
// impl<T: Item<Inner = TicketFields> + Send + 'static> Consumer<T> for DataDir {
//     async fn pull(&self, mut rx: mpsc::Receiver<T>) -> ResBoxed<usize> {
//         let mut count = 0;
//
//         while let Some(item) = rx.recv().await {
//             let key = item.key();
//             let fields = item.into_inner(); // Assuming `Item` has this method
//             self.write_file(key, fields.summary).await?;
//             count += 1;
//         }
//
//         Ok(count)
//     }
// }

#[async_trait]
impl<T: Item<Inner = String> + Send + 'static> Consumer<T> for DataDir {
    async fn pull(&self, mut rx: mpsc::Receiver<T>) -> ResBoxed<usize> {
        let mut count = 0;

        while let Some(item) = rx.recv().await {
            let key = item.key();
            let content = item.into_inner(); // Assuming `Item` has this method
            self.write_file(key, content).await?;
            count += 1;
        }

        Ok(count)
    }
}
