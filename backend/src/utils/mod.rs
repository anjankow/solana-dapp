pub mod jwt {
    use solana_sdk::pubkey::Pubkey;
    use uuid::Uuid;

    pub fn generate_jwt_id(pubkey: &Pubkey, nonce: &Uuid) -> String {
        let jwt_id = [nonce.as_bytes().to_vec(), pubkey.to_bytes().to_vec()].concat();
        let sha256_hash = solana_sdk::hash::hash(&jwt_id);
        sha256_hash.to_string()
    }

    pub fn jwt_id_valid(pubkey: &Pubkey, nonce: &Uuid, jwt_id: String) -> bool {
        let expected_input = [nonce.as_bytes().to_vec(), pubkey.to_bytes().to_vec()].concat();
        let expected = solana_sdk::hash::hash(&expected_input).to_string();
        expected.eq(&jwt_id)
    }
}
