#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    GeneralError(String),
    InvalidPubKey(String),
    UserNotFound,
    UserAlreadyInitialized,
    // no such transaction is known to the server
    TransactionNotFound,
    // transaction data is invalid or a signature is missing
    InvalidTransaction,
    TransactionExpired,
}

impl From<solana_sdk::pubkey::ParsePubkeyError> for Error {
    fn from(value: solana_sdk::pubkey::ParsePubkeyError) -> Self {
        Error::InvalidPubKey(value.to_string())
    }
}

impl From<solana_client::client_error::ClientError> for Error {
    fn from(value: solana_client::client_error::ClientError) -> Self {
        Error::GeneralError(value.to_string())
    }
}

impl From<borsh::io::Error> for Error {
    fn from(value: borsh::io::Error) -> Self {
        Error::GeneralError(format!("Ser/Deser failed: {}", value.to_string()))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::GeneralError(msg) => write!(f, "GeneralError: {}", msg),
            Error::InvalidPubKey(msg) => write!(f, "InvalidPubkey: {}", msg),
            Error::UserNotFound => write!(f, "UserNotFound"),
            Error::UserAlreadyInitialized => write!(f, "UserAlreadyInitialized"),
            Error::TransactionNotFound => write!(f, "TransactionNotFound"),
            Error::InvalidTransaction => write!(f, "InvalidTransaction"),
            Error::TransactionExpired => write!(f, "TransactionExpired"),
        }
    }
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }
}
