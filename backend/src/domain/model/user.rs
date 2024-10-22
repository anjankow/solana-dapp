#[derive(Clone, PartialEq)]
pub struct User {
    pub pubkey: solana_sdk::pubkey::Pubkey,
    pub nonce: u64,
    pub username: String,
}
