mod ui;

use std::{
    cell::{Ref, RefCell},
    sync::{Arc, Mutex},
};

use ui::user_interface;

#[tokio::main]
async fn main() {
    user_interface().await.unwrap();
}
