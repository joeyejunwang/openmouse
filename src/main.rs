#![windows_subsystem = "windows"]

mod constants;
mod input;
mod logger;
mod rect;
mod screenshot;
mod types;
mod util;
mod window;

use logger::log_to_file;

fn main() {
    log_to_file("Application starting...");
    window::init_overlay();
}
