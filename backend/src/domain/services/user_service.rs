use std::str::FromStr;
use std::time::SystemTime;

use crate::domain::error::{self, Error};
use crate::domain::model::{TransactionToSign, User};
use crate::repo::user::Repo;

use solana_sdk::bs58::decode::DecodeTarget;
use solana_sdk::pubkey::Pubkey;
use uuid::Uuid;

use super::solana_service;

#[derive(Clone)]
pub struct Config {
    access_token_validity_sec: u32,
    refresh_token_validity_sec: u32,
    token_issuer: String,
}

impl Config {
    pub fn default() -> Config {
        Config {
            access_token_validity_sec: 3600,
            refresh_token_validity_sec: 7200,
            token_issuer: "anti-loneliness".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct UserService {
    cfg: Config,
    repo: Repo,
    auth_secret: Vec<u8>,
    solana: solana_service::SolanaService,
}

#[derive(Clone)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
}

impl UserService {
    pub fn new(
        cfg: Config,
        repo: Repo,
        auth_secret: Vec<u8>,
        solana: solana_service::SolanaService,
    ) -> UserService {
        UserService {
            cfg,
            auth_secret,
            repo,
            solana,
        }
    }

    pub fn get_user(&self, pubkey: &Pubkey) -> Result<User, Error> {
        self.repo.get_user(&pubkey)
    }

    pub fn login_init(&self, pubkey: &Pubkey) -> Result<String /* refresh_token */, Error> {
        let mut user: User = self.repo.get_user(pubkey)?;
        // Validate the user
        if user.pda_pubkey.is_none() {
            return Err(error::Error::UserNotConfirmed);
        }

        // User gets signed in by signing the refresh token.
        // Generate one for this user.
        self.assign_new_refresh_token(&mut user)
    }

    fn assign_new_refresh_token(&self, user: &mut User) -> Result<String, Error> {
        let refresh_token = Uuid::new_v4().to_string();
        user.refresh_token = Some(crate::domain::model::RefreshToken {
            token: refresh_token.clone(),
            valid_until: SystemTime::now()
                .checked_add(std::time::Duration::from_secs(
                    self.cfg.refresh_token_validity_sec as u64,
                ))
                .expect("Refresh token validity is invalid and causes time register overflow"),
        });
        self.repo.update_user(&user)?;

        Ok(refresh_token)
    }

    pub fn login_complete(
        &self,
        pubkey: &Pubkey,
        refresh_token: String,
        signature: String,
    ) -> Result<AuthTokens, Error> {
        // Convert the signature
        let signature = ed25519_dalek::Signature::from_str(&signature).map_err(|err| {
            println!("Casting to ed25519_dalek::Signature failed: {}", err);
            error::Error::InvalidSignature
        })?;

        // Validate the user
        let mut user: User = self.repo.get_user(pubkey)?;
        if user.pda_pubkey.is_none() || user.refresh_token.is_none() {
            // Possible only if this user hasn't confirmed the registration yet.
            return Err(error::Error::UserNotConfirmed);
        }

        let expected_refresh_token = user
            .refresh_token
            .as_ref()
            .ok_or(error::Error::UserNotConfirmed)?;

        // Validate the given refresh token against the expectation
        expected_refresh_token.verify(&refresh_token)?;

        let verifier =
            ed25519_dalek::VerifyingKey::from_bytes(&pubkey.to_bytes()).map_err(|_| {
                error::Error::InvalidPubKey(
                    "Failed to convert pubkey, impossible to verify signature".to_string(),
                )
            })?;

        verifier
            .verify_strict(refresh_token.as_bytes(), &signature)
            .map_err(|_| {
                println!("Signature verification failed");
                error::Error::InvalidSignature
            })?;

        // All checks passed, can refresh the auth token now
        self.assign_auth_tokens(&mut user)
    }

    pub fn register_complete(&self, pubkey: &Pubkey) -> Result<AuthTokens, Error> {
        let mut user: User = self.repo.get_user(pubkey)?;

        // In case of registration
        if user.pda_pubkey.is_some() {
            return Err(error::Error::UserAlreadyInitialized);
        }

        // Get user PDA
        let pda = self.solana.get_user_pda(pubkey);
        user.pda_pubkey = Some(pda);

        // Generate access and refresh tokens
        self.assign_auth_tokens(&mut user)
    }

    fn assign_auth_tokens(&self, user: &mut User) -> Result<AuthTokens, Error> {
        let access_token = self.generate_jwt_token(&user.pubkey)?;
        let refresh_token = self.assign_new_refresh_token(user)?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
        })
    }

    pub fn register_init(
        &self,
        pubkey: &Pubkey,
        username: String,
    ) -> Result<TransactionToSign, Error> {
        let user: User = User {
            pubkey: pubkey.clone(),
            username,
            pda_pubkey: None,
            refresh_token: None,
        };

        // Check if we already have this user, update if found.
        let res = self.repo.get_user(&pubkey).and_then(|existing| {
            // If the user has been already registered, don't overrride.
            if existing.pda_pubkey.is_some() {
                return Err(error::Error::UserAlreadyInitialized);
            }
            self.repo.update_user(&user)
        });

        if let Err(err) = res {
            if err.eq(&Error::UserNotFound) {
                self.repo.add_user(user.clone())?;
            } else {
                return Err(err);
            }
        }

        // Now we want to create a transaction message creating a PDA
        // for this use. User will sign it and forward it back to backend.
        self.solana.create_user_pda(pubkey)
    }

    fn generate_jwt_token(&self, pubkey: &Pubkey) -> Result<String, Error> {
        use jwt_simple::prelude::*;

        #[derive(Serialize, Deserialize)]
        struct UserClaims {
            pubkey: String,
        }
        let user_claims = UserClaims {
            pubkey: pubkey.to_string(),
        };
        // create a new key for the `HS256` JWT algorithm
        let key = HS256Key::from_bytes(&self.auth_secret);
        let nonce = Uuid::new_v4();

        let claims = Claims::with_custom_claims(
            user_claims,
            Duration::from_secs(self.cfg.access_token_validity_sec as u64),
        )
        .with_issuer(&self.cfg.token_issuer)
        .with_nonce(nonce);
        let token = key.authenticate(claims).map_err(|err| {
            println!("Failed to generate auth token: {}", err);
            Error::GeneralError("Failed to generate auth token".to_string())
        })?;
        Ok(token)
    }
}
