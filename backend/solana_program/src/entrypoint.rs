use {
    crate::error::Error,
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, program_error::PrintProgramError,
        pubkey::Pubkey,
    },
};

solana_program::entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) =
        crate::processor::process_instruction(program_id, accounts, instruction_data)
    {
        error.print::<Error>();
        return Err(error);
    };
    Ok(())
}
