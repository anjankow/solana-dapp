use num_bigint::{BigUint, RandomBits};
use rand::Rng;

struct User {
    pubkey: solana_sdk::pubkey::Pubkey,
    nonce: u64,
    username: String,
}

fn create_user(pubkey: &String, username: String) {
    // generate nonce for this user
    let mut rng = rand::thread_rng();
    let nonce: BigUint = rng.sample(RandomBits::new(256));
}
