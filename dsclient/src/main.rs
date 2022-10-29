// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod bitmap;
mod client;
mod util;

fn main() {
    client::app_run();
}
