// TODO: Check if thread safe implementation can be used.
use linkme::distributed_slice;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::configuration::ProgramConfiguration;

pub const TRANSLATION: OnceCell<Translation> = OnceCell::new();
#[distributed_slice]
pub static REQUIRED_FIELDS: [&'static str] = [..];

#[derive(Serialize, Deserialize)]
pub enum Language {
    Polish,
    USEnglish,
}

impl Language {
    pub fn language_tag(&self) -> &'static str {
        match self {
            Self::Polish => "pl-PL",
            Self::USEnglish => "en-US",
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Self::USEnglish
    }
}

#[macro_export]
macro_rules! get_translation {
    ($name:expr) => {{
        use crate::language::REQUIRED_FIELDS;
        use linkme::distributed_slice;

        #[distributed_slice(REQUIRED_FIELDS)]
        static _N: &'static str = $name;

        crate::language::TRANSLATION
            .get()
            .unwrap()
            .get_translation($name)
    }};
}

// Translation lives through all of the program life so it can be leaked by Box::leak()
pub struct Translation(HashMap<String, &'static str>);
impl From<HashMap<String, &'static str>> for Translation {
    fn from(map: HashMap<String, &'static str>) -> Self {
        Translation(map)
    }
}

pub fn load_translation(conf: &RwLock<ProgramConfiguration>) -> Translation {
    let conf = conf.read().unwrap();

    match crate::configuration::read_translation(conf.get_language()) {
        Err(_e) => {
            // TODO: Log error
            default_translation().into()
        }
        Ok(map) => leak_map(map).into(),
    }
}

fn leak_map(map: HashMap<String, String>) -> HashMap<String, &'static str> {
    let mut new_map = HashMap::new();

    for (key, value) in map.into_iter() {
        new_map.insert(key, Box::leak(value.into_boxed_str()) as &'static str);
    }

    new_map
}

impl Translation {
    pub fn get_translation<'a, T>(&self, name_of_translation: T) -> &'static str
    where
        T: Into<&'static str>,
    {
        self.0
            .get(name_of_translation.into())
            .clone()
            .unwrap_or(self.0.get("TRANSLATION_NOT_FOUND").unwrap())
    }
}

macro_rules! generate_translation {
    ($($key:literal : $value:literal),*) => {
        let mut map: HashMap<String, &'static str> = HashMap::new();

        $(
            map.insert($key.into(), $value);
        )*

        map
    };
}

fn default_translation() -> HashMap<String, &'static str> {
    generate_translation! {
        "TRANSLATION_NOT_FOUND": "Translation for this field was not found. Please report this bug."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_translation_completeness() {
        let default_translation = default_translation();

        let mut uncovered_translations = Vec::new();

        for field in REQUIRED_FIELDS {
            if !default_translation.contains_key(*field) {
                uncovered_translations.push(*field);
            }
        }

        if !uncovered_translations.is_empty() {
            panic!(
                "Following fields are not covered: {:?}",
                uncovered_translations
            );
        }
    }
}
