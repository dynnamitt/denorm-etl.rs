use crate::common::*;
use async_trait::async_trait;
use tokio::sync::mpsc;

#[async_trait]
pub trait Consumer<O> {
    async fn pull(&self, rx: mpsc::Receiver<O>) -> ResBoxed<usize>;
}
