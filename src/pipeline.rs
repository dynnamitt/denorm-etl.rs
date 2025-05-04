pub mod consumer;
pub mod producer;
pub mod transformer;

pub type Key = String;
pub trait Item {
    type Inner;
    fn key(&self) -> Key;
    fn into_inner(self) -> Self::Inner;
}

pub struct DenormalizedItm(pub Key, pub String);

impl Item for DenormalizedItm {
    type Inner = String;
    fn key(&self) -> String {
        self.0.clone()
    }
    fn into_inner(self) -> Self::Inner {
        self.1
    }
}
#[allow(dead_code)]
// TODO: swap to this ..just maybe
pub trait Msg<T> {
    fn key(&self) -> String;
    fn payload(self) -> T;
}
