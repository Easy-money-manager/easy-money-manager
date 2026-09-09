mod server;
mod database;
mod record;

#[tokio::main]
async fn main() {
    server::run().await;
}
