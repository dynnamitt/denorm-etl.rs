use crate::common::{self, *};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

pub trait InputEndpoint<T> {
    async fn count(&self) -> ResBoxed<u32>;
    async fn query(&self, start: u32, max: u32) -> ResBoxed<Vec<T>>;
}

use crate::pipeline::Item;
pub trait Producer<T: Item>: InputEndpoint<T> {
    // implement via SuperTrait
    async fn push(&self, batch_size: u32, tx: mpsc::Sender<T>) -> ResBoxed<usize> {
        let total = self.count().await?;
        if total == 0 {
            Err("No data to process".to_string().into())
        } else {
            // loop and push
            let r = batch_range(total, batch_size);
            println!("Producer batch range: {:#?}", r);
            for idx in r {
                let start = (idx - 1) * batch_size;
                println!("Ready batch #{}", idx);
                match self.query(start, batch_size).await {
                    // ERROR-handling
                    Ok(inp_items) => {
                        for itm in inp_items {
                            let key = itm.key();
                            println!("Producer passing item.. {}", key);
                            if (tx.send(itm).await).is_err() {
                                eprintln!("Pipeline dropped {} in batch {}", key, idx);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("BATCH {} INPUT query call failed: {}", idx, e);
                        // You might want to implement retry logic here
                        continue;
                    }
                }
                sleep(Duration::from_millis(common::PRODUCER_WAIT)).await;
            }
            Ok(total as usize)
        }
    }
}

// Calculate the batch range
#[allow(dead_code)]
fn batch_range(total: u32, breakup: u32) -> std::ops::Range<u32> {
    if total < breakup {
        1..2
    } else {
        let b = breakup;
        let r = if total % b > 0 { 2 } else { 1 };
        1..(total / b) + r
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_above_break() {
        assert_eq!(batch_range(12, 10).start, 1);
        assert_eq!(batch_range(12, 10).end, 3);
    }

    #[test]
    fn test_range_below_break() {
        assert_eq!(batch_range(9, 10).start, 1);
        assert_eq!(batch_range(9, 10).end, 2);
    }
}
