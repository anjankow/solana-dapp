pub mod solana {
    use crate::domain::error::Error;
    use crate::domain::model::TransactionRecord;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    pub struct Repo {
        transactions: Arc<Mutex<HashMap<Uuid, TransactionRecord>>>,
    }

    impl Repo {
        pub fn new() -> Repo {
            Repo {
                transactions: Arc::new(Mutex::new(HashMap::<Uuid, TransactionRecord>::new())),
            }
        }

        pub fn get_transaction_record(&self, record_id: Uuid) -> Result<TransactionRecord, Error> {
            let transactions = self.transactions.lock().unwrap();
            transactions
                .get(&record_id)
                .map(|u| u.clone())
                .ok_or(Error::TransactionNotFound)
        }

        pub fn add_transaction_record(
            &self,
            transaction_record: &mut TransactionRecord,
        ) -> Result<Uuid, Error> {
            let id = Uuid::new_v4();
            transaction_record.id = id.clone();
            let mut transactions = self.transactions.lock().unwrap();
            transactions.insert(id, transaction_record.clone());
            Ok(id)
        }

        pub fn update_transaction_record(
            &self,
            transaction_record: &TransactionRecord,
        ) -> Result<(), Error> {
            let mut transactions = self.transactions.lock().unwrap();
            transactions.insert(transaction_record.id, transaction_record.clone());
            Ok(())
        }

        pub fn delete_transaction_record(&self, record_id: Uuid) -> Result<(), Error> {
            let mut transactions = self.transactions.lock().unwrap();
            transactions.remove(&record_id);
            Ok(())
        }
    }
}

pub mod user {
    use crate::domain::error::Error;
    use crate::domain::model::User;
    use solana_sdk::pubkey::Pubkey;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub struct Repo {
        users: Arc<Mutex<HashMap<Pubkey, User>>>,
    }

    impl Repo {
        pub fn new() -> Repo {
            Repo {
                users: Arc::new(Mutex::new(HashMap::<Pubkey, User>::new())),
            }
        }

        pub fn add_user(&self, user: User) -> Result<(), Error> {
            let mut users = self.users.lock().unwrap();
            if users.contains_key(&user.pubkey) {
                return Err(Error::UserAlreadyInitialized);
            }
            users.insert(user.pubkey, user);
            Ok(())
        }

        pub fn get_user(&self, pubkey: &Pubkey) -> Result<User, Error> {
            let users = self.users.lock().unwrap();
            users
                .get(pubkey)
                .map(|u| u.clone())
                .ok_or(Error::UserNotFound)
        }

        pub fn update_user(&self, user: &User) -> Result<(), Error> {
            let mut users = self.users.lock().unwrap();
            users.insert(user.pubkey, user.clone());
            Ok(())
        }
    }
}
