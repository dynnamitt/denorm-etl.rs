use crate::common::*;

use crate::pipeline::consumer::Consumer;
use crate::pipeline::Item;
use crate::plugins::s3;
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client as S3Client;
use tokio::sync::mpsc;

pub struct S3Upload {
    name: String,
    prefix: String,
    client: S3Client,
}

impl S3Upload {
    pub async fn new(name: &str, prefix: &str) -> ResBoxed<Self> {
        let aws_config = aws_config::defaults(BehaviorVersion::latest()).load().await;
        let client = S3Client::new(&aws_config);

        Ok(S3Upload {
            name: name.to_string(),
            client,
            prefix: prefix.to_string(),
        })
    }
}

#[async_trait]
impl<T: Item<Inner = String> + Send + 'static> Consumer<T> for S3Upload {
    async fn pull(&self, mut rx: mpsc::Receiver<T>) -> ResBoxed<usize> {
        let mut count = 0;

        while let Some(item) = rx.recv().await {
            let itm_key = item.key();
            let content = item.into_inner(); // Assuming `Item` has this method
            let obj_key = format!("{}{}", self.prefix, itm_key);
            let _put = s3::upload_object(&self.client, &self.name, content, obj_key).await;

            count += 1;
        }
        Ok(count)
    }
}
