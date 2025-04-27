pub mod consumer;
pub mod producer;
pub mod transformer;

pub trait Item {
    type Inner;
    fn key(&self) -> String;
    fn into_inner(self) -> Self::Inner;
}

#[allow(dead_code)]
// TODO: swap to this ..just maybe
pub trait Msg<T> {
    fn key(&self) -> String;
    fn payload(self) -> T;
}
