use super::{DynPasswordHasher, KeyDerivationError, PasswordHasher, PasswordHasherBuilder};
use argon2::Argon2;

const POSSIBLE_OPTIONS: &'static [(&'static str, &'static str)] = &[
    ("Memory size", "Determines RAM usage of algorithm"),
    ("Iterations", "Sets how many passes algorithm does - the more iterations it does, the more time it takes to save/load database"),
    ("Parallelism", "Configures parallelism of the algorithm")
];

struct Argon2Options {
    argon_version: argon2::Algorithm,
    argon_params: argon2::Params,
    memory_size: u32,
    iterations: u32,
    parallelism: u32,
}

impl PasswordHasherBuilder for Argon2Options {
    fn build(self) -> Box<dyn DynPasswordHasher> {
        Box::new(Argon2Wrapper::new(Argon2::new(
            self.argon_version,
            argon2::Version::V0x13,
            self.argon_params,
        )))
    }

    fn get_option(&self, option: &str) -> Result<String, KeyDerivationError> {
        match option.to_ascii_lowercase().as_str() {
            "memory size" => Ok(self.memory_size.to_string()),
            "iterations" => Ok(self.memory_size.to_string()),
            "parallelism" => Ok(self.parallelism.to_string()),
            _ => Err(KeyDerivationError::InvalidOptions {
                description: format!("Field {} was not found.", option),
            }),
        }
    }

    fn set_option(&mut self, option: &str, value: &str) -> Result<(), KeyDerivationError> {
        match option {
            "memory size" => {
                let memory_size: u32 =
                    value
                        .parse()
                        .map_err(|_| KeyDerivationError::InvalidValue {
                            description: format!(
                                "Unable to parse {} into unsigned 32-bit number.",
                                value
                            ),
                        })?;
                self.memory_size = memory_size;
            }
            "iterations" => {
                let iterations: u32 =
                    value
                        .parse()
                        .map_err(|_| KeyDerivationError::InvalidValue {
                            description: format!(
                                "Unable to parse {} into unsigned 32-bit number.",
                                value
                            ),
                        })?;
                self.iterations = iterations;
            }
            "parallelism" => {
                let parallelism: u32 =
                    value
                        .parse()
                        .map_err(|_| KeyDerivationError::InvalidValue {
                            description: format!(
                                "Unable to parse {} into unsigned 32-bit number.",
                                value
                            ),
                        })?;
                self.parallelism = parallelism;
            }
            _ => Err(KeyDerivationError::InvalidOptions {
                description: format!("Field {} was not found.", option),
            })?,
        }

        Ok(())
    }

    fn options(&self) -> &'static [(&'static str, &'static str)] {
        POSSIBLE_OPTIONS
    }
}

impl Argon2Options {
    fn new(argon_version: argon2::Algorithm) -> Self {
        Self {
            argon_version,
            argon_params: argon2::Params::default(),
            memory_size: argon2::Params::DEFAULT_M_COST,
            iterations: argon2::Params::DEFAULT_T_COST,
            parallelism: argon2::Params::DEFAULT_P_COST,
        }
    }
}

struct Argon2Wrapper {
    machine: Argon2<'static>,
}

impl Argon2Wrapper {
    fn new(argon2: Argon2<'static>) -> Self {
        Self { machine: argon2 }
    }
}

pub struct Argon2id;
pub struct Argon2i;

impl DynPasswordHasher for Argon2Wrapper {
    fn hash_password_into(
        &self,
        password: &[u8],
        salt: &[u8],
        hash_place: &mut [u8],
    ) -> Result<(), KeyDerivationError> {
        self.machine
            .hash_password_into(password, salt, hash_place)
            .map_err(|error| KeyDerivationError::HashingError {
                description: error.to_string(),
            })
    }

    fn hash_size(&self) -> usize {
        self.machine
            .params()
            .output_len()
            .unwrap_or(argon2::Params::DEFAULT_OUTPUT_LEN)
    }
}

impl PasswordHasher for Argon2id {
    fn options_builder() -> Box<dyn PasswordHasherBuilder> {
        Box::new(Argon2Options::new(argon2::Algorithm::Argon2id))
    }
}

impl PasswordHasher for Argon2i {
    fn options_builder() -> Box<dyn PasswordHasherBuilder> {
        Box::new(Argon2Options::new(argon2::Algorithm::Argon2i))
    }
}
