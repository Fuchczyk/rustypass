use digest::{FixedOutputReset, Update};
use sha2::{Sha256, Sha512};

trait HashGeneratorCore {
    fn new() -> Self
    where
        Self: Sized;
    fn update(&mut self, data: &[u8]);
    fn finalize(&mut self) -> Vec<u8>;
}

impl<T: Update + FixedOutputReset + Default> HashGeneratorCore for T {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self::default()
    }
    fn update(&mut self, data: &[u8]) {
        self.update(data);
    }

    fn finalize(&mut self) -> Vec<u8> {
        self.finalize_fixed_reset().to_vec()
    }
}

/// Macro to generate code used in `HashStruct`. Supplied names should be paths to structs
/// implementing `HashGeneratorCore trait - macro automatically generates enum which can be used
/// by user to select wanted hashing algorithm.
macro_rules! hash_algorithms {
    ($($name:ident),*) => {
        pub enum HashAlgorithm {
           $(
                $name,
            )*
        }

        impl std::fmt::Display for HashAlgorithm {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        $name => write!(f, stringify!($name)),
                    )*
                }
            }
        }

        pub struct HashStruct {
            hash_machine: Box<dyn HashGeneratorCore>,
        }

        impl HashStruct {
            pub fn new(algorithm: HashAlgorithm) -> Self {
                let hash_machine: Box<dyn HashGeneratorCore> = match algorithm {
                    $(
                        $name => Box::new($name::new()),
                    )*
                };

                Self { hash_machine }
            }

            pub fn update(&mut self, data: &[u8]) {
                self.hash_machine.update(data)
            }

            pub fn finalize(&mut self) -> Vec<u8> {
                self.hash_machine.finalize()
            }
        }
    };
}

hash_algorithms! {Sha256, Sha512}
