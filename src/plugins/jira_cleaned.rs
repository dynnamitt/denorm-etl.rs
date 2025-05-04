use askama::Template;
use async_trait::async_trait;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use crate::common::*;
use crate::pipeline::transformer::Transformer;
use crate::pipeline::DenormalizedItm;
use crate::pipeline::Item;
use crate::plugins::jira::TicketFields;
use crate::plugins::text_proc::transform;

const INDENT: &str = " ⌛";

#[derive(Template)]
#[template(path = "plain_ticket.templ.txt")] // TODO: load from disk?
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

#[derive(Clone)]
pub struct JiraIntoPlain {
    skip_pandoc: bool, //debug
}
impl JiraIntoPlain {
    pub fn new(skip_pandoc: bool) -> Self {
        Self { skip_pandoc }
    }
    pub async fn prep_and_render(&self, fields: TicketFields, key: String) -> ResBoxed<String> {
        // Process comments concurrently with Tokio-native concurrency
        let mut comment_tasks = JoinSet::new();
        // Spawn all comment processing
        let skip = self.skip_pandoc;
        let tr_ = async move |s: String| {
            if skip {
                Ok(s)
            } else {
                transform(s).await
            }
        };
        for c in fields.comment.comments {
            comment_tasks.spawn(async move {
                let cleaned_body = tr_(c.body.clone()).await;
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
        let clean_descr = tr_(fields.description.clone()).await;

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
}

#[async_trait]
impl<I> Transformer<I, DenormalizedItm> for JiraIntoPlain
where
    I: Item<Inner = TicketFields> + Sync + Send + 'static,
{
    async fn transform(
        &self,
        mut rx: mpsc::Receiver<I>,
        tx: mpsc::Sender<DenormalizedItm>,
    ) -> ResBoxed<()> {
        while let Some(item) = rx.recv().await {
            let key = item.key();
            let fields = item.into_inner();

            // TODO: pool proc maybe?
            let start = Instant::now();
            println!("{} Transforming {} ...", INDENT, key);
            let denorm_version = self.prep_and_render(fields, key.clone()).await?;

            let duration = start.elapsed();
            println!("{} Transform for {} took {:?}", INDENT, key, duration);
            let push_msg = DenormalizedItm(key, denorm_version);

            // send the result; exit if the receiver has been dropped
            if tx.send(push_msg).await.is_err() {
                break;
            }
        }
        Ok(())
    }
}
