
use crate::communication::Task;
use std::{
    sync::{mpsc::Receiver, mpsc::Sender},
    thread::JoinHandle,
};
enum Focus {
    Menu,
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
pub fn run_user_interface(backend_connector: Sender<Task>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        // TODO
        todo!()
    })
}