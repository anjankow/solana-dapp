use crate::instruction::{InitializeInstructionData, ProgramInstruction};
use solana_program::account_info::next_account_info;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction;
use solana_program::system_program;

pub const USER_PDA_SIZE: u64 = 1024;
pub const USER_PDA_SEED_PREFIX: &[u8] = b"user";

/// Instruction processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instr = crate::instruction::ProgramInstruction::unpack(input)?;
    match instr {
        ProgramInstruction::Initialize(data) => process_initialize(program_id, accounts, data),
        ProgramInstruction::CloseAccount => process_close_account(program_id, accounts),
    }
}

pub fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: InitializeInstructionData,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer = next_account_info(account_info_iter)?;
    validate_payer_account(payer)?;
    let pda = next_account_info(account_info_iter)?;
    if !pda.is_writable {
        return Err(ProgramError::Immutable);
    }
    // System program needs to come from the outside
    let system_program = next_account_info(account_info_iter)?;

    // Used to uniquely identify this PDA among others.
    let pda_seed = &[
        /* passed to find_program_address */ USER_PDA_SEED_PREFIX,
        /* passed to find_program_address */ payer.key.as_ref(),
        /* pda_bump_seed calculated by find_program_address */
        &[input.pda_bump_seed],
    ];

    // Invoke the system program to create an account while virtually
    // signing with the vault PDA, which is owned by this caller program.
    solana_program::program::invoke_signed(
        &system_instruction::create_account(
            payer.key,
            pda.key,
            input.lamports,
            USER_PDA_SIZE,
            program_id,
        ),
        &[payer.clone(), pda.clone(), system_program.clone()],
        &[pda_seed],
    )
}

fn validate_payer_account(payer: &AccountInfo) -> Result<(), ProgramError> {
    if !payer.is_signer || payer.signer_key().is_none() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !payer.is_writable {
        return Err(ProgramError::Immutable);
    }
    Ok(())
}

fn process_close_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer = next_account_info(account_info_iter)?;
    validate_payer_account(payer)?;
    let pda = next_account_info(account_info_iter)?;
    if !pda.is_writable {
        return Err(ProgramError::Immutable);
    }

    let source_account_info = pda;
    let dest_account_info = payer;

    let dest_starting_lamports = dest_account_info.lamports();
    **dest_account_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(source_account_info.lamports())
        .or(Some(u64::MAX))
        .expect("u64::MAX should never overflow u64");
    **source_account_info.lamports.borrow_mut() = 0;

    source_account_info.assign(&system_program::ID);
    source_account_info.realloc(0, false).map_err(Into::into)
}
