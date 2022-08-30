#![warn(missing_docs)]

mod cli;
mod configuration;
mod cryptography;
mod language;
mod manager;
mod storage;
mod ui;

pub use language::TRANSLATION;

fn main() -> Result<(), String> {
    let config = configuration::ProgramConfiguration::load().map_err(|e| format!("{e:?}"))?;
    language::load_translation(&config);

    let (tx, rx) = std::sync::mpsc::channel();
    ui::run_user_interface(tx).join();

    // TODO: Kill signal
    Ok(())
}
