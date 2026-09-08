use crate::record::Record;
use crate::sheet::SheetError;
use crate::sheetcollection::SheetCollection;
use crate::database::Database;
use eframe::egui;

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

    pub sheet_collections: [SheetCollection; 2],
    pub active_collection: usize,
    pub database: Database,
}

impl Default for EasyMoneyManager {
    fn default() -> Self {
        let mut def: Self = Self {
            description: String::new(),
            day: 0,
            month: 0,
            year: 0,
            value_zl: String::new(),
            value_gr: String::new(),
            error_msg: String::new(),

            sheet_collections: [
                SheetCollection::new(1, &"Main".to_string()),
                SheetCollection::new(2, &"Planning".to_string()),
            ],
            active_collection: 0,
            database: Database::new("easy_money_manager.db").expect("Failed to open database"),
        };
        def.database.initialize().expect("Failed to initialize database");

        for sheet_collection in &mut def.sheet_collections {
            sheet_collection.id_set(def.database.get_or_create_collection(&sheet_collection.name).unwrap());
            match sheet_collection.name.as_str() {
                "Main" => {
                    for (name, fraction) in [
                        ("Incomes", 100),
                        ("Essentials", 50),
                        ("Stability", 15),
                        ("Growth", 25),
                        ("Prizes", 10)
                    ] {
                        sheet_collection.push(
                            def.database.get_or_create_sheet(sheet_collection.id(), &name, fraction).unwrap(),
                            name,
                            fraction,
                        );
                    }
                }
                "Planning" => {
                    for (name, fraction) in [
                        ("Incomes", 100),
                        ("Expenses", 100),
                    ] {
                        sheet_collection.push(
                            def.database.get_or_create_sheet(sheet_collection.id(), &name, fraction).unwrap(),
                            name,
                            fraction,
                        );
                    }
                }
                _ => { },
            }
        }
        for sheet_collection in &mut def.sheet_collections {
            for sheet in &mut sheet_collection.sheets {
                match def.database.get_records(sheet.id()) {
                    Ok(records) => sheet.records = records,
                    Err(error) => eprintln!("Failed to load records for sheet {}:{}", sheet.name, error),
                }
            }
        }
        def
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
        match self.database.create_record(
            self.active_collection().active_sheet().id(),
            &record.description(),
            record.date(),
            record.value()
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

        match self.database.get_record(
            record.id,
            &record.description(),
            record.date(),
            record.value()
        ) {
            Ok(id) => {
                record.id = id;
                match self.active_collection_mut().active_sheet_mut().edit(index, record) {
                    Ok(()) => { },
                    Err(SheetError::IndexOutOfBounds) => self.error_msg = format!("Failed to edit record from cache vector"),
                }
                self.error_msg.clear();
            },
            Err(error) => {
                self.error_msg = format!("Failed to get record: {error}");
            },
        };
    }
    pub fn remove_record(&mut self, index: usize) {
        match self.database.remove_record(self.active_collection().active_sheet().records[index].id()) {
            Ok(()) => match self.active_collection_mut().active_sheet_mut().remove(index) {
                Ok(()) => { },
                Err(SheetError::IndexOutOfBounds) => self.error_msg = format!("Failed to remove record from cache vector"),
            },
            Err(error) => {
                self.error_msg = format!("Failed to remove record: {error}");
            },
        }
    }
}

impl eframe::App for EasyMoneyManager {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame,) {
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
