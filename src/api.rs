use crate::record::Record;

pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url: base_url,
            client: reqwest::Client::new(),
        }
    }
    pub async fn get_records(&self, sheet_id: i64) -> Result<Vec<Record>, reqwest::Error> {
        todo!();
    }
}

