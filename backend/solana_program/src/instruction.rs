use {
    borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        msg,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    std::{ffi::IntoStringError, mem::size_of},
};

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
pub struct InitializeInstructionData {
    pub lamports: u64, // to pay for rent of the PDA
    pub pda_bump_seed: u8,
}

/// Instructions supported by the program
#[derive(Clone, Debug, PartialEq)]
pub enum ProgramInstruction {
    /// Create a PDA for the user.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writable, signer]` User account, PDA owner.
    /// 1. `[writable]` PDA found with Pubkey::find_program_address for this user.
    /// 2. `[]` System program used to create a new account.
    Initialize(InitializeInstructionData),

    /// Close the provided PDA account, draining lamports to recipient
    /// account
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writable, signer]` User account, PDA owner.
    /// 1. `[writable]` User's PDA
    CloseAccount,
}

#[derive(Clone, Debug, PartialEq, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[repr(u8)]
enum InstructionTag {
    #[num_enum(default)]
    Invalid,
    Initialize,
    CloseAccount,
}

impl ProgramInstruction {
    /// Unpacks a byte buffer into a [ProgramInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, mut data) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        let tag = InstructionTag::from(tag);
        Ok(match tag {
            InstructionTag::Initialize => {
                let instruction_data =
                    InitializeInstructionData::deserialize(&mut data).map_err(|e| {
                        msg!("Failed to deserialize instruction body: {}", e);
                        return ProgramError::InvalidInstructionData;
                    })?;
                Self::Initialize(instruction_data)
            }
            InstructionTag::CloseAccount => Self::CloseAccount,

            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }

    /// Packs a [ProgramInstruction] into a byte buffer.
    pub fn pack(&self) -> Result<Vec<u8>, borsh::io::Error> {
        let mut buf: Vec<u8> = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::Initialize(data) => {
                buf.push(InstructionTag::Initialize.into());
                data.serialize(&mut buf)?;
            }
            Self::CloseAccount => buf.push(InstructionTag::CloseAccount.into()),
        };
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_initialize() {
        let instruction = ProgramInstruction::Initialize(InitializeInstructionData {
            lamports: 3213,
            pda_bump_seed: 255,
        });

        let packed = instruction.pack().unwrap();
        assert_eq!(1, *packed.get(0).unwrap());
        let unpacked = ProgramInstruction::unpack(&packed).unwrap();
        assert_eq!(instruction, unpacked);
    }
}
