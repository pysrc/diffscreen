use crate::screen::Cap;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;

pub fn run(port: u32) {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
    for sr in listener.incoming() {
        match sr {
            Ok(stream) => {
                let ss = stream.try_clone().unwrap();
                std::thread::spawn(move || {
                    screen_stream(ss);
                });
            }
            Err(e) => {
                println!("error {}", e);
            }
        }
    }
}

/**
 * 编码数据header
 */
fn encode(data_len: usize, res: &mut [u8]) {
    if data_len > 1 << 23 {
        // 数据超长
    }
    res[0] = ((data_len >> 16) & 0xff) as u8;
    res[1] = ((data_len >> 8) & 0xff) as u8;
    res[2] = (data_len & 0xff) as u8;
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

flag:
    0: 整个图像
    1: 部分图像，需要与之前的做加和
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
        pres_data.set_len(dlen);
    }

    // 发送w, h
    let mut meta = [0u8; 4];
    meta[0] = ((w >> 8) & 0xff) as u8;
    meta[1] = (w & 0xff) as u8;
    meta[2] = ((h >> 8) & 0xff) as u8;
    meta[3] = (h & 0xff) as u8;
    if let Err(_) = stream.write_all(&meta) {
        return;
    }

    let mut header = [0u8; 3];

    // 截第一张图
    cap.cap(&mut data1);
    let clen = dscom::compress(&data1, &mut pres_data);
    encode(clen, &mut header);
    if let Err(_) = stream.write_all(&header) {
        return;
    }
    if let Err(_) = stream.write_all(&pres_data[..clen]) {
        return;
    }
    loop {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(50));
            // 截图
            cap.cap(&mut data2);
            if data2 == data1 {
                continue;
            }
            // 做减法
            for i in 0..dlen {
                data1[i] = data1[i] ^ data2[i];
            }
            // 压缩
            let clen = dscom::compress(&data1, &mut pres_data);
            // 发送diff
            encode(clen, &mut header);
            if let Err(_) = stream.write_all(&header) {
                return;
            }
            if let Err(_) = stream.write_all(&pres_data[..clen]) {
                return;
            }
            break;
        }

        loop {
            std::thread::sleep(std::time::Duration::from_millis(50));
            // 截图
            cap.cap(&mut data1);
            if data1 == data2 {
                continue;
            }
            // 做减法
            for i in 0..dlen {
                data2[i] = data2[i] ^ data1[i];
            }
            // 压缩
            let clen = dscom::compress(&data2, &mut pres_data);
            // 发送diff
            encode(clen, &mut header);
            if let Err(_) = stream.write_all(&header) {
                return;
            }
            if let Err(_) = stream.write_all(&pres_data[..clen]) {
                return;
            }
            break;
        }
    }
}
