use crate::handlers;

use axum::{routing::get, Router};
pub struct Server {}

impl Server {
    pub fn new() -> Server {
        Server {}
    }

    pub async fn run(&self, bind_address: &str) -> Result<(), std::io::Error> {
        // bind all the routes
        let router = Router::new().route("/", get(handlers::handler));

        let listener = tokio::net::TcpListener::bind(bind_address).await?;

        println!(
            "listening on {}",
            listener
                .local_addr()
                .map(|a| a.to_string())
                .unwrap_or("<NO LOCAL ADDRESS>".to_string()),
        );
        axum::serve(listener, router).await?;
        Ok(())
    }
}
