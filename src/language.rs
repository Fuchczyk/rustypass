// TODO: Check if thread safe implementation can be used.
use crate::configuration::ProgramConfiguration;
use linkme::distributed_slice;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Contains program's translation (map of strings). Should be initialized by [`load_translation`]
/// function after initializing program's configuration (see documentation of [`configuration module`]).
/// Change in translation configuration has effect only after program restart.
///
/// [`load_translation`]: crate::language::load_translation
/// [`configuration module`]: crate::configuration
pub const TRANSLATION: OnceCell<Translation> = OnceCell::new();

/// Static variable containing all keys which should be available in [`TRANSLATION`].
/// It is generated at the time of compilation by using [`get_translation!`] macro.
///
/// # Examples
/// ```
/// let translation = get_translation!("TEST_FEATURE");
/// assert!(REQUIRED_FIELDS.contains("TEST_FEATURE"));
/// ```
///
/// [`TRANSLATION`]: crate::language::TRANSLATION
/// [`get_translation!`]: crate::language::get_translation
#[distributed_slice]
pub static REQUIRED_FIELDS: [&'static str] = [..];

/// Represent all languages available in program.
#[derive(Serialize, Deserialize)]
pub enum Language {
    /// Polish language
    Polish,
    /// United States English
    USEnglish,
}

impl Language {
    /// Returns IETF language tag representation (used for filenames of language files).
    ///
    /// # Examples
    /// ```
    /// let language = Language::Polish;
    /// assert_eq!("pl-PL", language.language_tag());
    /// ```
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

/// Macro should be used to retrieve sentence from program's translation.
/// Programmer should initialize program's dictionary in [`TRANSLATION`]
/// by running [`load_translation`] function before first usage of this macro.
///
/// # Examples
/// ```
/// // We are going to retrieve string,
/// // which should be displayed if user's password was wrong.
///
/// let pop_up_string: &'static str = get_translation!("USER_WRONG_PASSWORD");
/// // Example translation
/// assert_eq!(pop_up_string, "Invalid password");
/// ```
///
/// [`TRANSLATION`]: crate::language::TRANSLATION
/// [`get_translation!`]: crate::language::get_translation
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

/// Structure represents program's translation and is basically wrapper
/// around [`HashMap`] containing strings.
///
/// [`HashMap`]: std::collections::HashMap
pub struct Translation(HashMap<String, &'static str>);
impl From<HashMap<String, &'static str>> for Translation {
    fn from(map: HashMap<String, &'static str>) -> Self {
        Translation(map)
    }
}

/// Loads program's translation into [`TRANSLATION`] variable.
/// Can be used only once thanks to [`OnceCell`] implementation.
///
/// # Arguments
///
/// * `conf` - A lock reference providing non-mutable access to program's configuration.
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

/// Leaks values of map so they become static strings. It significantly makes usage of
/// all language functions easier. Leaking process does not reflect on program's memory
/// usage because [`TRANSLATION`] is initialized only once, so resulting HashMap
/// is alive during entire execution of program.
///
/// # Arguments
///
/// * `map` - A Map with String values, which will be used to initialize [`TRANSLATION`]. Using
///     this function with other maps will likely result in memory leak.
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
