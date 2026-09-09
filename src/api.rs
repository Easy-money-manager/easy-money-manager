use crate::record::Record;
use crate::sheetcollection::SheetCollection;
use crate::requests::{ GetRecordsResponse, CreateRecordRequest, CreateRecordResponse, UpdateRecordRequest, UpdateRecordResponse, RemoveRecordResponse, BootstrapResponse };

#[derive(Clone)]
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
        let url = format!("{}/records/sheet{}", self.base_url, sheet_id);

        let response = self.client.get(url).send().await?;

        let response = response.json::<GetRecordsResponse>().await?;

        Ok(response.records)
    }
    pub async fn create_record(&self, sheet_id: i64, request: &CreateRecordRequest) -> Result<i64, reqwest::Error> {
        let url = format!("{}/records/sheet{}", self.base_url, sheet_id);

        let response = self.client.post(url).json(request).send().await?;

        let response = response.json::<CreateRecordResponse>().await?;

        Ok(response.id)
    }
    pub async fn update_record(&self, record_id: i64, request: &UpdateRecordRequest) -> Result<(), reqwest::Error> {
        let url = format!("{}/records/{}", self.base_url, record_id);

        let response = self.client.put(url).json(request).send().await?;

        let _response = response.json::<UpdateRecordResponse>().await?;

        Ok(())
    }
    pub async fn remove_record(&self, record_id: i64) -> Result<(), reqwest::Error> {
        let url = format!("{}/records/{}", self.base_url, record_id);

        let response = self.client.delete(url).send().await?;

        let _response = response.json::<RemoveRecordResponse>().await?;

        Ok(())
    }
    pub async fn get_bootstrap(&self) -> Result<Vec<SheetCollection>, reqwest::Error> {
        let url = format!("{}/bootstrap", self.base_url);

        let response = self.client.post(url).send().await?;

        let response = response.json::<BootstrapResponse>().await?;

        Ok(response.collections)
    }
}
