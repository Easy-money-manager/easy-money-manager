use crate::record::Record;
use crate::sheet::SheetError;
use crate::sheetcollection::SheetCollection;
use crate::api::ApiClient;
use eframe::egui;
use crate::requests::{ CreateRecordRequest, UpdateRecordRequest };
use poll_promise::Promise;

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
    pub bootstrap_promise: Option<Promise<Vec<SheetCollection>, reqwest::Error>>,
    create_record_promise:,
    update_record_promise:,
    remove_record_promise:,
    todo!();
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
            bootstrap_promise: None,
            bootstrap_loaded: false,
        }
    }
}

impl EasyMoneyManager {
    pub fn load_bootstrap(&mut self) {
        let api_client = self.api_client.clone();

        self.bootstrap_promise = Some(
            Promise::spawn_async(async move {
                api_client.get_bootstrap().await
            })
        );
    }
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

    pub fn add_record(&mut self) {
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
        let request: CreateRecordRequest = CreateRecordRequest {
            description: record.description().to_string(),
            date: record.date(),
            value: record.value(),
        };
        match self.api_client.create_record(
            self.active_collection().active_sheet().id(),
            &request
        ) {
            Ok(id) => {
                record.id = id;
                self.active_collection_mut().active_sheet_mut().push(record);
                self.error_msg.clear();
            },
            Err(error) => {
                self.error_msg = format!("Failed to save record: {error}");
            },
        };
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

        record.id_set(self.active_collection().active_sheet().records[index].id());

        let request = UpdateRecordRequest {
            description: record.description().to_string(),
            date: record.date(),
            value: record.value(),
        };

        match self.api_client.update_record(
            record.id(),
            &request
        ) {
            Ok(()) => {
                match self.active_collection_mut().active_sheet_mut().edit(index, record) {
                    Ok(()) => { },
                    Err(SheetError::IndexOutOfBounds) => self.error_msg = format!("Failed to edit record in cache vector"),
                }
                self.error_msg.clear();
            },
            Err(error) => {
                self.error_msg = format!("Failed to get record: {error}");
            },
        };
    }
    pub fn remove_record(&mut self, index: usize) {
        match self.api_client.remove_record(self.active_collection().active_sheet().records[index].id()) {
            Ok(()) => match self.active_collection_mut().active_sheet_mut().remove(index) {
                Ok(()) => { },
                Err(SheetError::IndexOutOfBounds) => self.error_msg = format!("Failed to remove record in cache vector"),
            },
            Err(error) => {
                self.error_msg = format!("Failed to remove record: {error}");
            },
        }
    }
}

impl eframe::App for EasyMoneyManager {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame,) {

        if !self.bootstrap_loaded && self.bootstrap_promise.is_none() {
            self.load_bootstrap();
        }

        let bootstrap_finished = self
            .bootstrap_promise
            .as_ref()
            .is_some_and(|promise| promise.ready().is_some());

        if bootstrap_finished {
            let promise = self.bootstrap_promise.take().unwrap();

            match promise.block_and_take() {
                Ok(collections) => {
                    self.sheet_collections = collections;
                    self.bootstrap_loaded = true;
                    self.error_msg.clear();
                }

                Err(error) => {
                    self.error_msg =
                        format!("Failed to load application: {error}");
                }
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
