use super::EasyMoneyManager;
use eframe::egui;
use emm_shared::sheet::RecordSorting;

impl EasyMoneyManager {
    pub(super) fn main_ui(&mut self, ui: &mut egui::Ui) {

        self.handle_bootstrap_task();
        if !self.bootstrap_loaded {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.heading("Waiting for server to bootstrap data");
                if let Some(error) = &self.error_msg {
                    ui.label(error);
                }
            });
            return;
        }

        self.handle_create_task();
        self.handle_edit_task();
        self.handle_remove_task();
        egui::CentralPanel::default().frame(egui::Frame::new().fill(egui::Color32::TRANSPARENT)).show(ui, |ui| {
            let available_width: f32 = ui.available_width();
            ui.horizontal(|ui| {
                let heading_width: f32 = 200.0;
                let button_width: f32 = 80.0;
                ui.allocate_space(egui::vec2((available_width - heading_width) / 2.0 - button_width, 0.0));
                ui.add_sized(
                    [heading_width, 30.0],
                    egui::Label::new(egui::RichText::new("Easy money manager").heading()).halign(egui::Align::Center),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let username: String = self.username().expect("Logged in user should have name").to_owned();
                    ui.menu_button(username, |ui| {
                        if ui.button("Log out").clicked() {
                            self.logout();
                            ui.close();
                        }
                        if ui.button("Remove account").clicked() {
                            self.remove_account();
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if ui.button("Quit").clicked() {
                                self.quit();
                                ui.close();
                            }
                        }
                    });
                });
            });

            ui.separator();

            ui.horizontal_top(|ui| {

                egui::Grid::new("collections_grid").num_columns(2).spacing([100.0, 15.0]).show(ui, |ui| {
                    for (index, sheet_collection) in self.sheet_collections.iter().enumerate() {
                        if ui.button(&sheet_collection.name).clicked() {
                            self.active_collection = index;
                        }
                        ui.label(sheet_collection.balance_display());
                        ui.end_row();
                    }
                });
                ui.separator();
                let mut selected_sheet: Option<usize> = None;
                egui::Grid::new("sheets_grid").num_columns(2).spacing([100.0, 15.0]).show(ui, |ui| {
                    for (index, sheet) in self.active_collection().sheets.iter().enumerate() {
                        if ui.button(&sheet.name).clicked() {
                            selected_sheet = Some(index);
                        }
                        if index == 0 || self.active_collection == 1 {
                            ui.label(sheet.sum_display());
                        } else {
                            ui.label(sheet.balance_display(&self.sheet_collections[0].sheets[0].sum()));
                        }
                        ui.end_row();
                    }
                    if let Some(index) = selected_sheet {
                        self.active_collection_mut().active_sheet_set(index);
                    }
                    ui.label("Balance");
                    ui.label(self.balance_display());
                    ui.end_row();
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
                            ui.add(egui::TextEdit::singleline(&mut self.value_zl).desired_width(100.0));
                            ui.label(".");
                            ui.add(egui::TextEdit::singleline(&mut self.value_gr).desired_width(30.0));
                        });
                        ui.end_row();

                    });

                    if ui.button("Add record").clicked() {
                        self.add_record();
                    }
                    if let Some(error) = &self.error_msg {
                        ui.label(error);
                    }

                    ui.heading(&self.active_collection().active_sheet().name);
                    let mut remove_index = None;

                    if !self.active_collection().active_sheet().is_empty() {
                        egui::Grid::new("display_sheet_content_grid")
                            .num_columns(5)
                            .spacing([15.0, 10.0])
                            .show(ui, |ui| {
                                if ui.button("Name").clicked() {
                                    self.record_sorting = match self.record_sorting {
                                    RecordSorting::DescriptionAscending => RecordSorting::DescriptionDescending,
                                    _                                   => RecordSorting::DescriptionAscending,
                                    };
                                }
                                if ui.button("Date").clicked() {
                                    self.record_sorting = match self.record_sorting {
                                    RecordSorting::DateDescending => RecordSorting::DateAscending,
                                    _                             => RecordSorting::DateDescending,
                                    };
                                }
                                if ui.button("Value").clicked() {
                                    self.record_sorting = match self.record_sorting {
                                    RecordSorting::ValueDescending => RecordSorting::ValueAscending,
                                    _                              => RecordSorting::ValueDescending,
                                    };
                                }
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
        let sorting = self.record_sorting;
        self.active_collection_mut().active_sheet_mut().records_sort(sorting);
    }
    pub(super) fn login_ui(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().frame(egui::Frame::new().fill(egui::Color32::TRANSPARENT)).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);

                ui.heading(
                    egui::RichText::new("Easy Money Manager")
                    .size(32.0)
                );

                ui.add_space(8.0);

                ui.label("Manage your money clearly and simply.");

                ui.add_space(30.0);

                egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::same(20))
                    .show(ui, |ui| {
                        ui.set_width(320.0);

                        ui.label("Username");
                        ui.add(egui::TextEdit::singleline(&mut self.username_input).id(egui::Id::new("login_username")));

                        ui.add_space(10.0);

                        ui.label("Password");
                        #[cfg(debug_assertions)]
                        {
                            ui.add(egui::TextEdit::singleline(&mut self.password_input).password(false).id(egui::Id::new("login_password")));
                            ui.label(format!("chars={} bytes={}", self.password_input.chars().count(), self.password_input.len()));
                        }
                        #[cfg(not(debug_assertions))]
                        ui.add(egui::TextEdit::singleline(&mut self.password_input).password(true).id(egui::Id::new("login_password")));

                        ui.add_space(16.0);

                        if ui.button("Log in").clicked() {
                            self.login();
                        }

                        if ui.button("Register").clicked() {
                            self.register();
                        }

                        if let Some(message) = &self.auth_error {
                            ui.add_space(10.0);
                            ui.label(message);
                        }
                    });
            });
        });
    }
    pub(super) fn logging_ui(&mut self, ui: &mut egui::Ui) {
        self.handle_login_task();
        egui::CentralPanel::default().frame(egui::Frame::new().fill(egui::Color32::TRANSPARENT)).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.spinner();
                ui.label("Logging in...");
            });
        });
    }
    pub(super) fn unlogging_ui(&mut self, ui: &mut egui::Ui) {
        self.handle_logout_task(ui.ctx());
        self.handle_remove_account_task();
        egui::CentralPanel::default().frame(egui::Frame::new().fill(egui::Color32::TRANSPARENT)).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.spinner();
                ui.label("Logging out...");
            });
        });
    }
    pub(super) fn registering_ui(&mut self, ui: &mut egui::Ui) {
        self.handle_register_task();
        egui::CentralPanel::default().frame(egui::Frame::new().fill(egui::Color32::TRANSPARENT)).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.spinner();
                ui.label("Registering...")
            });
        });
    }
}
