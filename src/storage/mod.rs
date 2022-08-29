use std::path::Path;

use crate::cryptography::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ProgramConfiguration {
    encryption_algorithm: EncryptionAlgorithm,
    text_hash_algorithm: HashAlgorithm,
    key_derivation_algorithm: KeyDerivationAlgorithm,
    nonce: Vec<u8>,
    cipher_hash: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct SaveFile {
    configuration: ProgramConfiguration,
    encrypted_data: Vec<u8>,
}

fn encrypt_database<T>(database: T, master_key: &[u8]) {
    todo!()
}
