mod pipeline;
mod plugins;
use tokio::sync::mpsc;
// mod memchr_test;

use std::env;

use pipeline::consumer::Consumer;
use pipeline::producer::Producer;
use pipeline::transformer::Transformer;

//use crate::plugins::disk_consumer::DataDir;
use plugins::jira_cleaned::JiraIntoPlain;
use plugins::jira_producer::JiraInput;
use plugins::s3_consumer::S3Upload;

// producer
const BATCH_SIZE: u32 = 15;
const OUT_DIR: &str = "_disk_consumer_test";

// Some shared "grunt"
mod common {
    use std::error::Error;
    pub const PRODUCER_WAIT: u64 = 150;
    pub const BUCKET_NAME: &str = "jira-cleaned-for-inference";
    pub type ResBoxed<T> = Result<T, Box<dyn Error + Sync + Send>>;
}

#[tokio::main]
async fn main() -> common::ResBoxed<()> {
    // Jira configuration
    let base_url = env::var("JIRA_ENDPOINT").expect("env JIRA_ENDPOINT");
    let jql = env::var("JIRA_JQL").expect("env JIRA_JQL");
    let token = env::var("JIRA_TOKEN").expect("env JIRA_TOKEN");
    let _bucket = env::var("JIRA_DEST_S3");

    // TODO: Make(tm) a factory solution for this part

    // Create channels
    let (tx_fetch, rx_transform) = mpsc::channel(100);
    let (tx_transform, rx_upload) = mpsc::channel(100);

    // Spawn the tasks
    let producer_task = tokio::spawn({
        async move {
            // Search for tickets using JQL
            println!("Producing ticket(s) with JQL: {}", jql);
            let jira_inp = JiraInput::new(base_url, token, jql);
            match jira_inp.push(BATCH_SIZE, tx_fetch).await {
                Ok(count) => println!("Fetched {} JIRA tickets", count),
                Err(e) => eprintln!("Error fetching JIRA issues: {}", e),
            }
        }
    });

    let transf_task = tokio::spawn({
        async move {
            // Search for tickets using JQL
            println!("Test-tranform setup: ");
            let transf = JiraIntoPlain {};
            match transf.transform(rx_transform, tx_transform).await {
                Ok(_) => println!("Transformer done"),
                Err(e) => eprintln!("Transformer issues: {}", e),
            }
        }
    });

    /* DEBUG consumer
    let filestore = DataDir::new(OUT_DIR).await.unwrap();
    let consumer_task = tokio::spawn({
        async move {
            println!("Consumer set to disk");
            match filestore.pull(rx_upload).await {
                Ok(count) => println!("Saved {} documents.", count),
                Err(e) => eprintln!("Error saving: {}", e),
            }
        }
    });*/

    let s3_upload = S3Upload::new(common::BUCKET_NAME, "some-jql/").await?;
    let consumer_task = tokio::spawn({
        async move {
            println!("Consumer set to s3-upload");
            match s3_upload.pull(rx_upload).await {
                Ok(count) => println!("Saved {} objects.", count),
                Err(e) => eprintln!("Error saving: {}", e),
            }
        }
    });

    // Wait for all tasks to complete
    let _ = tokio::join!(producer_task, transf_task, consumer_task);
    Ok(())
}
