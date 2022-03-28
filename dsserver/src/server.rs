use crate::config;
use crate::key_mouse;
use crate::screen::Cap;
use crate::util;
use enigo::Enigo;
use enigo::KeyboardControllable;
use enigo::MouseControllable;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc::channel;

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
    // if data_len > 1 << 23 {
    //     // 数据超长
    // }
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
    let dlen = w * h * 3;
    let mut data2 = Vec::<u8>::with_capacity(dlen);
    let mut data1 = Vec::<u8>::with_capacity(dlen);
    let mut pres_data = Vec::<u8>::with_capacity(dlen);
    unsafe {
        data2.set_len(dlen);
        data1.set_len(dlen);
    }

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

    // 截第一张图
    cap.cap(&mut data1);
    let clen = util::compress(&data1, &mut pres_data);
    encode(clen, &mut header);
    if let Err(_) = stream.write_all(&header) {
        return;
    }
    if let Err(_) = stream.write_all(&pres_data) {
        return;
    }
    let dura = 1000 / config::FPS;
    loop {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(dura));
            // 截图
            cap.cap(&mut data2);
            if data2 == data1 {
                continue;
            }
            // 做减法
            data1.iter_mut().zip(data2.iter()).for_each(|(d1, d2)|{
                *d1 ^= *d2;
            });
            // for i in 0..dlen {
            //     data1[i] ^= data2[i];
            // }
            // 压缩
            let clen = util::compress(&data1, &mut pres_data);
            // 发送diff
            encode(clen, &mut header);
            if let Err(_) = stream.write_all(&header) {
                return;
            }
            if let Err(_) = stream.write_all(&pres_data) {
                return;
            }
            util::skip(clen);
            break;
        }

        loop {
            std::thread::sleep(std::time::Duration::from_millis(dura));
            // 截图
            cap.cap(&mut data1);
            if data1 == data2 {
                continue;
            }
            // 做减法
            data2.iter_mut().zip(data1.iter()).for_each(|(d2, d1)|{
                *d2 ^= *d1;
            });
            // for i in 0..dlen {
            //     data2[i] ^= data1[i];
            // }
            // 压缩
            let clen = util::compress(&data2, &mut pres_data);
            // 发送diff
            encode(clen, &mut header);
            if let Err(_) = stream.write_all(&header) {
                return;
            }
            if let Err(_) = stream.write_all(&pres_data[..clen]) {
                return;
            }
            util::skip(clen);
            break;
        }
    }
}
