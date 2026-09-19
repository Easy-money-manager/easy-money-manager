use emm_shared::record::Record;
use emm_shared::sheetcollection::SheetCollection;
use emm_shared::sheet::{ SheetError, RecordSorting };
use crate::api::ApiClient;
use eframe::egui;
use crate::clienttask::ClientTask;
use emm_shared::response::LoginResponse;
use unicode_normalization::UnicodeNormalization;
use crate::easymoneymanager::auth::AuthState;


pub struct EasyMoneyManager {
    pub(super) input_username: String,
    pub(super) input_password: String,
    pub(super) description: String,
    pub(super) day: u32,
    pub(super) month: u32,
    pub(super) year: i32,
    pub(super) value_zl: String,
    pub(super) value_gr: String,
    pub(super) record_edited: i64,

    pub(super) error_msg: Option<String>,
    pub(super) auth_error: Option<String>,

    pub(super) quit_after_logout: bool,
    pub(super) api_client: ApiClient,
    pub(super) auth_state: AuthState,
    pub(super) login_task: Option<ClientTask<Result<LoginResponse, reqwest::Error>>>,
    pub(super) register_task: Option<ClientTask<Result<(), reqwest::Error>>>,
    pub(super) logout_task: Option<ClientTask<Result<(), reqwest::Error>>>,
    pub(super) remove_account_task: Option<ClientTask<Result<(), reqwest::Error>>>,
    pub(super) bootstrap_loaded: bool,
    pub(super) bootstrap_task: Option<ClientTask<Result<Vec<SheetCollection>, reqwest::Error>>>,
    pub(super) create_record_task: Option<(
        usize,
        usize,
        Record,
        ClientTask<Result<i64, reqwest::Error>>
    )>,
    pub(super) update_record_task: Option<(
        usize,
        usize,
        usize,
        Record,
        ClientTask<Result<(), reqwest::Error>>,
    )>,
    pub(super) remove_record_task: Option< (
        usize,
        usize,
        usize,
        ClientTask<Result<(), reqwest::Error>>,
    )>,
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) runtime: tokio::runtime::Runtime,

    pub(super) record_sorting: RecordSorting,
    pub(super) show_remove_account_popup: bool,
    pub(super) sheet_collections: Vec<SheetCollection>,
    pub(super) active_collection: usize,

}
impl Default for EasyMoneyManager {
    fn default() -> Self {
        Self {
            input_username: String::new(),
            input_password: String::new(),

            description: String::new(),
            day: 1,
            month: 1,
            year: 1900,
            value_zl: String::new(),
            value_gr: String::new(),
            record_edited: 0,

            error_msg: None,
            auth_error: None,
            quit_after_logout: false,
            show_remove_account_popup: false,

            record_sorting: RecordSorting::DateDescending,
            sheet_collections: Vec::new(),
            active_collection: 0,
            api_client: ApiClient::new(),
            auth_state: AuthState::LoggedOut,
            register_task: None,
            login_task: None,
            logout_task: None,
            remove_account_task: None,
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
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self{
        Self::configure_style(&cc.egui_ctx);
        Self::default()
    }
    pub(super) fn balance(&self) -> i64 {
        let mut balance: i64 = self.sheet_collections[self.active_collection].sheets[0].sum();
        for sheet in &self.sheet_collections[self.active_collection].sheets[1..self.sheet_collections[self.active_collection].len()] {
            balance -= sheet.sum();
        }
        balance
    }
    pub(super) fn balance_display(&self) -> String {
        (self.balance() as f64 / 100.0).to_string()
    }
    pub(super) fn normalize_password(password: &str) -> String {
        password.nfc().collect()
    }

    pub(super) fn active_collection(&self) -> &SheetCollection {
        let collection_index = self.active_collection;
        &self.sheet_collections[collection_index]
    }
    pub(super) fn active_collection_mut(&mut self) -> &mut SheetCollection {
        let collection_index = self.active_collection;
        &mut self.sheet_collections[collection_index]
    }

    pub(super) fn log(&mut self, message: &str) {
        println!("[EMM LOG]: {}", message);
        self.error_msg = Some(message.to_string());
    }
    pub(super) fn log_error(&mut self, message: &str) {
        eprintln!("[EMM ERROR]: {}", message);
        self.error_msg = Some(message.to_string());
    }
    pub(super) fn quit(&mut self) {
        self.quit_after_logout = true;
        self.logout();
    }
}

impl eframe::App for EasyMoneyManager {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx: egui::Context = ui.ctx().clone();
        Self::paint_background(ui);
        match self.auth_state {
            AuthState::LoggedOut   => self.login_ui(ui),
            AuthState::LoggingIn   => self.logging_ui(ui),
            AuthState::LoggedIn(_) => self.main_ui(ui),
            AuthState::LoggingOut(_)  => self.unlogging_ui(ui),
            AuthState::Registering => self.registering_ui(ui),
        }
    }
}
