use emm_shared::record::Record;
use emm_shared::sheetcollection::SheetCollection;
use emm_shared::sheet::SheetError;
use crate::api::ApiClient;
use eframe::egui;
use emm_shared::request::{ RegisterRequest, LoginRequest, CreateRecordRequest, UpdateRecordRequest };
use emm_shared::response::{ LoginResponse };
use crate::clienttask::/*{*/ ClientTask; //, ClientTaskError };

// EasyMoneyManager
//
// description, day, month, year, values - variables to add / update records in sheets
// sheets - array with sheets
// active_sheet - variable with info on which sheet you currently are

pub struct UserSession {
    pub user_id: i64,
    pub username: String,
    pub session_token: String,
}

pub enum AuthState {
    LoggedOut,
    LoggingIn,
    LoggedIn(UserSession),
}

pub struct EasyMoneyManager {
    pub username_input: String,
    pub password_input: String,
    pub description: String,
    pub day: u32,
    pub month: u32,
    pub year: i32,
    pub value_zl: String,
    pub value_gr: String,
    pub error_msg: Option<String>,
    pub auth_error: Option<String>,

    pub sheet_collections: Vec<SheetCollection>,
    pub active_collection: usize,

    pub api_client: ApiClient,
    pub auth_state: AuthState,
    pub login_task: Option<ClientTask<Result<LoginResponse, reqwest::Error>>>,
    pub register_task: Option<ClientTask<Result<(), reqwest::Error>>>,
    pub logout_task: Option<ClientTask<Result<(), reqwest::Error>>>,
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
            username_input: String::new(),
            password_input: String::new(),

            description: String::new(),
            day: 0,
            month: 0,
            year: 0,
            value_zl: String::new(),
            value_gr: String::new(),
            error_msg: None,
            auth_error: None,

            sheet_collections: Vec::new(),
            active_collection: 0,
            api_client: ApiClient::new("http://127.0.0.1:3000".to_string()),
            auth_state: AuthState::LoggedOut,
            register_task: None,
            login_task: None,
            logout_task: None,
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
    fn session_token(&self) -> Option<&str> {
        match &self.auth_state {
            AuthState::LoggedIn(session) => Some(&session.session_token),
            _ => None,
        }
    }
    fn session_token_clone(&self) -> Option<String> {
        match &self.auth_state {
            AuthState::LoggedIn(session) => Some(session.session_token.clone()),
            _ => None,
        }
    }

