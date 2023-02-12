use fltk::button::Button;
use fltk::enums::Color;
use fltk::frame::Frame;
use fltk::input::Input;
use fltk::prelude::InputExt;
use fltk::window::Window;
use quinn::ClientConfig;
use quinn::Endpoint;
use tokio::sync::Mutex;
use std::fs::File;
use std::io::BufReader;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

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


#[tokio::main]
pub async fn app_run() {
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
    host_ipt.set_value("127.0.0.1:9980");
    let mut login_btn = Button::default().with_label("Login").with_pos(200, 80).with_size(80, 40);
    // wind窗口结束绘制
    wind.end();
    wind.show();

    let (tx, rx) = app::channel::<()>();
    login_btn.set_callback(move |_|{
        tx.send(());
    });

    while app.wait() {
        match rx.recv() {
            Some(()) => {
                wind.hide();
                let host = host_ipt.value();
                draw(host).await;
            }
            _ => {}
        }
    }
}

enum Msg {
    Draw,
}

// 解包
#[inline]
fn depack(buffer: &[u8]) -> usize {
    ((buffer[0] as usize) << 16) | ((buffer[1] as usize) << 8) | (buffer[2] as usize)
}

async fn draw(host: String) {

    let cert = "cert.pem";
    let file = File::open(Path::new(cert))
            .expect(format!("cannot open {}", cert).as_str());
    let mut br = BufReader::new(file);
    let cetrs = rustls_pemfile::certs(&mut br).unwrap();

    let certificate = rustls::Certificate(cetrs[0].clone());
    let mut certs = rustls::RootCertStore::empty();
    certs.add(&certificate).unwrap();

    let client_config = ClientConfig::with_root_certificates(certs);

    let endpoint = {
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
        let mut endpoint = Endpoint::client(bind_addr).unwrap();
        endpoint.set_default_client_config(client_config);
        endpoint
    };
    let server_addr = SocketAddr::from_str(host.as_str()).unwrap();
    let new_conn = endpoint.connect(server_addr, "diffscreen").unwrap().await.unwrap();

    let (mut wstream, mut rstream) = new_conn.open_bi().await.unwrap();

    wstream.write(&[1u8]).await.unwrap();

    // 接收meta信息
    let mut meta = [0u8; 4];
    if let Err(_) = rstream.read_exact(&mut meta).await {
        return;
    }
    let w = (((meta[0] as u16) << 8) | meta[1] as u16) as i32;
    let h = (((meta[2] as u16) << 8) | meta[3] as u16) as i32;

    // 开始绘制wind2窗口
    let (sw, sh) = app::screen_size();
    let mut wind_screen = Window::default()
        .with_size((sw / 2.0) as i32, (sh / 2.0) as i32)
        .with_label("Diffscreen");
    let mut frame = Frame::default().size_of(&wind_screen);
    wind_screen.make_resizable(true);
    wind_screen.end();
    wind_screen.show();

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
    let hooked = Arc::new(Mutex::new(AtomicBool::new(false)));
    let abmap = Arc::new(Mutex::new(bitmap::Bitmap::new()));
    let mut cmd_buf = [0u8; 5];
    
    let awstream = Arc::new(Mutex::new(wstream));
    frame.handle(move |ff, ev| {
        let e = ev.clone();
        let f = ff.clone();
        let bmap = abmap.clone();
        let a2wstream = awstream.clone();
        let hk = hooked.clone();
        tokio::spawn(async move {
            match e {
                Event::Enter => {
                    // 进入窗口
                    hk.lock().await.store(true, Ordering::Relaxed);
                }
                Event::Leave => {
                    // 离开窗口
                    hk.lock().await.store(false, Ordering::Relaxed);
                }
                Event::KeyDown if hk.lock().await.load(Ordering::Relaxed) => {
                    // 按键按下
                    let key = app::event_key().bits() as u8;
                    cmd_buf[0] = dscom::KEY_DOWN;
                    cmd_buf[1] = key;
                    let mut bm = bmap.lock().await;
                    bm.push(key);
                    a2wstream.lock().await.write_all(&cmd_buf[..2 as usize]).await.unwrap();
                }
                Event::Shortcut if hk.lock().await.load(Ordering::Relaxed) => {
                    // 按键按下
                    let key = app::event_key().bits() as u8;
                    cmd_buf[0] = dscom::KEY_DOWN;
                    cmd_buf[1] = key;
                    let mut bm = bmap.lock().await;
                    bm.push(key);
                    a2wstream.lock().await.write_all(&cmd_buf[..2 as usize]).await.unwrap();
                }
                Event::KeyUp if hk.lock().await.load(Ordering::Relaxed) => {
                    // 按键放开
                    let key = app::event_key().bits() as u8;
                    cmd_buf[0] = dscom::KEY_UP;
                    cmd_buf[1] = key;
                    let mut bm = bmap.lock().await;
                    bm.remove(key);
                    a2wstream.lock().await.write_all(&cmd_buf[..2 as usize]).await.unwrap();
                }
                Event::Move if hk.lock().await.load(Ordering::Relaxed) => {
                    // 鼠标移动
                    let relx = (w * app::event_x() / f.width()) as u16;
                    let rely = (h * app::event_y() / f.height()) as u16;
                    // MOVE xu xd yu yd
                    cmd_buf[0] = dscom::MOVE;
                    cmd_buf[1] = (relx >> 8) as u8;
                    cmd_buf[2] = relx as u8;
                    cmd_buf[3] = (rely >> 8) as u8;
                    cmd_buf[4] = rely as u8;
                    a2wstream.lock().await.write_all(&cmd_buf).await.unwrap();
                }
                Event::Push if hk.lock().await.load(Ordering::Relaxed) => {
                    // 鼠标按下
                    cmd_buf[0] = dscom::MOUSE_KEY_DOWN;
                    cmd_buf[1] = app::event_key().bits() as u8;
                    a2wstream.lock().await.write_all(&cmd_buf[..2 as usize]).await.unwrap();
                    
                }
                Event::Released if hk.lock().await.load(Ordering::Relaxed) => {
                    // 鼠标释放
                    cmd_buf[0] = dscom::MOUSE_KEY_UP;
                    cmd_buf[1] = app::event_key().bits() as u8;
                    a2wstream.lock().await.write_all(&cmd_buf[..2 as usize]).await.unwrap();
                    
                }
                Event::Drag if hk.lock().await.load(Ordering::Relaxed) => {
                    // 鼠标按下移动
                    let relx = (w * app::event_x() / f.width()) as u16;
                    let rely = (h * app::event_y() / f.height()) as u16;
                    // MOVE xu xd yu yd
                    cmd_buf[0] = dscom::MOVE;
                    cmd_buf[1] = (relx >> 8) as u8;
                    cmd_buf[2] = relx as u8;
                    cmd_buf[3] = (rely >> 8) as u8;
                    cmd_buf[4] = rely as u8;
                    a2wstream.lock().await.write_all(&cmd_buf).await.unwrap();
                    
                }
                Event::MouseWheel if hk.lock().await.load(Ordering::Relaxed) => {
                    // app::MouseWheel::Down;
                    match app::event_dy() {
                        app::MouseWheel::Down => {
                            // 滚轮下滚
                            cmd_buf[0] = dscom::MOUSE_WHEEL_DOWN;
                            a2wstream.lock().await.write_all(&cmd_buf[..1]).await.unwrap();
                            
                        }
                        app::MouseWheel::Up => {
                            // 滚轮上滚
                            cmd_buf[0] = dscom::MOUSE_WHEEL_UP;
                            a2wstream.lock().await.write_all(&cmd_buf[..1]).await.unwrap();
                            
                        }
                        _ => {}
                    }
                }
                _ => {
                    if hk.lock().await.load(Ordering::Relaxed) {
                        println!("{}", ev);
                    }
                }
            }
        });
        
        true
    });

    let (tx, rx) = app::channel::<Msg>();

    tokio::spawn(async move {
        let mut ctx = zstd::zstd_safe::DCtx::create();
        let mut recv_buf = Vec::<u8>::with_capacity(dlen);
        unsafe {
            recv_buf.set_len(dlen);
        }
        let mut depres_data = Vec::<u8>::with_capacity(dlen);
        // 接收第一帧数据
        let mut header = [0u8; 3];
        if let Err(_) = rstream.read_exact(&mut header).await {
            return;
        }
        let recv_len = depack(&header);
        if let Err(e) = rstream.read_exact(&mut recv_buf[..recv_len]).await {
            println!("error {}", e);
            return;
        }
        if let Ok(mut _buf) = work_buf.write() {
            unsafe {
                _buf.set_len(0);
            }
            ctx.decompress(&mut *_buf, &recv_buf[..recv_len]).unwrap();
        }
        tx.send(Msg::Draw);

        loop {
            if let Err(_) = rstream.read_exact(&mut header).await {
                return;
            }
            let recv_len = depack(&header);
            if let Err(_) = rstream.read_exact(&mut recv_buf[..recv_len]).await {
                return;
            }
            unsafe {
                depres_data.set_len(0);
            }
            ctx.decompress(&mut depres_data, &recv_buf[..recv_len])
                .unwrap();
            if let Ok(mut _buf) = work_buf.write() {
                _buf.par_iter_mut()
                    .zip(depres_data.par_iter())
                    .for_each(|(_d, d)| {
                        *_d ^= *d;
                    });
            }
            tx.send(Msg::Draw);
            app::awake();
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
    endpoint.close(0u32.into(), b"close");
}
