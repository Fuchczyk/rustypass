mod argon;

use serde::{Deserialize, Serialize};

pub enum KeyDerivationError {
    HashingError { description: String },
    InvalidOptions { description: String },
    InvalidValue { description: String },
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
}

pub trait PasswordHasherBuilder {
    fn build(self) -> Box<dyn DynPasswordHasher>;
    fn options(&self) -> &'static [(&'static str, &'static str)];
    fn set_option(&mut self, option: &str, value: &str) -> Result<(), KeyDerivationError>;
    fn get_option(&self, option: &str) -> Result<String, KeyDerivationError>;
}

trait PasswordHasher {
    fn options_builder() -> Box<dyn PasswordHasherBuilder>;
}

macro_rules! key_derivation_algorithms {
    ($($name:ident),*) => {
        #[derive(Serialize, Deserialize)]
        pub enum KeyDerivationAlgorithm {
            $(
                $name,
            )*
        }

        impl KeyDerivationAlgorithm {
            pub fn builder(&self) -> Box<dyn PasswordHasherBuilder> {
                match self {
                    $(
                        Self::$name => $name::options_builder(),
                    )*
                }
            }
        }
    };
}

use argon::{Argon2i, Argon2id};
key_derivation_algorithms! {Argon2id, Argon2i}
