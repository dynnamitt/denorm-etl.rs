use tokio::sync::mpsc;

use crate::pipeline::consumer::Consumer;
use crate::pipeline::producer::Producer;
use crate::pipeline::transformer::Transformer;

use crate::plugins::disk_consumer::DataDir;
use crate::plugins::jira_cleaned::JiraIntoPlain;
use crate::plugins::jira_producer::JiraInput;
use crate::plugins::s3_consumer::S3Upload;

use super::common::*;

pub async fn create() -> ResBoxed<()> {
    // Create channels
    let batch_size = get_conf("PRODUCER_BATCH_SIZE").parse::<usize>().unwrap();
    let (tx_fetch, rx_transform) = mpsc::channel(batch_size * 2);
    let (tx_transform, rx_upload) = mpsc::channel(batch_size * 2);

    // Spawn the tasks

    let producer_task = tokio::spawn({
        async move {
            let base_url = get_conf("JIRA_ENDPOINT");
            let jql = get_conf("JIRA_JQL");
            let token = get_conf("JIRA_TOKEN");

            if base_url.is_empty() || jql.is_empty() || token.is_empty() {
                eprintln!("!! Required JIRA-request ENVS not set. !!");
                panic!();
            }
            println!("Producing ticket(s) with JQL: {}", jql);
            let jira_inp = JiraInput::new(base_url, token, jql);
            match jira_inp.push(batch_size as u32, tx_fetch).await {
                Ok(count) => println!("Fetched {} JIRA tickets", count),
                Err(e) => eprintln!("Error fetching JIRA issues: {}", e),
            }
        }
    });

    // DE_NORM
    let skip_pandoc: bool = get_conf("SKIP_PANDOC")
        .to_lowercase()
        .parse()
        .unwrap_or_default();
    let transf_task = tokio::spawn({
        async move {
            // Search for tickets using JQL
            let transf = JiraIntoPlain::new(skip_pandoc);
            match transf.transform(rx_transform, tx_transform).await {
                Ok(_) => println!("Transformer done"),
                Err(e) => eprintln!("Transformer issues: {}", e),
            }
        }
    });

    let outdir = get_conf("DEBUG_DIR");

    let consumer_task = if !outdir.is_empty() {
        // DEBUG consumer
        let filestore = DataDir::new(outdir).await.unwrap();
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
        let bucket = get_conf("DEST_BUCKET");
        let prefix = get_conf("DEST_PREFIX"); // TODO slugify(JQL) when empty
        let s3_upload = S3Upload::new(&bucket, &prefix, ".txt").await?;
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
