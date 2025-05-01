use std::env::{self, JoinPathsError};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::pipeline::consumer::{self, Consumer};
use crate::pipeline::producer::Producer;
use crate::pipeline::transformer::Transformer;

//use crate::plugins::disk_consumer::DataDir;
use crate::plugins::disk_consumer::DataDir;
use crate::plugins::jira_cleaned::JiraIntoPlain;
use crate::plugins::jira_producer::JiraInput;
use crate::plugins::s3_consumer::S3Upload;

use super::common::*;

pub async fn create() -> ResBoxed<()> {
    // Create channels
    let (tx_fetch, rx_transform) = mpsc::channel(PRODUCER_BATCH_SIZE);
    let (tx_transform, rx_upload) = mpsc::channel(PRODUCER_BATCH_SIZE);

    // Spawn the tasks

    // -- Jira configuration
    let base_url = env::var("JIRA_ENDPOINT").expect("env JIRA_ENDPOINT");
    let jql = env::var("JIRA_JQL").expect("env JIRA_JQL");
    let token = env::var("JIRA_TOKEN").expect("env JIRA_TOKEN");
    let producer_task = tokio::spawn({
        async move {
            // Search for tickets using JQL
            println!("Producing ticket(s) with JQL: {}", jql);
            let jira_inp = JiraInput::new(base_url, token, jql);
            match jira_inp.push(PRODUCER_BATCH_SIZE as u32, tx_fetch).await {
                Ok(count) => println!("Fetched {} JIRA tickets", count),
                Err(e) => eprintln!("Error fetching JIRA issues: {}", e),
            }
        }
    });

    // DE_NORM
    // FIXME: use memchr , not regex
    // FIXME: use SAXHandler, not pandoc
    let transf_task = tokio::spawn({
        async move {
            // Search for tickets using JQL
            let transf = JiraIntoPlain {};
            match transf.transform(rx_transform, tx_transform).await {
                Ok(_) => println!("Transformer done"),
                Err(e) => eprintln!("Transformer issues: {}", e),
            }
        }
    });

    let debug = env::var("DEBUG_CONSUMER")
        .unwrap_or("true".into())
        .to_lowercase();

    let consumer_task = if debug == "true" {
        // DEBUG consumer
        let filestore = DataDir::new(OUT_DIR).await.unwrap();
        tokio::spawn({
            async move {
                println!("Consumer set to disk");
                match filestore.pull(rx_upload).await {
                    Ok(count) => println!("Saved {} documents.", count),
                    Err(e) => eprintln!("Error saving: {}", e),
                }
            }
        })
    } else {
        let bucket = env::var("JIRA_DEST_S3").unwrap_or("jira-cleaned-for-inference".into());
        let s3_upload = S3Upload::new(&bucket, "some-jql-2/", ".txt").await?;
        tokio::spawn({
            async move {
                println!("Consumer set to s3-upload");
                match s3_upload.pull(rx_upload).await {
                    Ok(count) => println!("Uploaded {} objects.", count),
                    Err(e) => eprintln!("Error saving: {}", e),
                }
            }
        })
    };

    // Wait for all tasks to complete
    let _ = tokio::join!(producer_task, transf_task, consumer_task);
    Ok(())
}
