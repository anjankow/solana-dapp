mod instruction;
use std::{
    sync::Arc,
    time::{self, SystemTime},
};

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
use uuid::Uuid;

use crate::{
    domain::{
        error::Error,
        model::{TransactionCallback, TransactionRecord, TransactionToSign},
    },
    repo::solana::Repo,
};

const USER_PDA_PREFIX: &[u8] = b"user";

#[derive(Clone)]
pub struct Config {
    user_pda_size: usize,
    rpc_client_url: String,
    commitment_config: CommitmentConfig,
    timeout_sec: u64,
    transaction_validity_sec: u32,
}

impl Config {
    pub fn default() -> Config {
        Config {
            user_pda_size: 1024,
            rpc_client_url: "http://localhost:8899".to_string(),
            commitment_config: CommitmentConfig::confirmed(),
            timeout_sec: 5,
            transaction_validity_sec: 3600,
        }
    }
}

#[derive(Clone)]
pub struct SolanaService {
    cfg: Config,
    program: Arc<Keypair>,
    client: Arc<solana_client::rpc_client::RpcClient>,
    repo: Arc<Repo>,
}

impl SolanaService {
    pub fn new(cfg: Config, program: Keypair, repo: Repo) -> SolanaService {
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
            repo: Arc::new(repo),
        }
    }

    pub fn get_user_pda(&self, wallet_pubkey: &Pubkey) -> Pubkey {
        let seeds = &[USER_PDA_PREFIX, wallet_pubkey.as_ref()];
        let (pda_pubkey, _) = Pubkey::find_program_address(seeds, &self.program.pubkey());
        pda_pubkey
    }

    fn get_validate_transaction_record(
        &self,
        pubkey: &Pubkey,
        transaction_id: Uuid,
    ) -> Result<TransactionRecord, Error> {
        let transaction_record = self.repo.get_transaction_record(transaction_id)?;
        // validate the received transaction
        if transaction_record.valid_until.lt(&SystemTime::now()) {
            return Err(Error::TransactionExpired);
        }
        if transaction_record.pubkey.ne(&pubkey) {
            return Err(Error::InvalidTransaction(
                "Invalid transaction ID".to_string(),
            ));
        }
        return Ok(transaction_record);
    }

    pub fn create_user_pda(&self, wallet_pubkey: &Pubkey) -> Result<TransactionToSign, Error> {
        let account_size = self.cfg.user_pda_size;

        let lamports = self
            .client
            .get_minimum_balance_for_rent_exemption(account_size)?;

        // Derive the PDA from the payer account, a string representing the unique
        // purpose of the account, and the address of our on-chain program.
        let seeds = &[USER_PDA_PREFIX, wallet_pubkey.as_ref()];
        let (pda_pubkey, pda_bump_seed) =
            Pubkey::find_program_address(seeds, &self.program.pubkey());
        // println!("PDA pubkey: {}", pda_pubkey);

        let instr_data =
            instruction::ProgramInstruction::Initialize(instruction::InitializeInstructionData {
                lamports,
                pda_bump_seed,
            })
            .pack()?;

        // The accounts required by both our on-chain program and the system program's
        // `create_account` instruction, including the vault's address.
        let accounts = vec![
            AccountMeta::new(*wallet_pubkey, true /* is_signer */),
            AccountMeta::new(pda_pubkey, false),
            AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
        ];

        // Create the instruction by serializing our instruction data via borsh
        let instruction = Instruction::new_with_bytes(self.program.pubkey(), &instr_data, accounts);
        let message = Message::new(&[instruction], Some(&wallet_pubkey));

        // Save the transaction record in repo
        let mut transaction_record = TransactionRecord {
            id: Uuid::nil(),
            message_hash: message.hash(),
            pubkey: *wallet_pubkey,
            valid_until: SystemTime::now()
                .checked_add(std::time::Duration::from_secs(
                    self.cfg.transaction_validity_sec as u64,
                ))
                .expect("Time adding should never exceed the bounds here"),
            callback: Some(TransactionCallback::RegisterComplete),
            client_signature: None,
        };
        self.repo.add_transaction_record(&mut transaction_record)?;

        Ok(TransactionToSign {
            message,
            transaction_id: transaction_record.id,
            valid_until: transaction_record.valid_until,
        })
    }

    pub fn execute_transaction(
        &self,
        wallet_pubkey: &Pubkey,
        transaction_id: Uuid,
        signed_transaction: Transaction,
    ) -> Result<(), Error> {
        let mut transaction_record =
            self.get_validate_transaction_record(&wallet_pubkey, transaction_id)?;

        // Make sure that the transaction is valid and signed by this user.
        if signed_transaction
            .message()
            .hash()
            .ne(&transaction_record.message_hash)
        {
            return Err(Error::InvalidTransaction(
                "Transaction message hash doesn't match the original".to_string(),
            ));
        }
        signed_transaction.verify()?;
        let pos = signed_transaction.get_signing_keypair_positions(&[*wallet_pubkey])?;
        if pos.len() == 0 || pos.get(0).is_none() {
            return Err(Error::InvalidTransaction(
                "The transaction is not signed by this public key".to_string(),
            ));
        }

        // All checks passed, now we can submit the transaction.
        let signature = self
            .client
            .send_and_confirm_transaction(&signed_transaction)?;
        transaction_record.client_signature = Some(signature);
        self.repo.update_transaction_record(&transaction_record)?;

        Ok(())
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

    use crate::{domain::model::TransactionCallback, repo};

    use super::{Config, SolanaService};

    #[test]
    fn test_create_user_pda() {
        let solana = new_solana_service();
        let wallet = Keypair::new();
        println!("WALLET: {}", wallet.to_base58_string());
        let wallet_pubkey = wallet.pubkey();
        println!("WALLET PUBKEY: {}", wallet_pubkey.to_string());

        Command::new("solana")
            .arg("airdrop")
            .arg("--commitment")
            .arg("finalized")
            .arg("1")
            .arg(format!("{}", wallet_pubkey.to_string()))
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        let to_sign = solana.create_user_pda(&wallet_pubkey).unwrap();

        let blockhash = solana.client.get_latest_blockhash().unwrap();
        let transaction = transaction::Transaction::new(&[wallet], to_sign.message, blockhash);

        solana
            .execute_transaction(&wallet_pubkey, to_sign.transaction_id, transaction)
            .unwrap();

        let transaction_record = solana
            .get_validate_transaction_record(&wallet_pubkey, to_sign.transaction_id)
            .unwrap();
        assert!(transaction_record.client_signature.is_some());
        println!(
            "Client signature: {}",
            transaction_record.client_signature.unwrap()
        );
    }

    fn new_solana_service() -> SolanaService {
        let mut cfg = Config::default();
        cfg.commitment_config = CommitmentConfig::finalized();
        let keypair_path = "solana_program/target/deploy/solana_program-keypair.json";
        let program_keypair = solana_sdk::signer::keypair::read_keypair_file(keypair_path).unwrap();
        let repo = repo::solana::Repo::new();
        SolanaService::new(cfg, program_keypair, repo)
    }
}
