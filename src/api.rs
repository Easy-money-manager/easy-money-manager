use emm_shared::sheetcollection::SheetCollection;
use emm_shared::request::{ RegisterRequest, LoginRequest, CreateRecordRequest,  UpdateRecordRequest };
use emm_shared::response::{ LoginResponse, CreateRecordResponse, BootstrapResponse };

#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            #[cfg(debug_assertions)]
            base_url: "http://127.0.0.1:3000".to_string(),
            #[cfg(not(debug_assertions))]
            base_url: "https://api.easy-money-manager.com".to_string(),
            client: reqwest::Client::new(),
        }
    }
/*    pub async fn get_records(&self, sheet_id: i64) -> Result<Vec<Record>, reqwest::Error> {
        let url = format!("{}/records/sheet/{}", self.base_url, sheet_id);

        let response = self.client.get(url).send().await?;

        let response = response.json::<GetRecordsResponse>().await?;

        Ok(response.records)
    }*/
    pub async fn register(&self, request: &RegisterRequest) -> Result<(), reqwest::Error> {
        let url = format!("{}/auth/register", self.base_url);
        self.client.post(url).json(request).send().await?.error_for_status()?;
        Ok(())
    }
    pub async fn login(&self, request: &LoginRequest) -> Result<LoginResponse, reqwest::Error> {
        let url = format!("{}/auth/login", self.base_url);
        let response = self.client.post(url).json(request).send().await?.error_for_status()?.json::<LoginResponse>().await?;
        Ok(response)
    }
    pub async fn logout(&self, session_token: &str) -> Result<(), reqwest::Error> {
        let url = format!("{}/auth/logout", self.base_url);
        self.client.post(url).bearer_auth(session_token).send().await?.error_for_status()?;
        Ok(())
    }
    pub async fn create_record(&self, session_token: &str, sheet_id: i64, request: &CreateRecordRequest) -> Result<i64, reqwest::Error> {
        let url: String= format!("{}/records/sheet/{}", self.base_url, sheet_id);

        let response = self.client.post(url).bearer_auth(session_token).json(request).send().await?.json::<CreateRecordResponse>().await?;

        Ok(response.id)
    }
    pub async fn update_record(&self, session_token: &str, record_id: i64, request: &UpdateRecordRequest) -> Result<(), reqwest::Error> {
        let url = format!("{}/records/{}", self.base_url, record_id);

        self.client.put(url).bearer_auth(session_token).json(request).send().await?.error_for_status()?;

        Ok(())
    }
    pub async fn remove_record(&self, session_token: &str, record_id: i64) -> Result<(), reqwest::Error> {
        let url = format!("{}/records/{}", self.base_url, record_id);

        self.client.delete(url).bearer_auth(session_token).send().await?.error_for_status()?;

        Ok(())
    }
    pub async fn get_bootstrap(&self, session_token: &str) -> Result<Vec<SheetCollection>, reqwest::Error> {
        let url = format!("{}/bootstrap", self.base_url);
        eprintln!("Getting bootstrap from: {}", url);

        let response = self.client.get(url).bearer_auth(session_token).send().await?.error_for_status()?.json::<BootstrapResponse>().await?;

        Ok(response.collections)
    }
}
