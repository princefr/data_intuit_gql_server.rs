use async_graphql::*;
use chrono::{DateTime, Utc};

#[derive(SimpleObject, Debug, PartialEq, InputObject, Clone)]
#[graphql(input_name = "UserInput")]
pub struct User {
    pub id: Option<String>,
    pub name: String,
    pub email: String,
    #[graphql(secret)]
    pub password: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl User {
    pub fn fill_id(&mut self, id: String) {
        self.id = Some(id);
        self.created_at = Some(Utc::now());
        self.updated_at = Some(Utc::now());
    }
}
