use crate::pipeline::Item;
use serde::Deserialize;

//   Structs matching the JIRA API Response
//

#[derive(Deserialize, Debug)]
pub struct JiraTicket {
    pub key: String,
    pub fields: TicketFields,
}

impl Item for JiraTicket {
    type Inner = TicketFields; // assoc type
    fn key(&self) -> String {
        self.key.clone()
    }
    fn into_inner(self) -> Self::Inner {
        self.fields
    }
}

pub const EXTRA_FIELDS: [&str; 6] = [
    "summary",
    "description",
    "assignee",
    "reporter",
    "created",
    "comment",
];

#[derive(Deserialize, Debug)]
pub struct TicketFields {
    pub summary: String,
    pub description: String,
    pub assignee: User,
    pub reporter: User,
    pub created: String,
    pub comment: CommentSet,
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct CommentSet {
    pub comments: Vec<CommentData>,
}

#[derive(Deserialize, Debug)]
pub struct CommentData {
    pub body: String,
    pub author: User,
    pub created: String,
}
