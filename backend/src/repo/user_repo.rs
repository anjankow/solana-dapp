use crate::domain::error::Error;
use crate::domain::model::user::User;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct UserRepo {
    repo: HashMap<Pubkey, User>,
}

impl UserRepo {
    pub fn new() -> UserRepo {
        UserRepo {
            repo: HashMap::<Pubkey, User>::new(),
        }
    }
    pub fn add(&mut self, user: User) -> Result<(), Error> {
        if self.repo.contains_key(&user.pubkey) {
            return Err(Error::UserAlreadyInitialized);
        }
        self.repo.insert(user.pubkey, user);
        Ok(())
    }

    pub fn get(&self, pubkey: Pubkey) -> Result<User, Error> {
        self.repo
            .get(&pubkey)
            .map(|u| u.clone())
            .ok_or(Error::UserNotFound)
    }
}
