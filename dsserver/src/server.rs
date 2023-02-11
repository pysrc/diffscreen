use crate::config::COMPRESS_LEVEL;
use crate::key_mouse;
use crate::screen::Cap;
use crate::util;
use enigo::Enigo;
use enigo::KeyboardControllable;
use enigo::MouseControllable;
use quinn::Endpoint;
use quinn::RecvStream;
use quinn::SendStream;
use quinn::ServerConfig;
use std::fs::File;
use std::io::BufReader;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::path::Path;
use rayon::prelude::*;

pub async fn run(port: u16) {
    
    let cert = "cert.pem";
    let file = File::open(Path::new(cert))
            .expect(format!("cannot open {}", cert).as_str());
    let mut br = BufReader::new(file);
    let cetrs = rustls_pemfile::certs(&mut br).unwrap();
    
    let key = "key.pem";
    let filek = File::open(Path::new(key))
            .expect(format!("cannot open {}", key).as_str());
    let mut brk = BufReader::new(filek);
    let keys = rustls_pemfile::pkcs8_private_keys(&mut brk).unwrap();


    let certificate = rustls::Certificate(cetrs[0].clone());
    let private_key = rustls::PrivateKey(keys[0].clone());

    let cert_chain = vec![certificate];

    let server_config = ServerConfig::with_single_cert(cert_chain, private_key).unwrap();

    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
    let endpoint = Endpoint::server(server_config, bind_addr).unwrap();

    while let Some(income_conn) = endpoint.accept().await {
        let new_conn = income_conn.await.unwrap();
        if let Ok((wstream, rstream)) = new_conn.accept_bi().await {
            let th1 = async {
                screen_stream(wstream).await;
            };
            let th2 = async {
                event(rstream).await;
            };
            tokio::join!(th1, th2);
            new_conn.close(0u32.into(), b"close");
        }
    }
}

/**
 * 事件处理
 */
async fn event(mut stream: RecvStream) {
    let mut cmd = [0u8];
    let mut move_cmd = [0u8; 4];
    let mut enigo = Enigo::new();
    while let Ok(_) = stream.read_exact(&mut cmd).await {
        match cmd[0] {
            dscom::KEY_UP => {
                stream.read_exact(&mut cmd).await.unwrap();
                if let Some(key) = key_mouse::key_to_enigo(cmd[0]) {
                    enigo.key_up(key);
                }
            }
            dscom::KEY_DOWN => {
                stream.read_exact(&mut cmd).await.unwrap();
                if let Some(key) = key_mouse::key_to_enigo(cmd[0]) {
                    enigo.key_down(key);
                }
            }
            dscom::MOUSE_KEY_UP => {
                stream.read_exact(&mut cmd).await.unwrap();
                if let Some(key) = key_mouse::mouse_to_engin(cmd[0]) {
                    enigo.mouse_up(key);
                }
            }
            dscom::MOUSE_KEY_DOWN => {
                stream.read_exact(&mut cmd).await.unwrap();
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
                stream.read_exact(&mut move_cmd).await.unwrap();
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
async fn screen_stream(mut stream: SendStream) {
    let mut ctx = zstd::zstd_safe::CCtx::create();
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
    if let Err(_) = stream.write_all(&meta).await {
        return;
    }

    let mut header = [0u8; 3];

    // 截第一张图
    cap.cap(&mut data1);
    unsafe {
        pres_data.set_len(0);
    }
    let clen = ctx.compress(&mut pres_data, &data1, COMPRESS_LEVEL).unwrap();
    // let clen = util::compress(&data1, &mut pres_data);
    encode(clen, &mut header);
    if let Err(_) = stream.write_all(&header).await {
        return;
    }
    if let Err(_) = stream.write_all(&pres_data).await {
        return;
    }
    // let dura = 1000 / config::FPS;
    loop {
        loop {
            // 截图
            cap.cap(&mut data2);
            if data2 == data1 {
                continue;
            }
            // 做减法
            data1.par_iter_mut().zip(data2.par_iter()).for_each(|(d1, d2)|{
                *d1 ^= *d2;
            });
            // 压缩
            unsafe {
                pres_data.set_len(0);
            }
            let clen = ctx.compress(&mut pres_data, &data1, COMPRESS_LEVEL).unwrap();
            // let clen = util::compress(&data1, &mut pres_data);
            // 发送diff
            encode(clen, &mut header);
            if let Err(_) = stream.write_all(&header).await {
                return;
            }
            if let Err(_) = stream.write_all(&pres_data).await {
                return;
            }
            util::skip(clen);
            break;
        }

        loop {
            // 截图
            cap.cap(&mut data1);
            if data1 == data2 {
                continue;
            }
            // 做减法
            data2.par_iter_mut().zip(data1.par_iter()).for_each(|(d2, d1)|{
                *d2 ^= *d1;
            });
            // 压缩
            unsafe {
                pres_data.set_len(0);
            }
            let clen = ctx.compress(&mut pres_data, &data2, COMPRESS_LEVEL).unwrap();
            // let clen = util::compress(&data2, &mut pres_data);
            // 发送diff
            encode(clen, &mut header);
            if let Err(_) = stream.write_all(&header).await {
                return;
            }
            if let Err(_) = stream.write_all(&pres_data[..clen]).await {
                return;
            }
            util::skip(clen);
            break;
        }
    }
}
