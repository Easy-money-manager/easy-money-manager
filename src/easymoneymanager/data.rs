use super::EasyMoneyManager;
use crate::clienttask::ClientTask;
use crate::api::ApiClient;
use chrono::{ Datelike, Local };

use emm_shared::record::Record;
use emm_shared::sheetcollection::SheetCollection;
use emm_shared::sheet::SheetError;
use emm_shared::request::{ CreateRecordRequest, UpdateRecordRequest };


impl EasyMoneyManager {
    pub(super) fn start_bootstrap(&mut self) {
        let api_client = self.api_client.clone();
        let session_token: String = match self.session_token_clone() {
            Some(session_token) => session_token,
            None => panic!(),
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.bootstrap_task = Some(
                ClientTask::spawn(
                    &self.runtime,
                    async move {
                        api_client.get_bootstrap(&session_token).await
                    }
                )
            );
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.bootstrap_task = Some(
                ClientTask::spawn(async move {
                    api_client.get_bootstrap(&session_token).await
                })
            );
        }
    }
    pub(super) fn add_record(&mut self) {
        let record = match Record::from_input(
            &self.description,
            &self.year,
            &self.month,
            &self.day,
            &self.value_zl,
            &self.value_gr
        ) {
            Ok(record) => record, 
            Err(error) => {
                self.log_error(&format!("Failed parsing record to add: {}", error.message()));
                return;
            },
        };
        let collection_index: usize = self.active_collection;
        let sheet_index: usize = self.active_collection().active_sheet_index();
        let api_client = self.api_client.clone();
        let sheet_id = self.active_collection().active_sheet().id();
        let session_token: String= match self.session_token_clone() {
            Some(session_token) => session_token,
            None => panic!(),
        };

        let request: CreateRecordRequest = CreateRecordRequest {
            description: record.description().to_string(),
            date: record.date(),
            value: record.value(),
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.create_record_task = Some((
                    collection_index,
                    sheet_index,
                    record,
                    ClientTask::spawn(
                        &self.runtime,
                        async move { api_client.create_record(&session_token, sheet_id, &request).await }
                    )
            ));
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.create_record_task = Some((
                    collection_index,
                    sheet_index,
                    record,
                    ClientTask::spawn(async move { api_client.create_record(&session_token, sheet_id, &request).await }
                    )
            ));
        }
        self.input_reset();
    }
    pub(super) fn edit_record(&mut self, index: usize) {
        let record = match Record::from_input(
            &self.description,
            &self.year,
            &self.month,
            &self.day,
            &self.value_zl,
            &self.value_gr
        ) {
            Ok(record) => record, 
            Err(error) => {
                self.log_error(&format!("Failed parsing record to edit: {}", error.message()));
                return;
            },
        };

        let collection_index: usize = self.active_collection;
        let sheet_index: usize = self.active_collection().active_sheet_index();
        let record_id: i64 = self.active_collection().active_sheet().records[index].id();
        let session_token: String = match self.session_token_clone() {
            Some(session_token) => session_token,
            None => panic!(),
        };

        let request = UpdateRecordRequest {
            description: record.description().to_string(),
            date: record.date(),
            value: record.value(),
        };
        let api_client = self.api_client.clone();

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.update_record_task = Some((
                    collection_index,
                    sheet_index,
                    index,
                    record,
                    ClientTask::spawn(
                        &self.runtime,
                        async move { api_client.update_record(&session_token, record_id, &request).await }
                    )
            ));
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.update_record_task = Some((
                    collection_index,
                    sheet_index,
                    index,
                    record,
                    ClientTask::spawn(async move { api_client.update_record(&session_token, record_id, &request).await } )
            ));
        }
        self.input_reset();
    }
    pub(super) fn remove_record(&mut self, index: usize) {
        let api_client = self.api_client.clone();
        let collection_index: usize = self.active_collection;
        let sheet_index: usize = self.active_collection().active_sheet_index();
        let record_id = self.active_collection().active_sheet().records[index].id();
        let session_token: String = match self.session_token_clone() {
            Some(session_token) => session_token,
            None => panic!(),
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.remove_record_task = Some((
                    collection_index,
                    sheet_index,
                    index,
                    ClientTask::spawn(
                        &self.runtime,
                        async move { api_client.remove_record(&session_token, record_id).await }
                    )
            ));
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.remove_record_task = Some((
                    collection_index,
                    sheet_index,
                    index,
                    ClientTask::spawn(async move { api_client.remove_record(&session_token, record_id).await } )
            ));
        }
    }
    pub(super) fn handle_bootstrap_task(&mut self) {
        let finished: bool = match &self.bootstrap_task {
            Some(task) => task.is_finished(),
            None       => false,
        };
        if !finished {
            return;
        }
        let task = self.bootstrap_task.take().expect("bootstrap_task should exist when it's mared as finished");
        #[cfg(not(target_arch = "wasm32"))]
        let result = task.take(&self.runtime);
        #[cfg(target_arch = "wasm32")]
        let result = task.take();
        match result {
            Ok(Ok(collections)) => {
                self.sheet_collections = collections;
                self.bootstrap_loaded = true;
                self.log(&format!("Bootstrap loaded"));
                self.error_msg = None;
            }
            Ok(Err(error)) => self.log_error(&format!("Failed to load application: {}", error)),
            Err(_error)     => self.log_error(&format!("Async task failed")),
        }
    }
    pub(super) fn handle_create_task(&mut self) {
        let finished: bool = match &self.create_record_task {
            Some((_, _, _, task)) => task.is_finished(),
            None                  => false,
        };
        if !finished {
            return;
        }
        let (collection_index, sheet_index, mut record, task) = self.create_record_task.take().expect("create_record_task should exist when it's mared as finished");
        #[cfg(not(target_arch = "wasm32"))]
        let result = task.take(&self.runtime);
        #[cfg(target_arch = "wasm32")]
        let result = task.take();
        match result {
            Ok(Ok(id)) => {
                record.id_set(id);
                self.sheet_collections[collection_index].sheets[sheet_index].push(record);
                self.error_msg = None;
            }
            Ok(Err(error)) => self.log_error(&format!("[Server response]: Failed to create record: {}", error)),
            Err(_error)    => self.log_error(&format!("Async task failed")),
        }
    }
    pub(super) fn handle_edit_task(&mut self) {
        let finished = match &self.update_record_task {
            Some((_, _, _, _, task)) => task.is_finished(),
            None                     => false,
        };
        if !finished {
            return;
        }
        let (collection_index, sheet_index, index, record, task) = self.update_record_task.take().expect("update_record_task should exist when it's mared as finished");
        #[cfg(not(target_arch = "wasm32"))]
        let result = task.take(&self.runtime);
        #[cfg(target_arch = "wasm32")]
        let result = task.take();
        match result {
            Ok(Ok(()))     => {
                if let Err(SheetError::IndexOutOfBounds) = self.sheet_collections[collection_index].sheets[sheet_index].edit(index, record) {
                    self.log_error(&"Failed to edit record in client cache: Index out of bounds");
                }
                self.error_msg = None;
            }
            Ok(Err(error)) => self.log_error(&format!("[Server response] failed to edit record: {}", error)),
            Err(_error)    => self.log_error(&format!("Async task failed")),
        }
    }
    pub(super) fn handle_remove_task(&mut self) {
        let finished: bool = match &self.remove_record_task {
            Some((_, _, _, task)) => task.is_finished(),
            None                  => false,
        };
        if !finished {
            return;
        }
        let (collection_index, sheet_index, index, task) = self.remove_record_task.take().expect("remove_task should exist when it's mared as finished");
        #[cfg(not(target_arch = "wasm32"))]
        let result = task.take(&self.runtime);
        #[cfg(target_arch = "wasm32")]
        let result = task.take();
        match result {
            Ok(Ok(()))     => {
                if let Err(SheetError::IndexOutOfBounds) = self.sheet_collections[collection_index].sheets[sheet_index].remove(index) {
                    self.log_error(&"Failed to remove record from client cache: Index out of bounds");
                }
                self.error_msg = None;
            }
            Ok(Err(error)) => self.log_error(&format!("[Server response] failed to remove record: {}", error)),
            Err(_error)    => self.log_error(&format!("Async task failed")),
        }
    }
    pub(super) fn input_reset(&mut self) {
            self.input_username =  String::new();
            self.input_password =  String::new();
            self.description    =  String::new();
            self.day            =  Local::now().date_naive().day();
            self.month          =  Local::now().date_naive().month();
            self.year           =  Local::now().date_naive().year();
            self.value_zl       =  String::new();
            self.value_gr       =  String::new();
    }
}
