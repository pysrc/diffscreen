mod key_mouse;
mod screen;
mod server;
mod config;
mod util;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    // defalut port
    let mut port = 9980;
    if args.len() >= 2 {
        port = args[1].parse::<u16>().unwrap();
    }

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(3)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            server::run(port).await;
        });
}
