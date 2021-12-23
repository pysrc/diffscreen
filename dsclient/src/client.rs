use fltk::button::Button;
use fltk::enums::Color;
use fltk::frame::Frame;
use fltk::input::Input;
use fltk::input::SecretInput;
use fltk::prelude::InputExt;
use fltk::window::Window;
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::rc::Rc;
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
        draw(app, host_ipt.value(), pwd_ipt.value());
    });
    app.run().unwrap();
}

fn draw(app: app::App, host: String, pwd: String) {
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
        // 密码错误
        return;
    }
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
    // 解包
    let depack = |buffer: &[u8]| -> usize {
        ((buffer[0] as usize) << 16) | ((buffer[1] as usize) << 8) | (buffer[2] as usize)
    };

    // 收到的数据
    let data = Vec::<u8>::with_capacity(dlen);
    let _data = Arc::new(RwLock::new(data));
    let arc_data1 = Arc::clone(&_data);
    let arc_data2 = Arc::clone(&_data);

    std::thread::spawn(move || {
        // let mut header = [0u8; 3];
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
        match arc_data1.write() {
            Ok(mut _data) => {
                dscom::decompress(&recv_buf[..recv_len], &mut _data);
            }
            Err(_) => {}
        }

        // 接收图像
        loop {
            if let Err(_) = conn.read_exact(&mut header) {
                app::quit();
                return;
            }
            let recv_len = depack(&header);
            if let Err(_) = conn.read_exact(&mut recv_buf[..recv_len]) {
                app::quit();
                return;
            }
            dscom::decompress(&recv_buf[..recv_len], &mut depres_data);
            match arc_data1.write() {
                Ok(mut _data) => {
                    let mut i = 0;
                    while i < dlen {
                        _data[i] ^= depres_data[i];
                        i += 1;
                    }
                }
                Err(_) => {}
            }
        }
    });
    // 开始绘制wind2窗口
    let (sw, sh) = app::screen_size();
    let mut wind_screen = Window::default().with_size((sw/2.0) as i32, (sh/2.0) as i32);
    let mut frame = Frame::default().size_of(&wind_screen);
    wind_screen.make_resizable(true);
    wind_screen.end();
    wind_screen.show();

    frame.draw(move |f| match arc_data2.read() {
        Ok(data) => {
            let d = &data;
            if let Ok(mut image) = image::RgbImage::new(d, w, h, enums::ColorDepth::Rgb8) {
                image.scale(f.width(), f.height(), false, true);
                image.draw(f.x(), f.y(), f.width(), f.height());
            }
        }
        Err(_) => {}
    });
    let hooked = Rc::new(RefCell::new(false));
    let press_record = Rc::new(RefCell::new(bitmap::Bitmap::new()));
    frame.handle(move |f, ev| {
        let mut hk = hooked.borrow_mut();
        let mut bmap = press_record.borrow_mut();
        match ev {
            Event::Enter => {
                // 进入窗口
                *hk = true;
            }
            Event::Leave => {
                // 离开窗口
                *hk = false;
            }
            Event::KeyDown if *hk => {
                // 按键按下
                let key = (app::event_key().bits() & 0xff) as u8;
                if bmap.push(key) {
                    txc.write_all(&[dscom::KEY_DOWN, key]).unwrap();
                }
            }
            Event::Shortcut if *hk => {
                // 按键按下
                let key = (app::event_key().bits() & 0xff) as u8;
                if bmap.push(key) {
                    txc.write_all(&[dscom::KEY_DOWN, key]).unwrap();
                }
            }
            Event::KeyUp if *hk => {
                // 按键放开
                let key = (app::event_key().bits() & 0xff) as u8;
                bmap.remove(key);
                txc.write_all(&[dscom::KEY_UP, key])
                    .unwrap();
            }
            Event::Move if *hk => {
                // 鼠标移动
                let relx = (w * app::event_x() / f.width()) as u16;
                let rely = (h * app::event_y() / f.height()) as u16;
                // MOVE xu xd yu yd
                txc.write_all(&[
                    dscom::MOVE,
                    (relx >> 8) as u8,
                    (relx & 0xff) as u8,
                    (rely >> 8) as u8,
                    (rely & 0xff) as u8,
                ])
                .unwrap();
            }
            Event::Push if *hk => {
                // 鼠标按下
                txc.write_all(&[
                    dscom::MOUSE_KEY_DOWN,
                    (app::event_key().bits() & 0xff) as u8,
                ])
                .unwrap();
            }
            Event::Released if *hk => {
                // 鼠标释放
                txc.write_all(&[dscom::MOUSE_KEY_UP, (app::event_key().bits() & 0xff) as u8])
                    .unwrap();
            }
            Event::Drag if *hk => {
                // 鼠标按下移动
                let relx = (w * app::event_x() / f.width()) as u16;
                let rely = (h * app::event_y() / f.height()) as u16;
                // MOVE xu xd yu yd
                txc.write_all(&[
                    dscom::MOVE,
                    (relx >> 8) as u8,
                    (relx & 0xff) as u8,
                    (rely >> 8) as u8,
                    (rely & 0xff) as u8,
                ])
                .unwrap();
            }
            Event::MouseWheel if *hk => {
                // app::MouseWheel::Down;
                match app::event_dy() {
                    app::MouseWheel::Down => {
                        // 滚轮下滚
                        txc.write_all(&[dscom::MOUSE_WHEEL_DOWN]).unwrap();
                    }
                    app::MouseWheel::Up => {
                        // 滚轮上滚
                        txc.write_all(&[dscom::MOUSE_WHEEL_UP]).unwrap();
                    }
                    _ => {}
                }
            }
            _ => {
                if *hk {
                    println!("{}", ev);
                }
            }
        }
        true
    });
    let dura = 1.0 / (dscom::FPS as f64);
    while app.wait() {
        frame.redraw();
        // 30fps
        app::sleep(dura);
    }
}
