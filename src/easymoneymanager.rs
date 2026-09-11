use crate::record::Record;
use crate::sheetcollection::SheetCollection;
use crate::sheet::SheetError;
use crate::api::ApiClient;
use eframe::egui;
use crate::requests::{ CreateRecordRequest, UpdateRecordRequest };
use crate::clienttask::/*{*/ ClientTask; //, ClientTaskError };

// EasyMoneyManager
//
// description, day, month, year, values - variables to add / update records in sheets
// sheets - array with sheets
// active_sheet - variable with info on which sheet you currently are


pub struct EasyMoneyManager {
    pub description: String,
    pub day: u32,
    pub month: u32,
    pub year: i32,
    pub value_zl: String,
    pub value_gr: String,
    pub error_msg: String,

    pub sheet_collections: Vec<SheetCollection>,
    pub active_collection: usize,

    pub api_client: ApiClient,
    pub bootstrap_loaded: bool,
    pub bootstrap_task: Option<ClientTask<Result<Vec<SheetCollection>, reqwest::Error>>>,
    pub create_record_task: Option<(
        usize,
        usize,
        Record,
        ClientTask<Result<i64, reqwest::Error>>
    )>,
    pub update_record_task: Option<(
        usize,
        usize,
        usize,
        Record,
        ClientTask<Result<(), reqwest::Error>>,
    )>,
    pub remove_record_task: Option< (
        usize,
        usize,
        usize,
        ClientTask<Result<(), reqwest::Error>>,
    )>,
    #[cfg(not(target_arch = "wasm32"))]
    runtime: tokio::runtime::Runtime,
}
impl Default for EasyMoneyManager {
    fn default() -> Self {
        Self {
            description: String::new(),
            day: 0,
            month: 0,
            year: 0,
            value_zl: String::new(),
            value_gr: String::new(),
            error_msg: String::new(),

            sheet_collections: Vec::new(),
            active_collection: 0,
            api_client: ApiClient::new("http://127.0.0.1:3000".to_string()),
            bootstrap_task: None,
            bootstrap_loaded: false,
            create_record_task: None,
            update_record_task: None,
            remove_record_task: None,
            #[cfg(not(target_arch = "wasm32"))]
            runtime: tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"),
        }
    }
}

impl EasyMoneyManager {
    pub fn balance(&self) -> i64 {
        let mut balance: i64 = self.sheet_collections[self.active_collection].sheets[0].sum();
        for sheet in &self.sheet_collections[self.active_collection].sheets[1..self.sheet_collections[self.active_collection].len()] {
            balance -= sheet.sum();
        }
        balance
    }
    pub fn balance_display(&self) -> String {
        (self.balance() as f64 / 100.0).to_string()
    }

    pub fn active_collection(&self) -> &SheetCollection {
        let collection_index = self.active_collection;
        &self.sheet_collections[collection_index]
    }
    pub fn active_collection_mut(&mut self) -> &mut SheetCollection {
        let collection_index = self.active_collection;
        &mut self.sheet_collections[collection_index]
    }

