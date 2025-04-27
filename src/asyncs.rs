mod input;
mod jira_cleaned;
mod jira_client;
mod jira_raw;
mod s3;
mod text;

use crate::input::{InputEndpoint, ResBoxed};

use jira_client::JiraInput;
use std::env;
use std::fs;
use std::path::Path;

use tokio::task::JoinSet;

const BATCH_SIZE: u32 = 10;

// Save the plain text output
// match bucket {
//     Ok(ref b) => {
//         let config = aws_config::load_from_env().await;
//         let _s3 = aws_sdk_s3::Client::new(&config);
//         println!("s3 it is: {}", b);
//     }
//     Err(ref _e) => {
//         println!("err {}", _e);
//         let output_file = output_dir.join(format!("{}_plain.txt", key));
//         fs::write(&output_file, full_text)?;
//         println!("  Plain text saved to {}", output_file.display());
//     }
// };

// async fn exec_ticket_batch(
//     jclient: impl InputEndpoint<serde_json::Value>,
//     jql: &str,
//     num: u32,
//     dest_dir: &'static Path,
// ) -> ResBoxed<()> {
//     let mut join_set: JoinSet<ResBoxed<()>> = JoinSet::new();
//
//     println!(">> Defining batch # {}", num);
//
//     let tix = jclient.query((num - 1) * BATCH_SIZE, BATCH_SIZE).await?;
//
//     let batch_len = tix.len();
//     // Process each ticket
//     for (index, ticket) in tix.into_iter().enumerate() {
//         join_set.spawn(async move {
//             async {
//                 let key = ticket.key.clone();
//                 println!(
//                     "Batch # {}: [{} of {}] ticket: {} - {} - {}",
//                     num,
//                     index + 1,
//                     batch_len,
//                     key,
//                     ticket.fields.summary,
//                     ticket.fields.created
//                 );
//                 let output_file = dest_dir.join(format!("{}_plain.txt", key));
//                 if let Ok(plain) = jira_cleaned::render(ticket) {
//                     tokio::fs::write(&output_file, plain).await?;
//                     println!("  Plain text saved to {}", output_file.display());
//                 };
//                 Ok(())
//             }
//             .await
//         });
//     }
//     println!(">> Awaiting batch # {}", num);
//     join_set.join_all().await;
//
//     println!("Batch # {} executed", num);
//     Ok(())
// }

// Calculate the batch range
fn batch_range(total: u32) -> std::ops::Range<u32> {
    if total < BATCH_SIZE {
        1..1
    } else {
        let b = BATCH_SIZE - 1;
        let r = if total % b > 0 { 1 } else { 0 };
        1..(total / b) + r
    }
}
