mod argon;

use std::{collections::HashMap, str::FromStr};

use chacha20poly1305::Key;
use erased_serde::Serialize as DynSerialize;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum KeyDerivationError {
    HashingError { description: String },
    InvalidOptions { description: String },
    InvalidValue { description: String },
    InvalidConfigFormat,
}

pub trait DynPasswordHasher {
    fn hash_size(&self) -> usize;
    fn hash_password_into(
        &self,
        password: &[u8],
        salt: &[u8],
        hash_place: &mut [u8],
    ) -> Result<(), KeyDerivationError>;

    fn hash_password(&self, password: &[u8], salt: &[u8]) -> Result<Vec<u8>, KeyDerivationError> {
        let mut hash = vec![0; self.hash_size()];
        self.hash_password_into(password, salt, &mut hash)?;

        Ok(hash)
    }

    // PVC String
    fn option_bytes(&self) -> Vec<u8>;
    fn algorithm(&self) -> KeyDerivationAlgorithm;
}

pub trait PasswordHasherBuilder {
    fn build(&self) -> Result<Box<dyn DynPasswordHasher>, KeyDerivationError>;
    fn options(&self) -> &'static [(&'static str, &'static str)];
    fn set_option(&mut self, option: &str, value: &str) -> Result<(), KeyDerivationError>;
    fn get_option(&self, option: &str) -> Result<String, KeyDerivationError>;
}

trait PasswordHasher {
    fn options_builder() -> Box<dyn PasswordHasherBuilder>;
    fn build(options: &[u8]) -> Result<Box<dyn DynPasswordHasher>, KeyDerivationError>;
}

macro_rules! key_derivation_algorithms {
    ($($name:ident),*) => {
        #[derive(Serialize, Deserialize)]
        pub enum KeyDerivationAlgorithm {
            $(
                $name,
            )*
        }

        impl std::fmt::Display for KeyDerivationAlgorithm {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$name => write!(f, stringify!($name)),
                    )*
                }
            }
        }

        impl FromStr for KeyDerivationAlgorithm {
            type Err = KeyDerivationError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        stringify!($name) => Ok(Self::$name),
                    )*
                    otherwise => Err(
                        KeyDerivationError::InvalidValue {
                            description: format!("{} does not represent KeyDerivationAlgorithm enum.", s)
                        }
                    )
                }
            }
        }

        impl KeyDerivationAlgorithm {
            pub fn builder(&self) -> Box<dyn PasswordHasherBuilder> {
                match self {
                    $(
                        Self::$name => $name::options_builder(),
                    )*
                }
            }

            fn variants() -> Vec<Self> {
                vec![
                    $(
                        Self::$name,
                    )*
                ]
            }
        }
    };
}

use argon::{Argon2i, Argon2id};
key_derivation_algorithms! {Argon2id, Argon2i}

#[cfg(test)]
mod tests {
    use super::KeyDerivationAlgorithm;

    #[test]
    fn algorithm_enum_display_from_string() {
        for item in KeyDerivationAlgorithm::variants() {
            assert!(item.to_string().parse::<KeyDerivationAlgorithm>().is_ok());
        }
    }
}