    // Auth tasks{{{
    fn register(&mut self) {
        let request: RegisterRequest = RegisterRequest {
            username: self.username_input.clone(),
            password: self.password_input.clone(),
        };
        let api_client: ApiClient = self.api_client.clone();

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.register_task = Some(
                ClientTask::spawn(
                    &self.runtime,
                    async move {
                        api_client.register(&request).await
                    }
                )
            );
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.register_task = Some(
                ClientTask::spawn(
                    async move {
                        api_client.register(&request).await
                    }
                )
            );
        }
    }
    fn login(&mut self) {
        let request: LoginRequest = LoginRequest {
            username: self.username_input.clone(),
            password: self.password_input.clone(),
        };
        let api_client: ApiClient = self.api_client.clone();

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.login_task = Some(
                ClientTask::spawn(
                    &self.runtime,
                    async move {
                        api_client.login(&request).await
                    }
                )
            );
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.login_task = Some(
                ClientTask::spawn(
                    async move {
                        api_client.login(&request).await
                    }
                )
            );
        }
        self.auth_state = AuthState::LoggingIn;
    }
    fn logout(&mut self) {
        let api_client: ApiClient = self.api_client.clone();
        let session_token: String = match self.session_token_clone() {
            Some(session_token) => session_token,
            None => panic!(),
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.logout_task = Some(
                ClientTask::spawn(
                    &self.runtime,
                    async move {
                        api_client.logout(&session_token).await
                    }
                )
            );
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.logout_task = Some(
                async move {
                    api_client.logout(&session_token).await
                }
            );
        }
    }

    fn handle_register_task(&mut self) {
        let finished: bool = match &self.register_task {
            Some(task) => task.is_finished(),
            None       => false,
        };
        if !finished {
            return;
        }
        let task = match self.register_task.take() {
            Some(task) => task,
            None       => return,
        };
        #[cfg(not(target_arch = "wasm32"))]
        let result = task.take(&self.runtime);
        #[cfg(target_arch = "wasm32")]
        let result = task.take();
        match result {
            Ok(Ok(()))     => self.auth_error = Some("Account created succesfully, you may log in now!".to_string()),
            Ok(Err(error)) => {
                self.auth_error = Some("Registration failed, probably name already in use\nPossibly server may be down".to_string());
                self.log_error(&format!("Registration failed: {}", error));
            }
            Err(error)     => {
                self.auth_error = Some("Registration failed die to server issue, sorry!".to_string());
                self.log_error(&format!("Registration task failed: {}", error));
            }
        };
    }
    fn handle_login_task(&mut self) {
        let finished: bool = match &self.login_task {
            Some(task) => task.is_finished(),
            None       => false,
        };
        if !finished {
            return;
        }
        let task = match self.login_task.take() {
            Some(task) => task,
            None       => return,
        };
        #[cfg(not(target_arch = "wasm32"))]
        let result = task.take(&self.runtime);
        #[cfg(target_arch = "wasm32")]
        let result = task.take();
        match result {
            Ok(Ok(response)) => {
                self.auth_state = AuthState::LoggedIn (
                    UserSession {
                        user_id: response.user_id,
                        username: response.username,
                        session_token: response.session_token,
                    }
                );
                self.auth_error = None;
                self.start_bootstrap();
            }
            Ok(Err(error)) => {
                self.auth_state = AuthState::LoggedOut;
                self.auth_error = Some("Username or password is incorrect\nPossibly server may be down".to_string());
                self.log_error(&format!("Login failed: {}", error))
            }
            Err(error)     => {
                self.auth_state = AuthState::LoggedOut;
                self.auth_error = Some("Login failed due to server issue, sorry!".to_string());
                self.log_error(&format!("Login task failed: {}", error))
            }
        }
    }
    fn handle_logout_task(&mut self) {
        let finished: bool = match &self.logout_task {
            Some(task) => task.is_finished(),
            None       => false,
        };
        if !finished {
            return;
        }
        let task = match self.logout_task.take() {
            Some(task) => task,
            None       => return,
        };
        #[cfg(not(target_arch = "wasm32"))]
        let result = task.take(&self.runtime);
        #[cfg(target_arch = "wasm32")]
        let result = task.take();
        match result {
            Ok(Ok(())) => {
                self.auth_state = AuthState::LoggedOut;
                self.log(&"Logged out successfully".to_string());
                self.auth_error = Some("Logged out successfully".to_string());
            }
            Ok(Err(error)) => {
                self.log(&format!("Couldn't log out: {}", error));
            }
            Err(error) => {
                self.log_error(&format!("Couldn't log out: {}", error));
            }
        }
    }
    // }}}

    // Content tasks {{{
    pub fn start_bootstrap(&mut self) {
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
    }
    pub fn edit_record(&mut self, index: usize) {
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
    }
    pub fn remove_record(&mut self, index: usize) {
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
    fn handle_bootstrap_task(&mut self) {
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
    fn handle_create_task(&mut self) {
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
    fn handle_edit_task(&mut self) {
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
    fn handle_remove_task(&mut self) {
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
    // }}}

    fn log(&mut self, message: &str) {
        println!("[EMM LOG]: {}", message);
        self.error_msg = Some(message.to_string());
    }
    fn log_error(&mut self, message: &str) {
        eprintln!("[EMM ERROR]: {}", message);
        self.error_msg = Some(message.to_string());
    }

    fn main_ui(&mut self, ctx: &egui::Context) {

        self.handle_bootstrap_task();
        if !self.bootstrap_loaded {
            egui::CentralPanel::default().show(ctx, |ui| {
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
        self.handle_logout_task();
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_width: f32 = ui.available_width();
            ui.horizontal(|ui| {
                let heading_width: f32 = 200.0;
                let button_width: f32 = 80.0;
                ui.allocate_space(egui::vec2((available_width - heading_width) / 2.0 - button_width, 0.0));
                ui.add_sized(
                    [heading_width, 30.0],
                    egui::Label::new(egui::RichText::new("Easy Money Manager").heading()).halign(egui::Align::Center),
                );
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("Log Out").clicked() {
                            self.logout();
                        }
                    }
                );
            });

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
                                if index == 0 || self.active_collection == 1 {
                                    ui.label(sheet.sum_display());
                                } else {
                                    ui.label(sheet.balance_display(&self.sheet_collections[0].sheets[0].sum()));
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
    fn login_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.handle_register_task();
            ui.vertical_centered(|ui| {
                ui.heading("Easy Money Manager");
                ui.add_space(20.0);

                ui.label("Username");
                ui.text_edit_singleline(&mut self.username_input);

                ui.label("Password");
                ui.add(egui::TextEdit::singleline(&mut self.password_input).password(true));

                if ui.button("Login").clicked() {
                    self.login();
                }

                if ui.button("Register").clicked() {
                    self.register();
                }

                if let Some(msg) = &self.auth_error {
                    ui.label(msg);
                }
            });
        });
    }
    fn logging_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.handle_login_task();
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.spinner();
                ui.label("Logging in...");
            });
        });
    }
}

impl eframe::App for EasyMoneyManager {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame,) {
        match self.auth_state {
            AuthState::LoggedOut   => self.login_ui(ctx),
            AuthState::LoggingIn   => self.logging_ui(ctx),
            AuthState::LoggedIn(_) => self.main_ui(ctx),
        }
    }
}
