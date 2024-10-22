#[derive(Debug)]
pub enum Error {
    GeneralError(String),
    InvalidPubKey(String),
}

impl From<solana_sdk::pubkey::ParsePubkeyError> for Error {
    fn from(value: solana_sdk::pubkey::ParsePubkeyError) -> Self {
        Error::InvalidPubKey(value.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::GeneralError(msg) => write!(f, "GeneralError: {}", msg),
            Error::InvalidPubKey(msg) => write!(f, "InvalidPubkey: {}", msg),
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
