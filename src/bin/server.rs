#[path="../server.rs"]
mod server;
#[path="../database.rs"]
mod database;
#[path="../record.rs"]
mod record;

#[tokio::main]
async fn main() {
    server::run().await;
}
