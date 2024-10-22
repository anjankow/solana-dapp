use crate::domain::error::Error;
use crate::domain::model::user::User;
use crate::repo::user_repo::UserRepo;
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use num_bigint::{BigUint, RandomBits};
use rand::Rng;
use solana_sdk::pubkey::Pubkey;

pub struct UserMgr {
    repo: Arc<Mutex<UserRepo>>,
}

impl UserMgr {
    pub fn new(repo: Arc<Mutex<UserRepo>>) -> UserMgr {
        UserMgr { repo: repo }
    }

    pub fn get_user(&self, pubkey: &String) -> Result<User, Error> {
        let pubkey_parsed = parse_pubkey(&pubkey)?;

        // If a thread panics while the mutex is locked we can't be certain
        // if the value inside Mutex is still valid and thus the default
        // behaviour is to return an error instead of a guard.
        let repo = self.repo.lock().unwrap();
        repo.get(pubkey_parsed)
    }

    pub fn create_user(&self, pubkey: &String, username: String) -> Result<User, Error> {
        // parse pubkey
        let pubkey_parsed = parse_pubkey(&pubkey)?;

        // generate nonce for this user
        let mut rng = rand::thread_rng();
        let nonce: BigUint = rng.sample(RandomBits::new(64));
        let user = User {
            pubkey: pubkey_parsed,
            nonce: *nonce.to_u64_digits().get(0).expect("Requested nonce is 64 bits long, so the resulting vector should have exactly 1 elem of type u64"),
            username: username, // can be empty
        };

        // add this user to the repo
        {
            // If a thread panics while the mutex is locked we can't be certain
            // if the value inside Mutex is still valid and thus the default
            // behaviour is to return an error instead of a guard.
            let mut repo = self.repo.lock().unwrap();
            repo.add(user.clone())?;
        }
        Ok(user)
    }
}

fn parse_pubkey(pubkey: &String) -> Result<Pubkey, Error> {
    pubkey
        .parse::<solana_sdk::pubkey::Pubkey>()
        .map_err(|e| Error::from(e))
        .inspect_err(|e| println!("Failed to parse pubkey: {}", e))
}
