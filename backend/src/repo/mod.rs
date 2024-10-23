use crate::domain::error::Error;
use crate::domain::model::{AccessToken, TransactionRecord, User};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

pub struct Repo {
    users: HashMap<Pubkey, User>,
    transactions: HashMap<Uuid, TransactionRecord>,
    access_tokens: HashMap<(String, Pubkey), AccessToken>,
}

impl Repo {
    pub fn new() -> Repo {
        Repo {
            users: HashMap::<Pubkey, User>::new(),
            transactions: HashMap::<Uuid, TransactionRecord>::new(),
            access_tokens: HashMap::<(String, Pubkey), AccessToken>::new(),
        }
    }
    pub fn add_user(&mut self, user: User) -> Result<(), Error> {
        if self.users.contains_key(&user.pubkey) {
            return Err(Error::UserAlreadyInitialized);
        }
        self.users.insert(user.pubkey, user);
        Ok(())
    }

    pub fn get_user(&self, pubkey: Pubkey) -> Result<User, Error> {
        self.users
            .get(&pubkey)
            .map(|u| u.clone())
            .ok_or(Error::UserNotFound)
    }

    pub fn update_user(&mut self, user: User) -> Result<(), Error> {
        self.users.insert(user.pubkey, user);
        Ok(())
    }

    pub fn get_transaction_record(&self, record_id: Uuid) -> Result<TransactionRecord, Error> {
        self.transactions
            .get(&record_id)
            .map(|u| u.clone())
            .ok_or(Error::TransactionNotFound)
    }

    pub fn add_transaction_record(
        &mut self,
        mut transaction_record: TransactionRecord,
    ) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        transaction_record.id = id.clone();
        self.transactions
            .insert(transaction_record.id, transaction_record);
        Ok(id)
    }

    pub fn delete_transaction_record(&mut self, record_id: Uuid) -> Result<(), Error> {
        self.transactions.remove(&record_id);
        Ok(())
    }

    pub fn insert_access_token(&mut self, access_token: AccessToken) -> Result<(), Error> {
        self.access_tokens.insert(
            (
                access_token.access_token.clone(),
                access_token.pubkey.clone(),
            ),
            access_token,
        );
        Ok(())
    }
}
