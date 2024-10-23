use crate::domain;
use crate::domain::error::{self, Error};
use crate::domain::model::{AccessToken, TransactionRecord, TransactionToSign, User};
use crate::repo::Repo;
use std::any::Any;
use std::time::SystemTime;
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use num_bigint::{BigUint, RandomBits};
use rand::Rng;
use solana_sdk::message::Message;
use solana_sdk::{pubkey::Pubkey, transaction::Transaction};
use uuid::Uuid;

use super::solana_service;

struct Config {
    access_token_validity_sec: u32,
    transaction_validity_sec: u32,
}

impl Config {
    fn default() -> Config {
        Config {
            access_token_validity_sec: 3600,
            transaction_validity_sec: 3600,
        }
    }
}

pub struct UserService {
    cfg: Config,
    repo: Arc<Mutex<Repo>>,
    solana: solana_service::SolanaService,
}

impl UserService {
    pub fn new(repo: Arc<Mutex<Repo>>, solana: solana_service::SolanaService) -> UserService {
        UserService {
            cfg: Config::default(),
            repo: repo,
            solana: solana,
        }
    }

    pub fn get_user(&self, pubkey: &String) -> Result<User, Error> {
        let pubkey_parsed = parse_pubkey(&pubkey)?;

        // If a thread panics while the mutex is locked we can't be certain
        // if the value inside Mutex is still valid and thus the default
        // behaviour is to return an error instead of a guard.
        let repo = self.repo.lock().unwrap();
        repo.get_user(pubkey_parsed)
    }

    pub fn register_complete(
        &self,
        pubkey: &String,
        transaction_id: Uuid,
        signed_transaction: Transaction,
    ) -> Result<AccessToken, Error> {
        let pubkey = parse_pubkey(&pubkey)?;

        let mut user: User;
        let transaction_record: domain::model::TransactionRecord;
        {
            let repo = self.repo.lock().unwrap();
            user = repo.get_user(pubkey)?;
            transaction_record = repo.get_transaction_record(transaction_id)?;
        }

        // validate the user
        if user.pda_pubkey.is_some() {
            return Err(error::Error::UserAlreadyInitialized);
        }

        // validate the received transaction
        if transaction_record.valid_until.lt(&SystemTime::now()) {
            return Err(error::Error::TransactionExpired);
        }

        if signed_transaction
            .message()
            .hash()
            .ne(&transaction_record.message_hash)
            || !signed_transaction.is_signed()
        {
            return Err(error::Error::InvalidTransaction);
        }

        // Solana will handle the verification of the signature itself.
        // Create the user's PDA.
        // todo: solana create user

        // User's PDA has been successfully created.
        user.pda_pubkey = None; // todo: set

        // Return the access token to the user's account.
        let access_token = AccessToken {
            access_token: Uuid::new_v4().to_string(), // todo: JWT
            expires_in: self.cfg.access_token_validity_sec,
            token_type: "Bearer".to_string(),
            nonce: generate_nounce(),
            pubkey: pubkey,
        };

        // In case if this transaction fails, the new PDA on solana should be closed.
        {
            let mut repo = self.repo.lock().unwrap();
            repo.delete_transaction_record(transaction_id)?;
            repo.insert_access_token(access_token.clone())?;
            repo.update_user(user)?;
        }

        Ok(access_token)
    }

    pub fn register_init(
        &self,
        pubkey: &String,
        username: String,
    ) -> Result<TransactionToSign, Error> {
        let pubkey = parse_pubkey(&pubkey)?;

        let user: User = User {
            pubkey: pubkey.clone(),
            username: username,
            pda_pubkey: None,
        };

        {
            // Check if we have this user in the repo already.
            let mut repo = self.repo.lock().unwrap();
            let mut existing: Option<User> = None;
            let res = repo.get_user(pubkey).inspect(|e| {
                // assign the existing user
                existing = Some(e.clone())
            });
            if let Err(err) = res {
                if err.ne(&Error::UserNotFound) {
                    // Server error
                    return Err(err);
                }
            }

            if let Some(_) = existing {
                // update existing user
                repo.update_user(user.clone())?;
            } else {
                // insert a new user
                repo.add_user(user.clone())?;
            }
        }

        // Now our user is created/updated.
        // We want to create a transaction message to be signed by this user.
        let message = self.solana.create_user_pda(pubkey)?;

        // Save the transaction record
        let mut transaction_id: Uuid;
        let transaction_record = TransactionRecord {
            id: Uuid::nil(),
            message_hash: message.hash(),
            pubkey: pubkey,
            valid_until: SystemTime::now()
                .checked_add(std::time::Duration::from_secs(
                    self.cfg.transaction_validity_sec as u64,
                ))
                .expect("Time adding should never exceed the bounds here"),
        };
        {
            let mut repo = self.repo.lock().unwrap();
            transaction_id = repo.add_transaction_record(transaction_record.clone())?;
        }

        // Transaction saved successfully, now we can prepare TransactionToSign for frontend.
        Ok(TransactionToSign {
            message,
            transaction_id,
            valid_until: transaction_record.valid_until,
            callback: "/api/v1/auth/register/complete".to_string(),
        })
    }
}

fn parse_pubkey(pubkey: &String) -> Result<Pubkey, Error> {
    pubkey
        .parse::<solana_sdk::pubkey::Pubkey>()
        .map_err(|e| Error::from(e))
        .inspect_err(|e| println!("Failed to parse pubkey: {}", e))
}

fn generate_nounce() -> u64 {
    // generate nonce for this user
    let mut rng = rand::thread_rng();
    let nonce: BigUint = rng.sample(RandomBits::new(64));
    *nonce.to_u64_digits().get(0).expect("Requested nonce is 64 bits long, so the resulting vector should have exactly 1 elem of type u64")
}
