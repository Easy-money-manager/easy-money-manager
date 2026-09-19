use super::EasyMoneyManager;
use crate::clienttask::ClientTask;
use crate::api::ApiClient;
use emm_shared::request::{ RegisterRequest, LoginRequest };
use emm_shared::response::{ LoginResponse };
use eframe::egui;

#[allow(dead_code)]
pub(super) struct UserSession {
    pub(super) user_id: i64,
    pub(super) username: String,
    pub(super) session_token: String,
}


pub(super) enum AuthState {
    LoggedOut,
    LoggingIn,
    LoggedIn(UserSession),
    LoggingOut(UserSession),
    Registering,
}

impl EasyMoneyManager {
    pub(super) fn username(&self) -> Option<&str> {
        match &self.auth_state {
            AuthState::LoggedIn(session) => Some(&session.username),
            _ => None,
        }
    }
    pub(super) fn session_token(&self) -> Option<&str> {
        match &self.auth_state {
            AuthState::LoggedIn(session) => Some(&session.session_token),
            _ => None,
        }
    }
    pub(super) fn session_token_clone(&self) -> Option<String> {
        match &self.auth_state {
            AuthState::LoggedIn(session) => Some(session.session_token.clone()),
            _ => None,
        }
    }

    pub(super) fn register(&mut self) {
        let request: RegisterRequest = RegisterRequest {
            username: self.input_username.clone(),
            password: Self::normalize_password(&self.input_password.clone()),
        };
        self.input_username.clear();
        self.input_password.clear();
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
        self.auth_state = AuthState::Registering;
    }
    pub(super) fn login(&mut self) {
        let request: LoginRequest = LoginRequest {
            username: self.input_username.clone(),
            password: Self::normalize_password(&self.input_password.clone()),
        };
        self.input_username.clear();
        self.input_password.clear();
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
    pub(super) fn logout(&mut self) {
        let api_client: ApiClient = self.api_client.clone();
        let session_token: String = match self.session_token_clone() {
            Some(session_token) => session_token,
            None => panic!("Called logout when not logged in"),
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
                ClientTask::spawn(async move {
                    api_client.logout(&session_token).await
                })
            );
        }
        self.auth_state = match std::mem::replace (
            &mut self.auth_state,
            AuthState::LoggedOut,
        ){
            AuthState::LoggedIn(session) => AuthState::LoggingOut(session),
            _ => panic!("Called remove account when not logged in"),
        };
    }
    pub(super) fn remove_account(&mut self) {
        let api_client: ApiClient = self.api_client.clone();
        let session_token: String = match self.session_token_clone() {
            Some(session_token) => session_token,
            None => panic!("Called remove account when not logged in"),
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.remove_account_task = Some(
                ClientTask::spawn(
                    &self.runtime,
                    async move {
                        api_client.remove_account(&session_token).await
                    }
                )
            );
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.remove_account_task = Some(
                ClientTask::spawn(async move {
                    api_client.remove_account(&session_token).await
                })
            );
        }
        self.auth_state = match std::mem::replace (
            &mut self.auth_state,
            AuthState::LoggedOut,
        ){
            AuthState::LoggedIn(session) => AuthState::LoggingOut(session),
            _ => panic!("Called remove account when not logged in"),
        };
    }

    pub(super) fn handle_register_task(&mut self) {
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
            Ok(Ok(()))     => {
                self.auth_error = Some("Account created succesfully, you may log in now!".to_string());
                self.auth_state = AuthState::LoggedOut;
            }
            Ok(Err(error)) => {
                self.auth_state = AuthState::LoggedOut;
                self.auth_error = Some("Registration failed, probably name already in use\nPossibly server may be down".to_string());
                self.log_error(&format!("Registration failed: {}", error));
            }
            Err(error)     => {
                self.auth_error = Some("Registration failed die to server issue, sorry!".to_string());
                self.log_error(&format!("Registration task failed: {}", error));
            }
        };
    }
    pub(super) fn handle_login_task(&mut self) {
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
                self.sheet_collections = response.bootstrap.collections;
                self.bootstrap_loaded = true;
                self.log(&format!("Bootstrap loaded"));
                self.error_msg = None;
                self.auth_error = None;
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
    pub(super) fn handle_logout_task(&mut self, ctx: &egui::Context) {
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
                if self.quit_after_logout {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                self.log(&"Logged out successfully".to_string());
                self.auth_error = Some("Logged out successfully".to_string());
            }
            Ok(Err(error)) => {
                self.auth_state = match std::mem::replace (
                    &mut self.auth_state,
                    AuthState::LoggedOut,
                ){
                    AuthState::LoggingOut(session) => AuthState::LoggedIn(session),
                    _ => panic!("Called remove account when not logged in"),
                };
                self.log(&format!("Couldn't log out: {}", error));
                self.auth_error = Some(format!("Couldn't log out: {}", error));
            }
            Err(error) => {
                self.auth_state = match std::mem::replace (
                    &mut self.auth_state,
                    AuthState::LoggedOut,
                ){
                    AuthState::LoggingOut(session) => AuthState::LoggedIn(session),
                    _ => panic!("Called remove account when not logged in"),
                };
                self.log_error(&format!("Logout task failed: {}", error));
            }
        }
    }
    pub(super) fn handle_remove_account_task(&mut self) {
        let finished: bool = match &self.remove_account_task {
            Some(task) => task.is_finished(),
            None       => false,
        };
        if !finished {
            return;
        }
        let task = match self.remove_account_task.take() {
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
                self.log(&"Removed account successfully".to_string());
                self.auth_error = Some("Removed account successfully".to_string())
            }
            Ok(Err(error)) => {
                self.auth_state = match std::mem::replace (
                    &mut self.auth_state,
                    AuthState::LoggedOut,
                ){
                    AuthState::LoggingOut(session) => AuthState::LoggedIn(session),
                    _ => panic!("Called remove account when not logged in"),
                };
                self.log(&format!("Couldn't remove_account: {}", error));
            }
            Err(error) => {
                self.auth_state = match std::mem::replace (
                    &mut self.auth_state,
                    AuthState::LoggedOut,
                ){
                    AuthState::LoggingOut(session) => AuthState::LoggedIn(session),
                    _ => panic!("Called remove account when not logged in"),
                };
                self.log_error(&format!("Remove account task failed: {}", error));
            }
        }
    }
}
