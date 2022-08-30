use std::{path::Path, pin::Pin};

use crate::cryptography::*;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Serialize, Deserialize)]
struct ProgramConfiguration {
    encryption_algorithm: EncryptionAlgorithm,
    text_hash_algorithm: HashAlgorithm,
    key_derivation_algorithm: KeyDerivationAlgorithm,
    key_derivation_options: Vec<u8>,
    nonce: Vec<u8>,
    cipher_hash: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct SaveFile {
    configuration: ProgramConfiguration,
    encrypted_data: Vec<u8>,
}

struct SafeBuffer;

impl AsRef<[u8]> for SafeBuffer {
    fn as_ref(&self) -> &[u8] {
        todo!();
    }
}

fn encrypt_database(
    mut database: Pin<Box<Vec<u8>>>,
    mut hash_algorithm: HashStruct,
    key_deriver: Box<dyn DynPasswordHasher>,
    encryptor: EncryptionStruct,
) -> SaveFile {
    let (encrypted_data, nonce) = encryptor.encrypt(database.as_slice());
    database.zeroize();
    hash_algorithm.update(&encrypted_data);

    let cipher_hash = hash_algorithm.finalize();
    let key_derivation_options = key_deriver.option_bytes();
    let key_derivation_algorithm = key_deriver.algorithm();
    let encryption_algorithm = encryptor.algorithm();
    let text_hash_algorithm = hash_algorithm.algorithm();

    let configuration = ProgramConfiguration {
        encryption_algorithm,
        text_hash_algorithm,
        key_derivation_algorithm,
        key_derivation_options,
        nonce,
        cipher_hash,
    };

    SaveFile {
        configuration,
        encrypted_data,
    }
}
