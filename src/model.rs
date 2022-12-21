use serde::{Deserialize, Serialize};

// the input to our `scrape` & `scrape-js` handler
#[derive(Serialize, Deserialize)]
pub struct ScrapeRequestBody {
    pub url: String,
    #[serde(rename = "waitMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_ms: Option<u32>,
}

// the input to our `create_user` handler
#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
pub struct User {
    pub id: u64,
    pub username: String,
}
