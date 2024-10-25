use solana_program::pubkey::Pubkey;
use solana_program::{decode_error::DecodeError, msg, program_error::PrintProgramError};
use std::fmt::Formatter;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Error {
    cause: ErrorCause,
    account_key: Option<Pubkey>,
    message: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Clone, num_enum::FromPrimitive)]
#[repr(u8)]
pub enum ErrorCause {
    #[num_enum(default)]
    GeneralError,
    InvalidPubKey,
    AccountNotFound,
    AccountAlreadyInitialized,
}

impl Error {
    fn get_error_msg(&self) -> String {
        let get_account_key = || -> String {
            self.account_key
                .map(|k| k.to_string())
                .unwrap_or("<NO DATA>".to_string())
        };

        match &self.cause {
            ErrorCause::GeneralError => {
                format!(
                    "GeneralError: {}",
                    self.message.clone().unwrap_or("Unknown".to_string())
                )
            }
            ErrorCause::InvalidPubKey => {
                format!(
                    "Account: {} | InvalidPubKey: {}",
                    get_account_key(),
                    self.message.clone().unwrap_or("Invalid".to_string())
                )
            }
            ErrorCause::AccountNotFound => {
                format!("Account: {} | AccountNotFound", get_account_key(),)
            }
            ErrorCause::AccountAlreadyInitialized => {
                format!("Account: {} | AccountAlreadyInitialized", get_account_key(),)
            }
        }
    }
}

impl num_traits::FromPrimitive for Error {
    fn from_i64(n: i64) -> Option<Self> {
        Some(Error {
            cause: ErrorCause::from(u8::from_i64(n).unwrap_or(0)),
            account_key: None,
            message: None,
        })
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(Error {
            cause: ErrorCause::from(u8::from_u64(n).unwrap_or(0)),
            account_key: None,
            message: None,
        })
    }
}

impl<T> DecodeError<T> for Error {
    fn type_of() -> &'static str {
        "MyHugeError"
    }
}

impl PrintProgramError for Error {
    fn print<E>(&self)
    where
        E: 'static
            + std::error::Error
            + DecodeError<E>
            + PrintProgramError
            + num_traits::FromPrimitive,
    {
        msg!("{}", self.get_error_msg())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.get_error_msg())
    }
}
