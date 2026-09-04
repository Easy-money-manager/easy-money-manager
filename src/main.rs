use eframe::egui;
use chrono::NaiveDate;
use chrono::Local;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Sheets",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

// Record {{{
// Definition {{{
struct Record {
    description: String,
    date: chrono::NaiveDate,
    value: i64,
}
// }}}
// Initialization {{{
impl Default for Record {
    fn default() -> Self {
        Self {
            description: String::new(),
            date: Local::now().date_naive(),
            value: 0,
        }
    }
}
// }}}
// Clone {{{
impl Clone for Record {
    fn clone(&self) -> Record {
        Record {
            description: self.description.clone(),
            date: self.date.clone(),
            value: self.value.clone(),
        }
    }
}
// }}}
// Display Value {{{
impl Record {
    fn value_display(&self) -> String {
        (self.value as f64 / 100.0).to_string()
    }
}
// }}}
// }}}
// Sheet {{{
// Definition {{{
struct Sheet {
    name: String,
    records: Vec<Record>,
    fraction: i64,
}
// }}}
// Initialization {{{
impl Default for Sheet {
    fn default() -> Self {
        Self {
            name: String::new(),
            records: Vec::new(),
            fraction: 0,
        }
    }
}
// }}}
// Sum{{{
impl Sheet {
    fn sum(&self) -> i64 {
        let mut sum = 0;
        for record in &self.records {
            sum += record.value;
        }
        sum
    }
    fn sum_display(&self) -> String {
        (self.sum() as f64 / 100.0).to_string()
    }
}
// }}}
// }}}
// App {{{
// Definition {{{
struct MyApp {
    description: String,
    day: u32,
    month: u32,
    year: i32,
    value_zl: String,
    value_gr: String,

    sheets: [Sheet; 5],
    active_sheet: usize,
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

impl eframe::App for MyApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My sheets app");
            ui.separator();

            ui.horizontal(|ui| {

                // Sheet choose {{{
                ui.vertical(|ui| {
                    for (index, sheet) in self.sheets.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.button(&sheet.name).clicked() {
                                self.active_sheet = index;
                            }
                            if 0 == index {
                                ui.label(sheet.sum_display());
                            } else {
                                ui.label(((self.sheets[0].sum() * sheet.fraction / 100 - sheet.sum()) as f64 / 100.0).to_string());
                            }
                        });
                    }
                });

                let sheet = &mut self.sheets[self.active_sheet];

                ui.separator();
                ui.vertical(|ui| {
                // }}}

                    // Display sheet's content {{{
                    let mut remove_index = None;

                    if !sheet.records.is_empty() {
                        for index in 0..sheet.records.len() {
                            let record = &mut sheet.records[index];
                            ui.horizontal(|ui| {
                                ui.label(&record.description);
                                ui.label(record.date.format("%d.%m.%Y").to_string());
                                ui.label(record.value_display());

                                // Edit button {{{
                                if ui.button("Edit").clicked() {
                                    record.date = NaiveDate::from_ymd_opt(self.year, self.month, self.day).unwrap();
                                    record.description = self.description.clone();
                                    record.value = match self.value_zl.is_empty() {
                                        false => (match self.value_zl.parse::<i64>() {
                                            Ok(value) => value,
                                            Err(_)    => 0,
                                        }) * 100,
                                        true  => 0,
                                    } + match self.value_gr.is_empty() {
                                        false => match self.value_gr.parse::<i64>() {
                                            Ok(value) => value,
                                            Err(_)    => 0,
                                        },
                                        true  => 0,
                                    };
                                }
                                // }}}

                                // Remove button {{{
                                if ui.button("Remove").clicked() {
                                    remove_index = Some(index);
                                }
                                // }}}
                            });
                        }
                        if let Some(index) = remove_index {
                            sheet.records.remove(index);
                        }
                    }
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
                        let record = Record {
                            date: NaiveDate::from_ymd_opt(self.year, self.month, self.day).unwrap(),
                            description: self.description.clone(),
                            value: match self.value_zl.is_empty() {
                                false => (match self.value_zl.parse::<i64>() {
                                    Ok(value) => value,
                                    Err(_)    => 0,
                                }) * 100,
                                true  => 0,
                            } + match self.value_gr.is_empty() {
                                false => match self.value_gr.parse::<i64>() {
                                    Ok(value) => value,
                                    Err(_)    => 0,
                                },
                                true  => 0,
                            }
                        };
                        sheet.records.push(record);
                    }
                    // }}}

                });
            });
        });
    }
}
