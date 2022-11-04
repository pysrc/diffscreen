use fltk::button::Button;
use fltk::enums::Color;
use fltk::frame::Frame;
use fltk::input::Input;
use fltk::input::SecretInput;
use fltk::prelude::InputExt;
use fltk::window::Window;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::RwLock;

use fltk::app;
use fltk::enums;
use fltk::enums::Event;
use fltk::image;
use fltk::prelude::GroupExt;
use fltk::prelude::ImageExt;
use fltk::prelude::WidgetBase;
use fltk::prelude::WidgetExt;
use rayon::prelude::*;

use crate::bitmap;

pub fn app_run() {
    let app = app::App::default();
    let (sw, sh) = app::screen_size();
    // 开始绘制wind窗口
    let mut wind = Window::new(
        (sw / 2.0) as i32 - 170,
        (sh / 2.0) as i32 - 70,
        340,
        140,
        "Diffscreen",
    );
    wind.set_color(Color::from_rgb(255, 255, 255));
    let mut host_ipt = Input::new(80, 20, 200, 25, "HOST:");
    host_ipt.set_value("127.0.0.1:80");
    let mut pwd_ipt = SecretInput::new(80, 50, 200, 25, "PASS:");
    pwd_ipt.set_value("diffscreen");
    let mut login_btn = Button::new(200, 80, 80, 40, "Login");
    // wind窗口结束绘制
    wind.end();
    wind.show();

    login_btn.set_callback(move |_| {
        wind.hide();
        draw(host_ipt.value(), pwd_ipt.value());
    });
    app.run().unwrap();
}

enum Msg {
    Draw,
}

// 解包
#[inline]
fn depack(buffer: &[u8]) -> usize {
    ((buffer[0] as usize) << 16) | ((buffer[1] as usize) << 8) | (buffer[2] as usize)
}

