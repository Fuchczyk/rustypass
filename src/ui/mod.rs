
use iced::{Sandbox, Text};
use iced::Settings;
use rfd::FileDialog;
use tokio::sync::mpsc::UnboundedSender;

use crate::communication::Task;
use std::{
    sync::{mpsc::Receiver, mpsc::Sender},
    thread::JoinHandle,
};

struct Rustypass {}

#[derive(Debug)]
enum UserMessage {}

impl Sandbox for Rustypass {
    type Message = UserMessage;

    fn new() -> Self {
        Self {  }
    }

    fn title(&self) -> String {
        "Rustypass".into()
    }

    fn update(&mut self, message: Self::Message) {
        
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        Text::new("Rustypass").into()
    }
}

#[derive(Debug)]
pub enum UIError {
    IOError(std::io::Error),
    ImpossibleAction,
}

impl From<std::io::Error> for UIError {
    fn from(error: std::io::Error) -> Self {
        Self::IOError(error)
    }
}

/// This should be non thread blocking function.
pub fn run_ui(backend_connector: UnboundedSender<Task>) {
    Rustypass::run(Settings::default()).unwrap()
}