//mod memchr_test;
mod pipeline;
mod pipeline_factory;
mod plugins;

// Some shared "grunt"
mod common {
    use std::error::Error;
    pub const PRODUCER_BATCH_SIZE: usize = 15;
    pub const PRODUCER_WAIT: u64 = 1500; // TODO: tune down when deNorm is MORE efficient
    pub type ResBoxed<T> = Result<T, Box<dyn Error + Sync + Send>>;

    #[allow(dead_code)]
    pub const OUT_DIR: &str = "_disk_consumer_debug";
}

use common::*;

#[tokio::main]
async fn main() -> ResBoxed<()> {
    pipeline_factory::create().await
}
