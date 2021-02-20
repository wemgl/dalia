extern crate shellexpand;

use dalia::command::Command;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Err(e) = Command::run(args) {
        eprintln!("dalia: {}", e);
        process::exit(1);
    }
}
