use crate::domain::model::user::User;
use crate::repo::error::Error;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct UserRepo {
    repo: HashMap<String, User>,
}

impl UserRepo {
    pub fn new() -> UserRepo {
        UserRepo {
            repo: HashMap::<String, User>::new(),
        }
    }
    pub fn add(&mut self, user: User) -> Result<(), crate::domain::error::Error> {
        self.repo.insert(user.pubkey.to_string(), user);
        Ok(())
    }
}
