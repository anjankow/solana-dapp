mod domain;
mod repo;
mod server;
use crate::server::Server;

#[tokio::main]
async fn main() {
    let server = Server::new();
    server.run("127.0.0.1:3000").await.unwrap();
}
