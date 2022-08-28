use aead::{Aead, AeadCore, KeyInit};
use generic_array::{ArrayLength, GenericArray};
use rand::RngCore;

pub enum EncryptionError {
    InvalidKeyLength { provided: usize, required: usize },
}

trait EncryptionCore {
    type NonceSize: ArrayLength<u8>;

    fn create<T: AsRef<[u8]>>(key: T) -> Result<Self, EncryptionError>
    where
        Self: Sized;
    fn random_nonce() -> GenericArray<u8, Self::NonceSize>;
    fn encrypt<T: AsRef<[u8]>, N: AsRef<[u8]>>(&self, nonce: N, plain_data: T) -> Vec<u8>;
    fn decrypt<C: AsRef<[u8]>, N: AsRef<[u8]>>(&self, nonce: N, encrypted_data: C) -> Vec<u8>;
}

impl<E: Aead + KeyInit + AeadCore> EncryptionCore for E {
    type NonceSize = <Self as AeadCore>::NonceSize;

    fn create<T: AsRef<[u8]>>(key: T) -> Result<Self, EncryptionError>
    where
        Self: Sized,
    {
        E::new_from_slice(key.as_ref()).map_err(|_| EncryptionError::InvalidKeyLength {
            provided: key.as_ref().len(),
            required: E::key_size(),
        })
    }

    fn random_nonce() -> GenericArray<u8, Self::NonceSize> {
        let mut nonce = GenericArray::default();
        rand::rngs::OsRng.fill_bytes(nonce.as_mut_slice());

        nonce
    }

    fn decrypt<C: AsRef<[u8]>, N: AsRef<[u8]>>(&self, nonce: N, encrypted_data: C) -> Vec<u8> {
        let nonce: &GenericArray<u8, <Self as AeadCore>::NonceSize> =
            GenericArray::from_slice(nonce.as_ref());

        self.decrypt(nonce, encrypted_data.as_ref())
            .expect("This should never fail.")
    }

    fn encrypt<T: AsRef<[u8]>, N: AsRef<[u8]>>(&self, nonce: N, plain_data: T) -> Vec<u8> {
        let nonce: &GenericArray<u8, <Self as AeadCore>::NonceSize> =
            GenericArray::from_slice(nonce.as_ref());

        self.encrypt(nonce, plain_data.as_ref())
            .expect("This should never fail.")
    }
}

trait DynEncryptionCore {
    fn random_nonce(&self) -> Vec<u8>;
    fn decrypt(&self, nonce: &[u8], encrypted_data: &[u8]) -> Vec<u8>;
    fn encrypt(&self, nonce: &[u8], plain_data: &[u8]) -> Vec<u8>;
}

impl<T: EncryptionCore> DynEncryptionCore for T {
    fn random_nonce(&self) -> Vec<u8> {
        Self::random_nonce().to_vec()
    }

    fn decrypt(&self, nonce: &[u8], encrypted_data: &[u8]) -> Vec<u8> {
        self.decrypt(nonce, encrypted_data)
    }

    fn encrypt(&self, nonce: &[u8], plain_data: &[u8]) -> Vec<u8> {
        self.encrypt(nonce, plain_data)
    }
}

macro_rules! encryption_algorithms {
    ($($name:ident),*) => {
        type EncryptionNonce = Vec<u8>;

        pub enum EncryptionAlgorithm {
            $(
                $name,
            )*
        }

        impl std::fmt::Display for EncryptionAlgorithm {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        $name => write!(f, stringify!($name)),
                    )*
                }
            }
        }

        pub struct EncryptionStruct {
            encryption_machine: Box<dyn DynEncryptionCore>
        }

        impl EncryptionStruct {
            pub fn new<T: AsRef<[u8]>>(
                algorithm: EncryptionAlgorithm,
                key: T
            )
            -> Result<Self, EncryptionError> {
                let encryption_machine: Box<dyn DynEncryptionCore> = match algorithm {
                    $(
                        EncryptionAlgorithm::$name => Box::new($name::create(key)?),
                    )*
                };

                Ok(Self { encryption_machine })
            }

            pub fn encrypt<T: AsRef<[u8]>>(
                &self,
                plain_data: T
            ) -> (Vec<u8>, EncryptionNonce) {
                let nonce = self.encryption_machine.random_nonce();
                let encrypted_data = self.encryption_machine.encrypt(&nonce, plain_data.as_ref());

                (encrypted_data, nonce)
            }

            pub fn decrypt<T, N>(
                &self,
                encrypted_data: T,
                nonce: N
            ) -> Vec<u8>
            where
                T: AsRef<[u8]>,
                N: AsRef<[u8]>
            {
                self.encryption_machine
                    .decrypt(nonce.as_ref(), encrypted_data.as_ref())
            }
        }
    };
}

use aes_gcm_siv::Aes256GcmSiv;
use chacha20poly1305::ChaCha20Poly1305;
encryption_algorithms!(Aes256GcmSiv, ChaCha20Poly1305);
