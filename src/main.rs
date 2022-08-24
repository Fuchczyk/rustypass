mod manager;
mod ui;

use std::{
    cell::{Ref, RefCell},
    sync::{Arc, Mutex},
};

fn main() {
    let (tx, rx) = std::sync::mpsc::channel();
    ui::run_user_interface(tx).join();

    // TODO: Kill signal
}
