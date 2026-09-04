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


struct Record {
    description: String,
    date: chrono::NaiveDate,
    value: i64,
}

impl Default for Record {
    fn default() -> Self {
        Self {
            description: String::new(),
            date: Local::now().date_naive(),
            value: 0,
        }
    }
}

impl Clone for Record {
    fn clone(&self) -> Record {
        Record {
            description: self.description.clone(),
            date: self.date.clone(),
            value: self.value.clone(),
        }
    }
}

struct MyApp {
    description: String,
    day: u32,
    month: u32,
    year: i32,
    value_zl: String,
    value_gr: String,
    operational: Record,
    saved: Vec<Record>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            description: String::new(),
            day: 0,
            month: 0,
            year: 0,
            value_zl: String::new(),
            value_gr: String::new(),
            operational: Record::default(),
            saved: Vec::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My sheets app");

            ui.separator();

            if !self.saved.is_empty() {
                for record in &self.saved {
                    ui.horizontal(|ui| {
                        ui.label(record.description.clone());
                        ui.label(record.date.format("%d.%m.%Y").to_string());
                        ui.label((record.value as f64 / 100.0).to_string());
                    });
                }
            }

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Name\t");
                    ui.label("Date [d/m/y]:");
                    ui.label("Value\t");
                });
                ui.vertical(|ui| {
                    ui.text_edit_singleline(&mut self.description);

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
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.value_zl).desired_width(50.0));
                        ui.label(".");
                        ui.add(egui::TextEdit::singleline(&mut self.value_gr).desired_width(15.0));
                    });
                });
            });

            if ui.button("Add record").clicked() {
                self.operational.date = NaiveDate::from_ymd_opt(self.year, self.month, self.day).unwrap();
                self.operational.description = self.description.clone();
                self.operational.value = match self.value_zl.is_empty() {
                    false => self.value_zl.parse::<i64>().unwrap() * 100,
                    true  => 0,
                } + match self.value_gr.is_empty() {
                    false => self.value_gr.parse::<i64>().unwrap(),
                    true  => 0,
                };
                self.saved.push(self.operational.clone());
            }

        });
    }
}