    pub fn start_bootstrap(&mut self) {
        let api_client = self.api_client.clone();

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.bootstrap_task = Some(
                ClientTask::spawn(
                    &self.runtime,
                    async move {
                        api_client.get_bootstrap().await
                    }
                )
            );
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.bootstrap_task = Some(
                ClientTask::spawn(async move {
                    api_client.get_bootstrap().await
                })
            );
        }
    }
    pub fn add_record(&mut self) {
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
                self.error_msg = error.message().to_string();
                return;
            },
        };
        let collection_index: usize = self.active_collection;
        let sheet_index: usize = self.active_collection().active_sheet_index();
        let api_client = self.api_client.clone();
        let sheet_id = self.active_collection().active_sheet().id();

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
                        async move { api_client.create_record(sheet_id, &request).await }
                    )
            ));
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.create_record_task = Some((
                    collection_index,
                    sheet_index,
                    record,
                    ClientTask::spawn(async move { api_client.create_record(sheet_id, &request).await }
                    )
            ));
        }
    }
    pub fn edit_record(&mut self, index: usize) {
        let mut record = match Record::from_input(
            &self.description,
            &self.year,
            &self.month,
            &self.day,
            &self.value_zl,
            &self.value_gr
        ) {
            Ok(record) => record, 
            Err(error) => {
                self.error_msg = error.message().to_string();
                return;
            },
        };

        let collection_index: usize = self.active_collection;
        let sheet_index: usize = self.active_collection().active_sheet_index();
        record.id_set(self.active_collection().active_sheet().records[index].id());
        let record_id: i64 = record.id();

        let request = UpdateRecordRequest {
            description: record.description().to_string(),
            date: record.date(),
            value: record.value(),
        };
        let api_client = self.api_client.clone();

        self.update_record_task = Some((
            collection_index,
            sheet_index,
            index,
            record,
            self.runtime.spawn(async move { api_client.update_record(record_id, &request).await } )
        ));
    }
    pub fn remove_record(&mut self, index: usize) {
        let api_client = self.api_client.clone();
        let collection_index: usize = self.active_collection;
        let sheet_index: usize = self.active_collection().active_sheet_index();
        let record_id = self.active_collection().active_sheet().records[index].id();

        self.remove_record_task = Some((
            collection_index,
            sheet_index,
            index,
            self.runtime.spawn(async move { api_client.remove_record(record_id).await } )
        ));
    }
}

impl eframe::App for EasyMoneyManager {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame,) {

        if !self.bootstrap_loaded && self.bootstrap_task.is_none() {
            self.start_bootstrap();
        }

        let bootstrap_finished = self.bootstrap_task.as_ref().is_some_and(|task| task.is_finished());
        if bootstrap_finished {
            let task = self.bootstrap_task.take().expect("bootstrap_task should exist when it's mared as finished");
            #[cfg(not(target_arch = "wasm32"))]
            let result = task.take(&self.runtime);
            #[cfg(target_arch = "wasm32")]
            let result = task.take();

            match result {
                Ok(Ok(collections)) => {
                    self.sheet_collections = collections;
                    self.bootstrap_loaded = true;
                    self.error_msg.clear();
                }
                Ok(Err(error)) => self.error_msg = format!("Failed to load application: {}", error),
                Err(error)     => self.error_msg = format!("Async task failed: {}", error),
            }
        }

