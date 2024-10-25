mod domain;
mod repo;
mod server;
mod utils;

use solana_sdk::signature::Keypair;

use crate::server::Server;

#[tokio::main]
async fn main() {
    // relative to Cargo
    let keypair_dir = "solana_program/target/deploy/";
    let keypair = get_keypair_from_dir(keypair_dir);
    let auth_secret = jwt_simple::prelude::HS256Key::generate().to_bytes();
    let cfg = server::Config::default();
    let server = Server::new(cfg);
    server.run(auth_secret, keypair).await.unwrap();
}

fn get_keypair_from_dir(dir: &str) -> Keypair {
    let files = std::fs::read_dir(dir).unwrap();

    let file_path = files
        .filter_map(Result::ok)
        .filter_map(|d| {
            d.path().to_str().and_then(|f| {
                if f.ends_with("keypair.json") {
                    Some(d)
                } else {
                    None
                }
            })
        })
        .next()
        .unwrap()
        .path();
    solana_sdk::signer::keypair::read_keypair_file(file_path).unwrap()
}
