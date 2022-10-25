// #![warn(missing_docs)]

mod cli;
mod configuration;
mod cryptography;
mod language;
mod communication;
mod storage;
mod ui;
mod service;

pub use language::TRANSLATION;
use tokio::sync::mpsc::unbounded_channel;

fn main() -> Result<(), String> {
    let config = configuration::ProgramConfiguration::load().map_err(|e| format!("{e:?}"))?;
    language::load_translation(&config);

    let (tx, rx) = unbounded_channel();
    ui::run_ui(tx);

    // TODO: Kill signal
    Ok(())
}
