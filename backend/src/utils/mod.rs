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

pub mod bincode {

    use bincode::{ErrorKind, Options};
    use serde::{Deserialize, Serialize};

    pub fn deserialize<T>(input: Vec<u8>) -> Result<T, Box<ErrorKind>>
    where
        T: for<'a> Deserialize<'a>,
    {
        bincode::options()
            .with_little_endian()
            .deserialize::<T>(&input)
            .inspect_err(|e| {
                println!("Failed to deserialize input: {}", e);
            })
    }

    pub fn serialize<T>(input: &T) -> Result<Vec<u8>, Box<ErrorKind>>
    where
        T: Serialize,
    {
        bincode::options()
            .with_little_endian()
            .serialize::<T>(&input)
            .inspect_err(|e| {
                println!("Failed to deserialize input: {}", e);
            })
    }
}
