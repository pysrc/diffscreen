use std::io::Read;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::RwLock;

use fltk::app;
use fltk::enums;
use fltk::frame;
use fltk::image;
use fltk::prelude::GroupExt;
use fltk::prelude::ImageExt;
use fltk::prelude::WidgetBase;
use fltk::prelude::WidgetExt;
use fltk::window;

pub fn run(host: String) {
    let conn = TcpStream::connect(host).unwrap();
    let sc = conn.try_clone().unwrap();
    let th2 = std::thread::spawn(move || {
        app_run(sc);
    });
    th2.join().unwrap();
}

fn app_run(mut conn: TcpStream) {
    // 接收meta信息
    let mut meta = [0u8; 4];
    if let Err(_) = conn.read_exact(&mut meta) {
        return;
    }
    let w = (((meta[0] as u16) << 8) | meta[1] as u16) as i32;
    let h = (((meta[2] as u16) << 8) | meta[3] as u16) as i32;

    let dlen = (w * h * 3) as usize;
    // 解包
    let depack = |buffer: &[u8]| -> usize {
        ((buffer[0] as usize) << 16) | ((buffer[1] as usize) << 8) | (buffer[2] as usize)
    };

    // 收到的数据
    let mut recv_buf = Vec::<u8>::with_capacity(dlen);
    let mut data = Vec::<u8>::with_capacity(dlen);
    let mut depres_data = Vec::<u8>::with_capacity(dlen);
    unsafe {
        recv_buf.set_len(dlen);
        data.set_len(dlen);
        depres_data.set_len(dlen);
    }
    // 接收第一帧数据
    let mut header = [0u8; 3];
    if let Err(_) = conn.read_exact(&mut header) {
        return;
    }
    let recv_len = depack(&header);
    if let Err(e) = conn.read_exact(&mut recv_buf[..recv_len]) {
        println!("error {}", e);
        return;
    }
    dscom::decompress(&recv_buf[..recv_len], &mut data);

    let app = app::App::default();
    let mut wind = window::Window::default().with_size(500, 300);
    let mut frame = frame::Frame::default().size_of(&wind);
    wind.make_resizable(true);
    wind.end();
    wind.show();

    let _data = Arc::new(RwLock::new(data));
    let arc_data1 = Arc::clone(&_data);
    let arc_data2 = Arc::clone(&_data);

    std::thread::spawn(move || {
        let mut header = [0u8; 3];
        // 接收图像
        loop {
            if let Err(_) = conn.read_exact(&mut header) {
                return;
            }
            let recv_len = depack(&header);
            if let Err(e) = conn.read_exact(&mut recv_buf[..recv_len]) {
                println!("error {}", e);
                return;
            }
            dscom::decompress(&recv_buf[..recv_len], &mut depres_data);
            match arc_data1.write() {
                Ok(mut _data) => {
                    for (i, v) in depres_data.iter().enumerate() {
                        _data[i] = _data[i] ^ *v;
                    }
                }
                Err(_) => {}
            }
        }
    });
    frame.draw(move |f| match arc_data2.read() {
        Ok(data) => {
            let d = &data;
            let mut image = image::RgbImage::new(d, w, h, enums::ColorDepth::Rgb8).unwrap();
            image.scale(f.width(), f.height(), false, true);
            image.draw(f.x(), f.y(), f.width(), f.height());
        }
        Err(_) => {}
    });
    while app.wait() {
        frame.redraw();
        app::sleep(0.005);
    }
}
