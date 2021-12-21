#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod client;

fn main() {
    client::app_run();
}
