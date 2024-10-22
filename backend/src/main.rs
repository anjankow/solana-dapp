mod handlers;
mod server;
mod user;

#[tokio::main]
async fn main() {
    let server = server::Server::new();
    server.run("127.0.0.1:3000").await.unwrap();
}
