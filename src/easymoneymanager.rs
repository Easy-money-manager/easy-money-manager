use crate::record::{ Record, RecordError, ValueError };
use crate::sheet::/*{ Sheet,*/ SheetError;// };
use crate::sheetcollection::{ SheetCollection };
use eframe::egui;

// EasyMoneyManager {{{
//
// description, day, month, year, values - variables to add / update records in sheets
// sheets - array with sheets
// active_sheet - variable with info on which sheet you currently are
//
// Definition {{{
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
    pub active_sheet: usize,
}
// }}}

// Initialization {{{
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
                SheetCollection::create(&"Past".to_string()),
                SheetCollection::create(&"Future".to_string()),
            ],
            active_collection: 0,
            active_sheet: 0,
        };
        for (name, fraction) in [
            ("Incomes", 100),
            ("Essentials", 50),
            ("Stability", 15),
            ("Growth", 25),
            ("Prizes", 10)
        ] {
                def.sheet_collections[0].push(name, fraction);
            }
        for (name, fraction) in [
            ("Incomes", 100),
            ("Expenses", 100),
        ] {
                def.sheet_collections[1].push(name, fraction);
            }
            def
    }
}
// }}}
// }}}

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
}

impl eframe::App for EasyMoneyManager {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame,) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My sheets app");
            ui.separator();

            ui.horizontal(|ui| {

                // Sheet choose and basic stats {{{
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        for (index, sheet_collection) in self.sheet_collections.iter().enumerate() {
                            if ui.button(&sheet_collection.name).clicked() {
                                self.active_collection = index;
                            }
                        }
                    });
                    ui.vertical(|ui| {
                        let sheet_collection = &self.sheet_collections[self.active_collection];
                        for (index, sheet) in sheet_collection.sheets.iter().enumerate() {
                            ui.horizontal(|ui| {
                                if ui.button(&sheet.name).clicked() {
                                    self.active_sheet = index;
                                }
                                if self.active_collection == 0 {
                                    if index == 0 {
                                        ui.label(sheet.sum_display());
                                    } else {
                                        ui.label(sheet.balance_display(&sheet_collection.sheets[0].sum()));
                                    }
                                }
                            });
                        }
                        if self.active_collection == 0 {
                            ui.label(format!("Balance\t{}", self.balance_display()));
                        }
                    });
                });
                let sheet = &mut self.sheet_collections[self.active_collection].sheets[self.active_sheet];

                ui.separator();
                ui.vertical(|ui| {
                    // }}}

                    // Add content to sheet {{{
                    egui::Grid::new("add_record_grid").num_columns(2).spacing([20.0, 8.0]).show(ui, |ui| {

                        // Name {{{
                        ui.label("Name");
                        ui.text_edit_singleline(&mut self.description);
                        ui.end_row();
                        //}}}

                        // Date {{{
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
                        // }}}

                        // Value {{{
                        ui.label("Value\t");
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.value_zl).desired_width(50.0));
                            ui.label(".");
                            ui.add(egui::TextEdit::singleline(&mut self.value_gr).desired_width(15.0));
                        });
                        ui.end_row();
                        // }}}

                    });
                // }}}

                // Add record button {{{
                if ui.button("Add record").clicked() {
                    match Record::from_input(&self.description, &self.year, &self.month, &self.day, &self.value_zl, &self.value_gr) {
                        Ok(record) => {
                            sheet.push(record);
                            self.error_msg = String::new();
                        },
                        Err(error) => {
                            self.error_msg = match error {
                                RecordError::EmptyDescription => "Description can't be empty".to_string(),
                                RecordError::InvalidYear => "Year has to be integer between 1900 and 2200".to_string(),
                                RecordError::InvalidMonth => "Month has to be valid (integer between 1 and 12)".to_string(),
                                RecordError::InvalidDay => "Day has to be valid (integer that satysfies \"day > 0 && (((day < 30 + ((month % 2) ^ (month > 7))) && month != 2) || month == 2 && day < 28 + ((year % 4 == 0 && year % 100 != 0) || year % 400 == 0))\")".to_string(),
                                RecordError::ValueError(ValueError::InvalidValueZl) => "There's some unwanted sign in value field".to_string(),
                                RecordError::ValueError(ValueError::InvalidValueGr) => "There's some unwanted sign in decimal value field".to_string(),
                                RecordError::ValueError(ValueError::TooBigGr)  => "Too big decimal value".to_string(),
                            };
                        },
                    }
                }
                ui.label(&self.error_msg);
                // }}}

                // Display sheet's content {{{
                ui.heading(&sheet.name);
                let mut remove_index = None;

                if !sheet.is_empty() {
                    egui::Grid::new("display_sheet_content_grid")
                        .num_columns(5)
                        .spacing([15.0, 10.0])
                        .show(ui, |ui| {
                            ui.label("Name");
                            ui.label("Date");
                            ui.label("Value");
                            ui.end_row();
                            for index in 0..sheet.len() {
                                let record = &mut sheet.records[index];
                                ui.label(record.description());
                                ui.label(record.date_display());
                                ui.label(record.value_display());

                                // Edit button {{{
                                if ui.button("Edit").clicked() {
                                    record.date_set(&self.year, &self.month, &self.day);
                                    record.description_set(&self.description);
                                    record.value_set(&self.value_zl, &self.value_gr);                                }
                                // }}}

                                // Remove button {{{
                                if ui.button("Remove").clicked() {
                                    remove_index = Some(index);
                                }
                                // }}}
                                ui.end_row();
                            }
                        });
                    if let Some(index) = remove_index {
                        match sheet.remove(index) {
                            Ok(()) => { },
                            Err(SheetError::IndexOutOfBounds) => { ui.label("Something went wrong with removing last record, please contact support for support"); },
                        }
                    }
                }
                // }}}

            });
            });
        });
    }
}
