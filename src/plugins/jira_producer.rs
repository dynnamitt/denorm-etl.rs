use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;
use serde_json::Value;

use serde::Deserialize;

use crate::common::ResBoxed;
use crate::pipeline::producer::{InputEndpoint, Producer};
use crate::plugins::jira::{JiraTicket, EXTRA_FIELDS};

const API_SEGM: &str = "/rest/api/2";
const JSON_HTTP: &str = "application/json";

#[derive(Deserialize, Debug)]
struct JiraSearchResult {
    issues: Vec<JiraTicket>,
    total: u32,
    #[serde(rename = "maxResults")]
    #[allow(dead_code)]
    max_results: u32,
    #[serde(rename = "startAt")]
    #[allow(dead_code)]
    start_at: u32,
}

pub struct JiraInput {
    base_url: String,
    token: String,
    query: String,
    http_client: reqwest::Client,
}

impl JiraInput {
    pub fn new<T: ToString>(base_url: T, token: T, query: T) -> Self {
        Self {
            base_url: base_url.to_string(),
            token: token.to_string(),
            query: query.to_string(),
            http_client: reqwest::Client::new(),
        }
    }

    fn api_post(&self, path: &str) -> RequestBuilder {
        let url = format!("{}{}{}", self.base_url, API_SEGM, path);
        //println!("url={}", url);
        self.http_client
            .post(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.token)) // Set Auth header
            .header(CONTENT_TYPE, JSON_HTTP)
            .header(ACCEPT, JSON_HTTP)
    }

    async fn search_call<T: DeserializeOwned>(&self, jbody: Value) -> ResBoxed<T> {
        let jira_search = self
            .api_post("/search")
            .json(&jbody)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())?
            // TODO: improve err-msg
            .json()
            .await?;
        Ok(jira_search)
    }
}

impl InputEndpoint<JiraTicket> for JiraInput {
    // Use this to find a total (count)
    // Will not return any issues/tickets
    async fn count(&self) -> ResBoxed<u32> {
        let jbody = serde_json::json!({
            "jql":self.query,
            "maxResults":0 // Only count
        });

        //println!("url={}", url);
        let res: ResBoxed<JiraSearchResult> = self.search_call(jbody).await;

        res.map(|r| r.total)
    }
    // Fetch the real tickets/issues in batches
    // makes async/parallel post-handling simpler
    async fn query(&self, start: u32, max: u32) -> ResBoxed<Vec<JiraTicket>> {
        let jbody = serde_json::json!({
            "jql":self.query,
            "startAt":start,
            "maxResults":max,
            "fields":EXTRA_FIELDS,

        });
        let res: JiraSearchResult = self.search_call(jbody).await?;
        Ok(res.issues)
    }
}

// formallity;
//  also impl the msg-passing subTrait
impl<T: InputEndpoint<JiraTicket>> Producer<JiraTicket> for T {}
