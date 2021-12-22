#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod client;
mod bitmap;

fn main() {
    client::app_run();
}
