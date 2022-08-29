use serde::{Deserialize, Serialize};

use crate::language::Language;
use std::ops::Deref;
use std::sync::RwLock;
use std::{collections::HashMap, path::PathBuf};

const CONFIGURATION_FOLDER_NAME: &'static str = "rustypass";

#[derive(Default, Serialize, Deserialize)]
pub struct ProgramConfiguration {
    language: Language,
}

impl ProgramConfiguration {
    pub fn get_language(&self) -> &Language {
        &self.language
    }

    const CONFIGURATION_FILE_NAME: &'static str = "config";
    pub fn load() -> Result<RwLock<Self>, ConfigurationError> {
        let mut config_path = configuration_path();
        config_path.push(Self::CONFIGURATION_FILE_NAME);

        let content = match std::fs::read_to_string(config_path) {
            Ok(cont) => cont,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return Ok(RwLock::new(ProgramConfiguration::default()));
                } else {
                    return Err(e.into());
                }
            }
        };

        Ok(RwLock::new(
            serde_json::from_str(&content).map_err(|_| ConfigurationError::DeserializationError)?,
        ))
    }
}

#[derive(Debug)]
pub enum ConfigurationError {
    IOError(std::io::Error),
    SerializationError,
    DeserializationError,
}

impl From<std::io::Error> for ConfigurationError {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(err)
    }
}

#[cfg(target_os = "linux")]
fn configuration_path() -> PathBuf {
    let mut config_folder = PathBuf::new();

    if let Ok(path) = std::env::var("XDG_CONFIG_HOME") {
        config_folder.push(path);
    } else {
        config_folder.push(std::env::var("HOME").unwrap());
        config_folder.push(".config");
    }

    config_folder
}

const CONFIGURATION_LANGUAGE_FOLDER: &'static str = "language";
pub fn read_translation(lang: &Language) -> Result<HashMap<String, String>, ConfigurationError> {
    let mut path = configuration_path();
    path.push(CONFIGURATION_LANGUAGE_FOLDER);
    path.push(lang.language_tag());

    let file_content = std::fs::read_to_string(path)?;
    serde_json::from_str(&file_content).map_err(|_| ConfigurationError::DeserializationError)
}