        if !self.bootstrap_loaded {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Waiting for server to bootstrap data");
                ui.label(&self.error_msg);
            });
            return;
        }

        let create_finished = self.create_record_task.as_ref().is_some_and(|(_, _, _, task)| task.is_finished());
        if create_finished {
            let (collection_index, sheet_index, mut record, task) = self.create_record_task.take().expect("create_record_task should exist when it's mared as finished");
            #[cfg(not(target_arch = "wasm32"))]
            let result = task.take(&self.runtime);
            #[cfg(target_arch = "wasm32")]
            let result = task.take();

            match result {
                Ok(Ok(id)) => {
                    record.id_set(id);
                    self.sheet_collections[collection_index].sheets[sheet_index].push(record);
                    self.error_msg.clear();
                }
                Ok(Err(error)) => self.error_msg = format!("[Server response] failed to create record: {}", error),
                Err(error)     => self.error_msg = format!("Async task failed: {}", error),
            }
        }

        let update_finished = self.update_record_task.as_ref().is_some_and(|(_, _, _, _, task)| task.is_finished());
        if update_finished {
            let (collection_index, sheet_index, index, record, task) = self.update_record_task.take().expect("update_record_task should exist when it's mared as finished");

            match self.runtime.block_on(task) {
                Ok(Ok(()))     => if let Err(SheetError::IndexOutOfBounds) = self.sheet_collections[collection_index].sheets[sheet_index].edit(index, record) {
                    eprintln!("Failed to edit record in client cache: Index out of bounds");
                },
                Ok(Err(error)) => self.error_msg = format!("[Server response] failed to edit record: {}", error),
                Err(error)     => self.error_msg = format!("Async task failed: {}", error),
            }
        }

        let remove_finished = self.remove_record_task.as_ref().is_some_and(|(_, _, _, task)| task.is_finished());
        if remove_finished {
            let (collection_index, sheet_index, index, task) = self.remove_record_task.take().expect("remove_task should exist when it's mared as finished");

            match self.runtime.block_on(task) {
                Ok(Ok(()))     => if let Err(SheetError::IndexOutOfBounds) = self.sheet_collections[collection_index].sheets[sheet_index].remove(index) {
                    eprintln!("Failed to remove record from client cache: Index out of bounds");
                },
                Ok(Err(error)) => self.error_msg = format!("[Server response] failed to remove record: {}", error),
                Err(error)     => self.error_msg = format!("Async task failed: {}", error),
            }
        }


        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My sheets app");
            ui.separator();

            ui.horizontal(|ui| {

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        for (index, sheet_collection) in self.sheet_collections.iter().enumerate() {
                            if ui.button(&sheet_collection.name).clicked() {
                                self.active_collection = index;
                            }
                        }
                    });
                    ui.vertical(|ui| {
                        let mut selected_sheet: Option<usize> = None;
                        for (index, sheet) in self.active_collection().sheets.iter().enumerate() {
                            ui.horizontal(|ui| {
                                if ui.button(&sheet.name).clicked() {
                                    selected_sheet = Some(index);
                                }
                                if self.active_collection == 0 {
                                    if index == 0 {
                                        ui.label(sheet.sum_display());
                                    } else {
                                        ui.label(sheet.balance_display(&self.sheet_collections[0].sheets[0].sum()));
                                    }
                                }
                            });
                        }
                        if let Some(index) = selected_sheet {
                            self.active_collection_mut().active_sheet_set(index);
                        }
                        if self.active_collection == 0 {
                            ui.label(format!("Balance\t{}", self.balance_display()));
                        }
                    });
                });

                ui.separator();
                ui.vertical(|ui| {

                    egui::Grid::new("add_record_grid").num_columns(2).spacing([20.0, 8.0]).show(ui, |ui| {

                        ui.label("Name");
                        ui.text_edit_singleline(&mut self.description);
                        ui.end_row();

                        ui.label("Date [d/m/y]:");
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::DragValue::new(&mut self.day)
                                .range(1..=31)
                            );
                            ui.label("/");
                            ui.add(
                                egui::DragValue::new(&mut self.month)
                                .range(1..=12)
                            );
                            ui.label("/");
                            ui.add(
                                egui::DragValue::new(&mut self.year)
                                .range(1900..=2100)
                            );
                        });
                        ui.end_row();

                        ui.label("Value\t");
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.value_zl).desired_width(50.0));
                            ui.label(".");
                            ui.add(egui::TextEdit::singleline(&mut self.value_gr).desired_width(15.0));
                        });
                        ui.end_row();

                    });

                    if ui.button("Add record").clicked() {
                        self.add_record();
                    }
                    ui.label(&self.error_msg);

                    ui.heading(&self.active_collection().active_sheet().name);
                    let mut remove_index = None;

                    if !self.active_collection().active_sheet().is_empty() {
                        egui::Grid::new("display_sheet_content_grid")
                            .num_columns(5)
                            .spacing([15.0, 10.0])
                            .show(ui, |ui| {
                                ui.label("Name");
                                ui.label("Date");
                                ui.label("Value");
                                ui.end_row();
                                for index in 0..self.active_collection().active_sheet().len() {
                                    let record = &mut self.active_collection_mut().active_sheet_mut().records[index];

                                    ui.label(record.description());
                                    ui.label(record.date_display());
                                    ui.label(record.value_display());

                                    if ui.button("Edit").clicked() {
                                        self.edit_record(index);
                                    }

                                    if ui.button("Remove").clicked() {
                                        remove_index = Some(index);
                                    }
                                    ui.end_row();
                                }
                        });
                        if let Some(index) = remove_index {
                            self.remove_record(index)
                        }
                    }
                });
            });
        });
    }
}
