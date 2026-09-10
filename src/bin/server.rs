#[path = "../server.rs"]
mod server;
#[path = "../database.rs"]
mod database;
#[path = "../record.rs"]
mod record;
#[path = "../sheetcollection.rs"]
mod sheetcollection;
#[path = "../sheet.rs"]
mod sheet;
#[path = "../requests.rs"]
mod requests;

#[tokio::main]
async fn main() {
    server::run().await;
}
