use crate::record::{Record, RecordError, ValueError};
use crate::sheet::Sheet;
use eframe::egui;

// App {{{
//
// description, day, month, year, values - variables to add / update records in sheets
// sheets - array with sheets
// active_sheet - variable with info on which sheet you currently are
//
// Definition {{{
pub struct MyApp {
    pub description: String,
    pub day: u32,
    pub month: u32,
    pub year: i32,
    pub value_zl: String,
    pub value_gr: String,
    pub error_msg: String,

    pub sheets: [Sheet; 5],
    pub active_sheet: usize,
}
// }}}

// Initialization {{{
impl Default for MyApp {
    fn default() -> Self {
        Self {
            description: String::new(),
            day: 0,
            month: 0,
            year: 0,
            value_zl: String::new(),
            value_gr: String::new(),
            error_msg: String::new(),

            sheets: [
                Sheet {
                    name: "Incomes".to_string(),
                    records: Vec::new(),
                    fraction: 100,
                },
                Sheet {
					name: "Essentials".to_string(),
					records: Vec::new(),
                    fraction: 50,
				},
                Sheet {
					name: "Stability".to_string(),
					records: Vec::new(),
                    fraction: 15,
				},
                Sheet {
					name: "Growth".to_string(),
					records: Vec::new(),
                    fraction: 25,
				},
                Sheet {
					name: "Prizes".to_string(),
					records: Vec::new(),
                    fraction: 10,
				},
            ],
            active_sheet: 0,
        }
    }
}
// }}}
// }}}

impl MyApp {
    pub fn balance(&self) -> i64 {
        let mut balance: i64 = self.sheets[0].sum();
        for sheet in &self.sheets[1..=4] {
            balance -= sheet.sum();
        }
        balance
    }
    pub fn balance_display(&self) -> String {
        (self.balance() as f64 / 100.0).to_string()
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame,) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My sheets app");
            ui.separator();

            ui.horizontal(|ui| {

                // Sheet choose and basic stats {{{
                ui.vertical(|ui| {
                    for (index, sheet) in self.sheets.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.button(&sheet.name).clicked() {
                                self.active_sheet = index;
                            }
                            if 0 == index {
                                ui.label(sheet.sum_display());
                            } else {
                                ui.label(sheet.balance_display(&self.sheets[0].sum()));
                            }
                        });
                    }
                    ui.label(format!("Balance\t{}", self.balance_display()));
                });

                let sheet = &mut self.sheets[self.active_sheet];

                ui.separator();
                ui.vertical(|ui| {
                // }}}

                    // Add content to sheet {{{
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Name\t");
                            ui.label("Date [d/m/y]:");
                            ui.label("Value\t");
                        });
                        ui.vertical(|ui| {
                            ui.text_edit_singleline(&mut self.description);

                            // Date {{{
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
                            // }}}

                            // Value {{{
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.value_zl).desired_width(50.0));
                                ui.label(".");
                                ui.add(egui::TextEdit::singleline(&mut self.value_gr).desired_width(15.0));
                            });
                            // }}}

                        });
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
                                    RecordError::ValueError(ValueError::InvalidZl) => "There's some unwanted sign in value field".to_string(),
                                    RecordError::ValueError(ValueError::InvalidGr) => "There's some unwanted sign in decimal value field".to_string(),
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
                        for index in 0..sheet.len() {
                            let record = &mut sheet.records[index];
                            ui.horizontal(|ui| {
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
                            });
                        }
                        if let Some(index) = remove_index {
                            sheet.remove(index);
                        }
                    }
                    // }}}

                });
            });
        });
    }
}
