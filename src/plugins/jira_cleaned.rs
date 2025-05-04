use askama::Template;
use async_trait::async_trait;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use crate::common::*;
use crate::pipeline::transformer::Transformer;
use crate::pipeline::Item;
use crate::plugins::jira::TicketFields;
use crate::plugins::text_proc::transform_with_proc;
use regex::Regex;

pub struct JiraPlain(String, String);

impl Item for JiraPlain {
    type Inner = String;
    fn key(&self) -> String {
        self.0.clone()
    }
    fn into_inner(self) -> Self::Inner {
        self.1
    }
}
const INDENT: &str = " ⌛";

pub struct JiraIntoPlain {
    skip_pandoc: bool, //debug
}
impl JiraIntoPlain {
    pub fn new(skip_pandoc: bool) -> Self {
        Self { skip_pandoc }
    }
}

#[async_trait]
impl<I> Transformer<I, JiraPlain> for JiraIntoPlain
where
    I: Item<Inner = TicketFields> + Sync + Send + 'static,
{
    async fn transform(
        &self,
        mut rx: mpsc::Receiver<I>,
        tx: mpsc::Sender<JiraPlain>,
    ) -> ResBoxed<()> {
        while let Some(item) = rx.recv().await {
            let key = item.key();
            let fields = item.into_inner();

            // TODO: pool proc maybe?
            let start = Instant::now();
            let plain_version = prep_and_render(fields, key.clone()).await?;

            let duration = start.elapsed();
            println!("{} Transform for {} took {:?}", INDENT, key, duration);
            let push_msg = JiraPlain(key, plain_version);

            // send the result; exit if the receiver has been dropped
            if tx.send(push_msg).await.is_err() {
                break;
            }
        }
        Ok(())
    }
}

pub async fn prep_and_render(fields: TicketFields, key: String) -> ResBoxed<String> {
    // Process comments concurrently with Tokio-native concurrency
    let mut comment_tasks = JoinSet::new();
    // Spawn all comment processing tasks
    for c in fields.comment.comments {
        comment_tasks.spawn(async move {
            let cleaned_body = transform_with_proc(c.body.clone()).await;
            SimplerComment {
                body: cleaned_body.unwrap_or(c.body),
                author_id: c.author.name,
                created: c.created,
            }
        });
    }
    // Collect results as they complete
    let mut cs = Vec::with_capacity(comment_tasks.len());
    while let Some(res) = comment_tasks.join_next().await {
        cs.push(res?); // Handle both join error and your application error
    }

    // Process description
    let clean_descr = transform_with_proc(fields.description.clone()).await;

    let t = SimplerJiraTicket {
        key: key.clone(),
        summary: fields.summary,
        assignee_id: fields.assignee.name,
        reporter_id: fields.reporter.name,
        created: fields.created,
        description: clean_descr.unwrap_or(fields.description),
        comments: cs,
    };

    t.render().map_err(|e| e.into())
}

#[allow(dead_code)]
// TODO: remove
fn strip_base64_images(content: &str, ticket_key: &str) -> String {
    // Create output directory if it doesn't exist
    //fs::create_dir_all(output_dir)?; TAKEN OUT .. for now

    // Regex to find base64 encoded images in the content
    // FIXME: stream from the content and MAKE THIS BETTER
    let re = Regex::new(r"data:image/([a-zA-Z0-9]+);base64,([A-Za-z0-9+/=]+)").unwrap();

    // FIXME: DONT CLONE that str !!
    let mut result = content.to_string();
    let mut image_counter = 0;

    for cap in re.captures_iter(content) {
        let img_type = &cap[1];
        let _base64_data = &cap[2]; // skip scan/trace for now !!!!

        // Decode base64 data
        //let image_data = decode(base64_data)?;

        // Create a file for the image
        image_counter += 1;
        let file_name = format!("{}_{}.{}", ticket_key, image_counter, img_type);
        //let file_path = output_dir.join(&file_name);

        // Write image data to file
        //let mut file = fs::File::create(&file_path)?;
        //file.write_all(&image_data)?;

        // Replace the base64 data with a link to the image file
        let image_reference = format!("!{}!", file_name);
        println!("  Took out an image: {}", file_name);
        result = result.replace(&cap[0], &image_reference);
    }

    content.to_string()
}
#[derive(Template)]
#[template(path = "plain_ticket.templ.txt")]
struct SimplerJiraTicket {
    pub key: String,
    pub summary: String,
    pub assignee_id: String,
    pub reporter_id: String,
    pub created: String,
    pub description: String,
    pub comments: Vec<SimplerComment>,
}

#[derive(Debug)]
struct SimplerComment {
    pub body: String,
    pub author_id: String,
    pub created: String,
}
