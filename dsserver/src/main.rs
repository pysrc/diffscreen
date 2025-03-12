mod key_mouse;
mod screen;
mod server;
mod convert;
fn main() {
    let args: Vec<String> = std::env::args().collect();

    // defalut password
    let mut pwd = String::from("diffscreen");
    if args.len() >= 2 {
        pwd = args[1].clone();
    }

    // defalut port
    let mut port = 38971;
    if args.len() >= 3 {
        port = args[2].parse::<u16>().unwrap();
    }

    // run forever
    server::run(port, pwd);
}
