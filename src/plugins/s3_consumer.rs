use crate::common::*;

use crate::pipeline::consumer::Consumer;
use crate::pipeline::Item;
use crate::plugins::s3;
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client as S3Client;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

pub struct Bucket {
    name: String,
    prefix: String,
    client: S3Client,
}

impl Bucket {
    pub async fn new(name: &str, prefix: &str) -> ResBoxed<Self> {
        let aws_config = aws_config::defaults(BehaviorVersion::latest()).load().await;
        let client = S3Client::new(&aws_config);

        Ok(Bucket {
            name: name.to_string(),
            client,
            prefix: prefix.to_string(),
        })
    }
}

#[async_trait]
impl<T: Item<Inner = String> + Send + 'static> Consumer<T> for Bucket {
    async fn pull(&self, rx: mpsc::Receiver<T>) -> ResBoxed<usize> {
        let rx_stream = ReceiverStream::new(rx);

        // Process up to 10 uploads concurrently
        let mut s3_stream = rx_stream
            .map(|itm| async move {
                let client = self.client.clone();
                let bucket_name = self.name.clone();
                let prefix = self.prefix.clone();
                let itm_key = itm.key();
                let contents = itm.into_inner();
                let key = format!("{}/{}.txt", prefix, &itm_key);
                let put_res = s3::upload_object(&client, bucket_name, contents, key).await;
                match put_res {
                    Ok(_) => {
                        println!("s3: Successfully uploaded {}", itm_key);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("s3: Failed to upload {}: {}", itm_key, e);
                        Err(e)
                    }
                }
            })
            .collect::<Vec<_>>() // Need to collect to actually run the map eagerly
            .await;

        let mut join_set = JoinSet::new();

        while let Some(fut) = s3_stream.next().await {
            join_set.spawn(fut);
        }
        // Await all the spawned tasks
        let mut count = 0;
        while let Some(put_res) = join_set.join_next().await {
            count += 1;
        }
        Ok(count)
    }
}
