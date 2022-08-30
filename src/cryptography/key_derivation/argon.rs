use super::{
    DynPasswordHasher, KeyDerivationAlgorithm, KeyDerivationError, PasswordHasher,
    PasswordHasherBuilder,
};
use argon2::Argon2;
use serde::{Deserialize, Serialize};

const POSSIBLE_OPTIONS: &'static [(&'static str, &'static str)] = &[
    ("Memory size", "Determines RAM usage of algorithm"),
    ("Iterations", "Sets how many passes algorithm does - the more iterations it does, the more time it takes to save/load database"),
    ("Parallelism", "Configures parallelism of the algorithm")
];

#[derive(Clone)]
struct Argon2Options {
    argon_version: argon2::Algorithm,
    argon_params: argon2::ParamsBuilder,
    memory_size: u32,
    iterations: u32,
    parallelism: u32,
}

fn deserialize_options(bytes: &[u8]) -> Result<Argon2Options, KeyDerivationError> {
    match postcard::from_bytes::<Argon2OptionsSerialization>(bytes) {
        Ok(structure) => {
            let algorithm = match structure.algorithm {
                KeyDerivationAlgorithm::Argon2i => argon2::Algorithm::Argon2i,
                KeyDerivationAlgorithm::Argon2id => argon2::Algorithm::Argon2id,
            };

            let mut builder = argon2::ParamsBuilder::default();
            builder.m_cost(structure.memory_size).map_err(|e| {
                KeyDerivationError::InvalidValue {
                    description: format!(
                        "Value of memory size ({}) was not accepted. Error = {}",
                        structure.memory_size,
                        e.to_string()
                    ),
                }
            })?;

            builder
                .t_cost(structure.iterations)
                .map_err(|e| KeyDerivationError::InvalidValue {
                    description: format!(
                        "Value of iterations ({}) was not accepted. Error = {}",
                        structure.memory_size,
                        e.to_string()
                    ),
                })?;

            builder.p_cost(structure.parallelism).map_err(|e| {
                KeyDerivationError::InvalidValue {
                    description: format!(
                        "Value of parallelism ({}) was not accepted. Error = {}",
                        structure.memory_size,
                        e.to_string()
                    ),
                }
            })?;

            Ok(Argon2Options {
                argon_version: algorithm,
                argon_params: builder,
                memory_size: structure.memory_size,
                iterations: structure.iterations,
                parallelism: structure.parallelism,
            })
        }
        Err(_e) => Err(KeyDerivationError::InvalidConfigFormat),
    }
}

#[derive(Serialize, Deserialize)]
struct Argon2OptionsSerialization {
    memory_size: u32,
    iterations: u32,
    parallelism: u32,
    algorithm: super::KeyDerivationAlgorithm,
}

impl Serialize for Argon2Options {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let algorithm = match self.argon_version {
            argon2::Algorithm::Argon2id => KeyDerivationAlgorithm::Argon2id,
            argon2::Algorithm::Argon2i => KeyDerivationAlgorithm::Argon2i,
            _ => unimplemented!(),
        };

        Argon2OptionsSerialization {
            memory_size: self.memory_size,
            iterations: self.iterations,
            parallelism: self.parallelism,
            algorithm,
        }
        .serialize(serializer)
    }
}

impl From<argon2::Error> for KeyDerivationError {
    fn from(err: argon2::Error) -> Self {
        Self::InvalidValue {
            description: err.to_string(),
        }
    }
}

impl PasswordHasherBuilder for Argon2Options {
    fn build(&self) -> Result<Box<dyn DynPasswordHasher>, KeyDerivationError> {
        let argon_params = self.argon_params.clone().params()?;

        Ok(Box::new(Argon2Wrapper::new(
            Argon2::new(
                self.argon_version.clone(),
                argon2::Version::V0x13,
                argon_params,
            ),
            self.clone(),
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

impl From<argon2::Algorithm> for KeyDerivationAlgorithm {
    fn from(algo: argon2::Algorithm) -> Self {
        match algo {
            argon2::Algorithm::Argon2i => KeyDerivationAlgorithm::Argon2i,
            argon2::Algorithm::Argon2id => KeyDerivationAlgorithm::Argon2id,
            _ => unreachable!(),
        }
    }
}

impl Argon2Options {
    fn new(argon_version: argon2::Algorithm) -> Self {
        Self {
            argon_version,
            argon_params: argon2::ParamsBuilder::default(),
            memory_size: argon2::Params::DEFAULT_M_COST,
            iterations: argon2::Params::DEFAULT_T_COST,
            parallelism: argon2::Params::DEFAULT_P_COST,
        }
    }
}

struct Argon2Wrapper {
    machine: Argon2<'static>,
    params: Box<Argon2Options>,
}

impl Argon2Wrapper {
    fn new(argon2: Argon2<'static>, options: Argon2Options) -> Self {
        Self {
            machine: argon2,
            params: Box::new(options),
        }
    }
}

pub struct Argon2id;
pub struct Argon2i;

impl DynPasswordHasher for Argon2Wrapper {
    fn option_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(&self.params).unwrap()
    }

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

    fn algorithm(&self) -> KeyDerivationAlgorithm {
        match self.params.argon_version {
            argon2::Algorithm::Argon2i => KeyDerivationAlgorithm::Argon2i,
            argon2::Algorithm::Argon2id => KeyDerivationAlgorithm::Argon2id,
            _ => unimplemented!(),
        }
    }
}

impl PasswordHasher for Argon2id {
    fn options_builder() -> Box<dyn PasswordHasherBuilder> {
        Box::new(Argon2Options::new(argon2::Algorithm::Argon2id))
    }

    fn build(options: &[u8]) -> Result<Box<dyn DynPasswordHasher>, KeyDerivationError> {
        let options = deserialize_options(options)?;
        options.build()
    }
}

impl PasswordHasher for Argon2i {
    fn options_builder() -> Box<dyn PasswordHasherBuilder> {
        Box::new(Argon2Options::new(argon2::Algorithm::Argon2i))
    }

    fn build(options: &[u8]) -> Result<Box<dyn DynPasswordHasher>, KeyDerivationError> {
        let options = deserialize_options(options)?;
        options.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let mut params = argon2::ParamsBuilder::default();
        params.t_cost(123).unwrap();
        params.m_cost(567).unwrap();
        params.p_cost(2).unwrap();

        let config = Argon2Options {
            argon_version: argon2::Algorithm::Argon2i,
            argon_params: params.clone(),
            memory_size: 567,
            iterations: 123,
            parallelism: 2,
        };

        let bytes = postcard::to_allocvec(&config).unwrap();

        let options = deserialize_options(&bytes).unwrap();
        assert_eq!(options.argon_version, argon2::Algorithm::Argon2i);
        assert_eq!(options.argon_params, params);
        assert_eq!(options.memory_size, 567);
        assert_eq!(options.iterations, 123);
        assert_eq!(options.parallelism, 2);
    }
}
