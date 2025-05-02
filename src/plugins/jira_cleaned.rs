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

pub struct JiraIntoPlain {}

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
            println!("{} Transformer for {} took {:?}", INDENT, key, duration);
            let push_msg = JiraPlain(key, plain_version);

            // send the result; exit if the receiver has been dropped
            if tx.send(push_msg).await.is_err() {
                break;
            }
        }
        Ok(())
    }
}

// TODO: make this into a lib that spawns its own REQUIRED number of channels
pub async fn prep_and_render(fields: TicketFields, key: String) -> ResBoxed<String> {
    // Process comments concurrently with Tokio-native concurrency
    let mut comment_tasks = JoinSet::new();
    // Spawn all comment processing tasks
    for c in fields.comment.comments {
        comment_tasks.spawn(async move {
            // let cleaned_body = transform_with_proc(c.body.clone()).await;
            let cleaned_body = c.body;
            SimplerComment {
                // body: cleaned_body.unwrap_or(c.body),
                body: cleaned_body,
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
    //let clean_descr = transform_with_proc(fields.description.clone()).await;
    let clean_descr = fields.description; // debug

    let t = SimplerJiraTicket {
        key: key.clone(),
        summary: fields.summary,
        assignee_id: fields.assignee.name,
        reporter_id: fields.reporter.name,
        created: fields.created,
        //description: clean_descr.unwrap_or(fields.description),
        description: clean_descr,
        comments: cs,
    };

    t.render().map_err(|e| e.into())
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
