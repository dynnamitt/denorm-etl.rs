use crate::common::*;
use async_trait::async_trait;
use tokio::sync::mpsc;

#[async_trait]
pub trait Transformer<I, O> {
    async fn transform(&self, rx: mpsc::Receiver<I>, tx: mpsc::Sender<O>) -> ResBoxed<()>;
}
