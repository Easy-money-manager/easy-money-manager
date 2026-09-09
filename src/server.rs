use axum::{ Router, routing::get, Json, extract::{ Path, State } };
use serde::Serialize;
use std::sync::{ Arc, Mutex};
use crate::database::Database;
use crate::record::Record;

#[derive(Serialize)]
struct TestResponse {
    message: String,
}

#[derive(Clone)]
pub struct AppState {
    database: Arc<Mutex<Database>>,
}

async fn test() -> Json<TestResponse> {
    Json(TestResponse {
        message: "Server works".to_string(),
    })
}

pub async fn get_records(Path(sheet_id): Path<i64>, State(state): State<AppState>) -> Json<Vec<Record>> {
    let database = state.database.lock().unwrap();

    let records = database.get_records(sheet_id).unwrap();

    Json(records)
}

pub async fn run() {
    let database = Database::new("easy_money_manager.db").expect("Failed to initialize database");

    let state = AppState {
        database: Arc::new(Mutex::new(database)),
    };

    let app = Router::new()
        .route("/", get(test))
        .route("/records/{sheet_id}", get(get_records))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
