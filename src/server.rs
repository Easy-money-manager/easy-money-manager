use axum::{ Router, routing::{ get, put }, Json, extract::{ Path, State }, http::StatusCode };
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

#[derive(serde::Deserialize)]
struct CreateRecordRequest {
    description: String,
    date: chrono::NaiveDate,
    value: i64,
}

#[derive(serde::Deserialize)]
struct UpdateRecordRequest {
    description: String,
    date: chrono::NaiveDate,
    value: i64,
}

async fn test() -> Json<TestResponse> {
    Json(TestResponse {
        message: "Server works".to_string(),
    })
}

async fn get_records(Path(sheet_id): Path<i64>, State(state): State<AppState>) -> Result<Json<Vec<Record>>, StatusCode> {
    let database = match state.database.lock() {
        Ok(database) => database,
        Err(error)   => {
            eprintln!("Failed to lock database: {}", error);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match database.get_records(sheet_id) {
        Ok(records) => Ok(Json(records)),
        Err(error)  => {
            eprintln!("Failed to get records: {}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_record(Path(sheet_id): Path<i64>, State(state): State<AppState>, Json(input): Json<CreateRecordRequest>) -> Result<Json<Record>, StatusCode> {
    let database = match state.database.lock() {
        Ok(database) => database,
        Err(error)   => {
            eprintln!("Failed to lock database: {}", error);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let id = match database.create_record(sheet_id, &input.description, input.date, input.value) {
        Ok(id)     => id,
        Err(error) => {
            eprintln!("Failed to create record: {}", error);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(Record {
        id,
        description: input.description,
        date: input.date,
        value: input.value
    }))
}

async fn update_record(Path(id): Path<i64>, State(state): State<AppState>, Json(input): Json<UpdateRecordRequest>) -> Result<StatusCode, StatusCode> {
    let database = match state.database.lock() {
        Ok(database) => database,
        Err(error)   => {
            eprintln!("Failed to lock database: {}", error);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match database.update_record(id, &input.description, input.date, input.value) {
        Ok(())     => Ok(StatusCode::NO_CONTENT),
        Err(error) => {
            eprintln!("Failed to edit record: {}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn remove_record(Path(id): Path<i64>, State(state): State<AppState>) -> Result<StatusCode, StatusCode> {
    let database = match state.database.lock() {
        Ok(database) => database,
        Err(error)   => {
            eprintln!("Failed to lock database: {}", error);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match database.remove_record(id) {
        Ok(())     => Ok(StatusCode::NO_CONTENT),
        Err(error) => {
            eprintln!("Failed to remove record: {}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

fn records_router() -> Router<AppState> {
    Router::new().route(
        "/records/sheet/{sheet_id}",
        get(get_records)
        .post(create_record)
    ).route(
        "/records/{id}",
        put(update_record)
        .delete(remove_record)
    )
}

pub async fn run() {
    let database = Database::new("easy_money_manager.db").expect("Failed to initialize database");
    match database.initialize() {
        Ok(())     => eprintln!("Initialized database"),
        Err(error) => eprintln!("Failed to initialize database: {}", error),
    };

    let state = AppState {
        database: Arc::new(Mutex::new(database)),
    };

    let app = Router::new()
        .route("/", get(test))
        .merge(records_router())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
