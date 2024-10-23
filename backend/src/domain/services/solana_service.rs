use std::sync::Arc;

use solana_sdk::{
    message::Message,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction,
    transaction::{self, Transaction},
};

use crate::domain::error::Error;

const USER_PDA_PREFIX: &[u8] = b"user";

#[derive(Clone)]
pub struct Config {
    user_pda_size: usize,
    rpc_client_url: String,
}

impl Config {
    pub fn default() -> Config {
        Config {
            user_pda_size: 1024,
            rpc_client_url: "http://localhost:8899".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct SolanaService {
    cfg: Config,
    program: Arc<Keypair>,
    client: Arc<solana_client::rpc_client::RpcClient>,
}

impl SolanaService {
    pub fn new(cfg: Config, program: Keypair) -> SolanaService {
        SolanaService {
            cfg: Config::default(),
            program: Arc::new(program),
            client: Arc::new(solana_client::rpc_client::RpcClient::new(
                &cfg.rpc_client_url,
            )),
        }
    }

    pub fn create_user_pda(&self, wallet_pubkey: Pubkey) -> Result<Message, Error> {
        let account_size = self.cfg.user_pda_size;

        let lamports = self
            .client
            .get_minimum_balance_for_rent_exemption(account_size)?;

        // Derive the PDA from the payer account, a string representing the unique
        // purpose of the account, and the address of our on-chain program.
        let seeds = &[USER_PDA_PREFIX, wallet_pubkey.as_ref()];
        let (pda_pubkey, pda_bump_seed) =
            Pubkey::find_program_address(seeds, &self.program.pubkey());

        // todo: when instruction available
        Ok(Message::default())
    }

    pub fn send_and_confirm_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<Signature, Error> {
        let signature = self.client.send_and_confirm_transaction(&transaction)?;
        Ok(signature)
    }
}
