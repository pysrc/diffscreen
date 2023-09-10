use crate::key_mouse;
use crate::screen::Cap;
use enigo::Enigo;
use enigo::KeyboardControllable;
use enigo::MouseControllable;
use flate2::Compression;
use flate2::write::DeflateEncoder;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc::channel;
use rayon::prelude::*;

pub fn run(port: u16, pwd: String) {
    let mut hasher = DefaultHasher::new();
    hasher.write(pwd.as_bytes());
    let pk = hasher.finish();
    let suc = [
        (pk >> (7 * 8)) as u8,
        (pk >> (6 * 8)) as u8,
        (pk >> (5 * 8)) as u8,
        (pk >> (4 * 8)) as u8,
        (pk >> (3 * 8)) as u8,
        (pk >> (2 * 8)) as u8,
        (pk >> (1 * 8)) as u8,
        pk as u8,
    ];
    let (tx6, rx) = channel::<TcpStream>();
    if cfg!(target_os = "windows") {
        let tx4 = tx6.clone();
        std::thread::spawn(move || {
            let listener_ipv4 = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
            for sr in listener_ipv4.incoming() {
                match sr {
                    Ok(stream) => {
                        tx4.send(stream).unwrap();
                    }
                    Err(e) => {
                        println!("error {}", e);
                    }
                }
            }
        });
    }
    std::thread::spawn(move || {
        let listener_ipv6 = TcpListener::bind(format!("[::0]:{}", port)).unwrap();
        for sr in listener_ipv6.incoming() {
            match sr {
                Ok(stream) => {
                    tx6.send(stream).unwrap();
                }
                Err(e) => {
                    println!("error {}", e);
                }
            }
        }
    });

    loop {
        match rx.recv() {
            Ok(mut stream) => {
                // 检查连接合法性
                let mut check = [0u8; 8];
                match stream.read_exact(&mut check) {
                    Ok(_) => {
                        if suc != check {
                            println!("Password error");
                            let _ = stream.write_all(&[2]);
                            continue;
                        }
                    }
                    Err(_) => {
                        println!("Request error");
                        continue;
                    }
                }
                if let Err(_) = stream.write_all(&[1]) {
                    continue;
                }
                let ss = stream.try_clone().unwrap();
                let th1 = std::thread::spawn(move || {
                    screen_stream(ss);
                });
                let th2 = std::thread::spawn(move || {
                    event(stream);
                });
                th1.join().unwrap();
                th2.join().unwrap();
                println!("Break !");
            }
            Err(_) => {
                return;
            }
        }
    }
}

/**
 * 事件处理
 */
fn event(mut stream: TcpStream) {
    let mut cmd = [0u8];
    let mut move_cmd = [0u8; 4];
    let mut enigo = Enigo::new();
    while let Ok(_) = stream.read_exact(&mut cmd) {
        match cmd[0] {
            dscom::KEY_UP => {
                stream.read_exact(&mut cmd).unwrap();
                if let Some(key) = key_mouse::key_to_enigo(cmd[0]) {
                    enigo.key_up(key);
                }
            }
            dscom::KEY_DOWN => {
                stream.read_exact(&mut cmd).unwrap();
                if let Some(key) = key_mouse::key_to_enigo(cmd[0]) {
                    enigo.key_down(key);
                }
            }
            dscom::MOUSE_KEY_UP => {
                stream.read_exact(&mut cmd).unwrap();
                if let Some(key) = key_mouse::mouse_to_engin(cmd[0]) {
                    enigo.mouse_up(key);
                }
            }
            dscom::MOUSE_KEY_DOWN => {
                stream.read_exact(&mut cmd).unwrap();
                if let Some(key) = key_mouse::mouse_to_engin(cmd[0]) {
                    enigo.mouse_down(key);
                }
            }
            dscom::MOUSE_WHEEL_UP => {
                enigo.mouse_scroll_y(-2);
            }
            dscom::MOUSE_WHEEL_DOWN => {
                enigo.mouse_scroll_y(2);
            }
            dscom::MOVE => {
                stream.read_exact(&mut move_cmd).unwrap();
                let x = ((move_cmd[0] as i32) << 8) | (move_cmd[1] as i32);
                let y = ((move_cmd[2] as i32) << 8) | (move_cmd[3] as i32);
                enigo.mouse_move_to(x, y);
            }
            _ => {
                return;
            }
        }
    }
}

/**
 * 编码数据header
 */
#[inline]
fn encode(data_len: usize, res: &mut [u8]) {
    res[0] = (data_len >> 16) as u8;
    res[1] = (data_len >> 8) as u8;
    res[2] = data_len as u8;
}


/*
图像字节序
+------------+
|     24     |
+------------+
|   length   |
+------------+
|   data     |
+------------+
length: 数据长度
data: 数据
*/
fn screen_stream(mut stream: TcpStream) {
    let mut cap = Cap::new();

    let (w, h) = cap.wh();

    // 发送w, h
    let mut meta = [0u8; 4];
    meta[0] = (w >> 8) as u8;
    meta[1] = w as u8;
    meta[2] = (h >> 8) as u8;
    meta[3] = h as u8;
    if let Err(_) = stream.write_all(&meta) {
        return;
    }
    let mut header = [0u8; 3];
    let mut yuv = Vec::<u8>::new();
    let mut last = Vec::<u8>::new();
    // 第一帧
    let bgra = cap.cap();
    dscom::convert::bgra_to_i420(w, h, bgra, &mut yuv);
    // 压缩
    let mut buf = Vec::<u8>::with_capacity(1024 * 4);
    let mut e = DeflateEncoder::new(buf, Compression::default());
    e.write_all(&yuv).unwrap();
    buf = e.reset(Vec::new()).unwrap();
    (last, yuv) = (yuv, last);

    let clen = buf.len();
    encode(clen, &mut header);
    if let Err(_) = stream.write_all(&header) {
        return;
    }
    if let Err(_) = stream.write_all(&buf) {
        return;
    }
    loop {
        let bgra = cap.cap();
        unsafe {
            yuv.set_len(0);
        }
        dscom::convert::bgra_to_i420(w, h, bgra, &mut yuv);
        if yuv[..w*h] == last[..w*h] {
            continue;
        }
        last.par_iter_mut().zip(yuv.par_iter()).for_each(|(a, b)|{
            *a = *a ^ *b;
        });
        // 压缩
        unsafe {
            buf.set_len(0);
        }
        e.write_all(&last).unwrap();
        buf = e.reset(buf).unwrap();
        (last, yuv) = (yuv, last);
        // 发送
        let clen = buf.len();
        encode(clen, &mut header);
        if let Err(_) = stream.write_all(&header) {
            return;
        }
        if let Err(_) = stream.write_all(&buf) {
            return;
        }
    }
}
