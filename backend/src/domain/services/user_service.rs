use crate::domain::error::{self, Error};
use crate::domain::model::{TransactionToSign, User};
use crate::repo::user::Repo;

use solana_sdk::pubkey::Pubkey;
use uuid::Uuid;

use super::solana_service;

#[derive(Clone)]
struct Config {
    access_token_signing_key: Vec<u8>,
    access_token_validity_sec: u32,
}

impl Config {
    fn default() -> Config {
        Config {
            access_token_validity_sec: 3600,
            access_token_signing_key: jwt_simple::prelude::HS256Key::generate().to_bytes(),
        }
    }
}

#[derive(Clone)]
pub struct UserService {
    cfg: Config,
    repo: Repo,
    solana: solana_service::SolanaService,
}

impl UserService {
    pub fn new(repo: Repo, solana: solana_service::SolanaService) -> UserService {
        UserService {
            cfg: Config::default(),
            repo: repo,
            solana: solana,
        }
    }

    pub fn get_user(&self, pubkey: &Pubkey) -> Result<User, Error> {
        self.repo.get_user(&pubkey)
    }

    pub fn register_complete(
        &self,
        pubkey: &Pubkey,
    ) -> Result<
        (
            String, /* access_token */
            String, /* refresh_token */
        ),
        Error,
    > {
        let mut user: User = self.repo.get_user(pubkey)?;
        // Validate the user
        if user.pda_pubkey.is_some() {
            return Err(error::Error::UserAlreadyInitialized);
        }

        // Get user PDA
        let pda = self.solana.get_user_pda(pubkey);
        user.pda_pubkey = Some(pda);

        // Generate access and refresh tokens
        let access_token = self.generate_jwt_token(pubkey)?;
        let refresh_token = Uuid::new_v4().to_string();
        user.refresh_token = Some(refresh_token.clone());

        // Save refresh token and pda_pubkey in user repo
        self.repo.update_user(&user)?;

        Ok((access_token, refresh_token))
    }

    pub fn register_init(
        &self,
        pubkey: &Pubkey,
        username: String,
    ) -> Result<TransactionToSign, Error> {
        let user: User = User {
            pubkey: pubkey.clone(),
            username: username,
            pda_pubkey: None,
            refresh_token: None,
        };

        // Check if we already have this user, update if found.
        let res = self
            .repo
            .get_user(&pubkey)
            .and_then(|_| self.repo.update_user(&user));

        // If the result is Err, there are 2 options:
        // user doesn't exist yet (no error) or internal error.
        if let Err(err) = res {
            if err.eq(&Error::UserNotFound) {
                self.repo.add_user(user.clone())?;
            } else {
                // Internal error
                return Err(err);
            }
        }

        // Now we want to create a transaction message creating a PDA
        // for this use. User will sign it and forward it back to backend.
        self.solana.create_user_pda(pubkey)
    }

    fn generate_jwt_token(&self, pubkey: &Pubkey) -> Result<String, Error> {
        use jwt_simple::prelude::*;

        // create a new key for the `HS256` JWT algorithm
        let key = HS256Key::from_bytes(&self.cfg.access_token_signing_key);
        let nonce = Uuid::new_v4();
        let jwt_id = crate::utils::jwt::generate_jwt_id(pubkey, &nonce);

        let claims = Claims::create(Duration::from_secs(
            self.cfg.access_token_validity_sec as u64,
        ))
        .with_issuer("anti-loneliness")
        .with_jwt_id(jwt_id)
        .with_nonce(nonce);
        let token = key.authenticate(claims).map_err(|err| {
            println!("Failed to generate auth token: {}", err);
            Error::GeneralError("Failed to generate auth token".to_string())
        })?;
        Ok(token)
    }
}
