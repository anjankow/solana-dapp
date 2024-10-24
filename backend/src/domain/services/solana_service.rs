mod instruction;
use std::{sync::Arc, time};

use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
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
    commitment_config: CommitmentConfig,
    timeout_sec: u64,
}

impl Config {
    pub fn default() -> Config {
        Config {
            user_pda_size: 1024,
            rpc_client_url: "http://localhost:8899".to_string(),
            commitment_config: CommitmentConfig::confirmed(),
            timeout_sec: 5,
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
        let timeout = std::time::Duration::from_secs(cfg.timeout_sec);
        let solana_client = solana_client::rpc_client::RpcClient::new_with_timeout_and_commitment(
            &cfg.rpc_client_url,
            timeout,
            cfg.commitment_config,
        );
        SolanaService {
            cfg: cfg,
            program: Arc::new(program),
            client: Arc::new(solana_client),
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

        let instr_data =
            instruction::ProgramInstruction::Initialize(instruction::InitializeInstructionData {
                lamports,
                pda_bump_seed,
            })
            .pack()?;

        // The accounts required by both our on-chain program and the system program's
        // `create_account` instruction, including the vault's address.
        let accounts = vec![
            AccountMeta::new(wallet_pubkey, true /* is_signer */),
            AccountMeta::new(pda_pubkey, false),
            AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
        ];

        // Create the instruction by serializing our instruction data via borsh
        let instruction = Instruction::new_with_bytes(self.program.pubkey(), &instr_data, accounts);
        let message = Message::new(&[instruction], Some(&wallet_pubkey));

        // let blockhash = self.client.get_latest_blockhash()?;
        // let transaction = Transaction::new_unsigned(message);

        // todo: when instruction available
        Ok(message)
    }

    pub fn send_and_confirm_transaction(
        &self,
        wallet_pubkey: Pubkey,
        signed_transaction: Transaction,
        // message: Message,
        // signature: Signature,
    ) -> Result<Signature, Error> {
        // let mut transaction = Transaction::new_unsigned(message);
        // transaction
        //     .replace_signatures(&[(wallet_pubkey, signature)])
        //     .unwrap();

        // let blockhash = self.client.get_latest_blockhash()?;
        let signature = self
            .client
            .send_and_confirm_transaction(&signed_transaction)?;
        Ok(signature)
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Read, process::Command, thread};

    use solana_sdk::{
        commitment_config::{CommitmentConfig, CommitmentLevel},
        signature::Keypair,
        signer::Signer,
        transaction,
    };

    use super::{Config, SolanaService};

    #[test]
    fn test_create_user_pda() {
        let solana = new_solana_service();
        let wallet = Keypair::new();
        println!("WALLET: {}", wallet.to_base58_string());
        let wallet_pubkey = wallet.pubkey();
        println!("WALLET PUBKEY: {}", wallet_pubkey.to_string());

        let blockhash = solana.client.get_latest_blockhash().unwrap();
        let s = solana
            .client
            .request_airdrop_with_config(
                &wallet_pubkey,
                100,
                solana_client::rpc_config::RpcRequestAirdropConfig {
                    recent_blockhash: Some(blockhash.to_string()),
                    commitment: Some(CommitmentConfig::finalized()),
                },
            )
            .unwrap();
        println!("Airdrop signature: {}", s);

        let message = solana.create_user_pda(wallet_pubkey).unwrap();

        // Now sign the message and create a transaction
        // let signature = wallet.sign_message(message.serialize().as_slice());

        let transaction = transaction::Transaction::new(&[wallet], message, blockhash);

        let client_sign = solana
            .send_and_confirm_transaction(wallet_pubkey, transaction)
            .unwrap();
        println!("Client signature: {}", client_sign);
    }

    fn new_solana_service() -> SolanaService {
        let mut cfg = Config::default();
        cfg.commitment_config = CommitmentConfig::finalized();
        let keypair_path = "solana_program/target/deploy/solana_program-keypair.json";
        let program_keypair = solana_sdk::signer::keypair::read_keypair_file(keypair_path).unwrap();

        SolanaService::new(cfg, program_keypair)
    }
}