fn draw(host: String, pwd: String) {
    let mut conn = TcpStream::connect(host).unwrap();
    // 认证
    let mut hasher = DefaultHasher::new();
    hasher.write(pwd.as_bytes());
    let pk = hasher.finish();
    conn.write_all(&[
        (pk >> (7 * 8)) as u8,
        (pk >> (6 * 8)) as u8,
        (pk >> (5 * 8)) as u8,
        (pk >> (4 * 8)) as u8,
        (pk >> (3 * 8)) as u8,
        (pk >> (2 * 8)) as u8,
        (pk >> (1 * 8)) as u8,
        pk as u8,
    ])
    .unwrap();
    let mut suc = [0u8];
    conn.read_exact(&mut suc).unwrap();
    if suc[0] != 1 {
        if suc[0] == 2 {
            panic!("Password error !");
        } else {
            panic!("Some error !");
        }
    }

    // 开始绘制wind2窗口
    let (sw, sh) = app::screen_size();
    let mut wind_screen = Window::default()
        .with_size((sw / 2.0) as i32, (sh / 2.0) as i32)
        .with_label("Diffscreen");
    let mut frame = Frame::default().size_of(&wind_screen);
    wind_screen.make_resizable(true);
    wind_screen.end();
    wind_screen.show();

    // 发送指令socket
    let mut txc = conn.try_clone().unwrap();
    // 接收meta信息
    let mut meta = [0u8; 4];
    if let Err(_) = conn.read_exact(&mut meta) {
        return;
    }
    let w = (((meta[0] as u16) << 8) | meta[1] as u16) as i32;
    let h = (((meta[2] as u16) << 8) | meta[3] as u16) as i32;

    let dlen = (w * h * 3) as usize;

    let work_buf = Arc::new(RwLock::new(vec![0u8; dlen]));
    let draw_work_buf = work_buf.clone();
    frame.draw(move |f| {
        if let Ok(_buf) = draw_work_buf.read() {
            unsafe {
                if let Ok(mut image) =
                    image::RgbImage::from_data2(&_buf, w, h, enums::ColorDepth::Rgb8 as i32, 0)
                {
                    image.scale(f.width(), f.height(), false, true);
                    image.draw(f.x(), f.y(), f.width(), f.height());
                }
            }
        }
    });
    let mut hooked = false;
    let mut bmap = bitmap::Bitmap::new();
    let mut cmd_buf = [0u8; 5];
    frame.handle(move |f, ev| {
        match ev {
            Event::Enter => {
                // 进入窗口
                hooked = true;
            }
            Event::Leave => {
                // 离开窗口
                hooked = false;
            }
            Event::KeyDown if hooked => {
                // 按键按下
                let key = app::event_key().bits() as u8;
                cmd_buf[0] = dscom::KEY_DOWN;
                cmd_buf[1] = key;
                if bmap.push(key) {
                    txc.write_all(&cmd_buf[..2]).unwrap();
                }
            }
            Event::Shortcut if hooked => {
                // 按键按下
                let key = app::event_key().bits() as u8;
                cmd_buf[0] = dscom::KEY_DOWN;
                cmd_buf[1] = key;
                if bmap.push(key) {
                    txc.write_all(&cmd_buf[..2]).unwrap();
                }
            }
            Event::KeyUp if hooked => {
                // 按键放开
                let key = app::event_key().bits() as u8;
                bmap.remove(key);
                cmd_buf[0] = dscom::KEY_UP;
                cmd_buf[1] = key;
                txc.write_all(&cmd_buf[..2]).unwrap();
            }
            Event::Move if hooked => {
                // 鼠标移动
                let relx = (w * app::event_x() / f.width()) as u16;
                let rely = (h * app::event_y() / f.height()) as u16;
                // MOVE xu xd yu yd
                cmd_buf[0] = dscom::MOVE;
                cmd_buf[1] = (relx >> 8) as u8;
                cmd_buf[2] = relx as u8;
                cmd_buf[3] = (rely >> 8) as u8;
                cmd_buf[4] = rely as u8;
                txc.write_all(&cmd_buf).unwrap();
            }
            Event::Push if hooked => {
                // 鼠标按下
                cmd_buf[0] = dscom::MOUSE_KEY_DOWN;
                cmd_buf[1] = app::event_key().bits() as u8;
                txc.write_all(&cmd_buf[..2]).unwrap();
            }
            Event::Released if hooked => {
                // 鼠标释放
                cmd_buf[0] = dscom::MOUSE_KEY_UP;
                cmd_buf[1] = app::event_key().bits() as u8;
                txc.write_all(&cmd_buf[..2]).unwrap();
            }
            Event::Drag if hooked => {
                // 鼠标按下移动
                let relx = (w * app::event_x() / f.width()) as u16;
                let rely = (h * app::event_y() / f.height()) as u16;
                // MOVE xu xd yu yd
                cmd_buf[0] = dscom::MOVE;
                cmd_buf[1] = (relx >> 8) as u8;
                cmd_buf[2] = relx as u8;
                cmd_buf[3] = (rely >> 8) as u8;
                cmd_buf[4] = rely as u8;
                txc.write_all(&cmd_buf).unwrap();
            }
            Event::MouseWheel if hooked => {
                // app::MouseWheel::Down;
                match app::event_dy() {
                    app::MouseWheel::Down => {
                        // 滚轮下滚
                        cmd_buf[0] = dscom::MOUSE_WHEEL_DOWN;
                        txc.write_all(&cmd_buf[..1]).unwrap();
                    }
                    app::MouseWheel::Up => {
                        // 滚轮上滚
                        cmd_buf[0] = dscom::MOUSE_WHEEL_UP;
                        txc.write_all(&cmd_buf[..1]).unwrap();
                    }
                    _ => {}
                }
            }
            _ => {
                if hooked {
                    println!("{}", ev);
                }
            }
        }
        true
    });

    let (tx, rx) = app::channel::<Msg>();

    std::thread::spawn(move || {
        let mut ctx = zstd::zstd_safe::DCtx::create();
        let mut recv_buf = Vec::<u8>::with_capacity(dlen);
        unsafe {
            recv_buf.set_len(dlen);
        }
        let mut depres_data = Vec::<u8>::with_capacity(dlen);
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
        let mut data = Vec::<u8>::with_capacity(dlen);
        ctx.decompress(&mut data, &recv_buf[..recv_len]).unwrap();
        if let Ok(mut _buf) = work_buf.write() {
            _buf.par_iter_mut()
                .zip(data.par_iter())
                .for_each(|(_d, d)| {
                    *_d = *d;
                });
        }
        tx.send(Msg::Draw);

        loop {
            if let Err(_) = conn.read_exact(&mut header) {
                return;
            }
            let recv_len = depack(&header);
            if let Err(_) = conn.read_exact(&mut recv_buf[..recv_len]) {
                return;
            }
            unsafe {
                depres_data.set_len(0);
            }
            ctx.decompress(&mut depres_data, &recv_buf[..recv_len])
                .unwrap();
            if let Ok(mut _buf) = work_buf.write() {
                data.par_iter_mut()
                    .zip(depres_data.par_iter())
                    .zip(_buf.par_iter_mut())
                    .for_each(|((_d, d), _dw)| {
                        *_d ^= *d;
                        *_dw = *_d;
                    });
            }
            tx.send(Msg::Draw);
        }
    });
    while app::wait() {
        match rx.recv() {
            Some(Msg::Draw) => {
                frame.redraw();
            }
            _ => {}
        }
    }
}
