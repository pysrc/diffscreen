
use std::{io::ErrorKind::WouldBlock};
use scrap::Display;
use std::time::Duration;
use scrap::Capturer;

/**
 * 截屏
 */
pub struct Cap {
    w: usize,
    h: usize,
    step: usize,
    capturer: Capturer,
    sleep: Duration,
}
impl Cap {
    pub fn new() -> Cap {
        let display = Display::primary().unwrap();
        let capturer = Capturer::new(display).unwrap();
        let (w, h) = (capturer.width(), capturer.height());
        Cap {
                w,
                h,
                step: w * 4,
                capturer,
                sleep: Duration::new(1, 0) / 60,
            }
    }
    pub fn wh(&self) -> (usize, usize) {
        return (self.w, self.h);
    }
    pub fn cap(&mut self, cap_buf: &mut [u8]) {
        loop {
            // Wait until there's a frame.
            let cp = &self.capturer.frame();
            let buffer = match cp {
                Ok(buffer) => buffer,
                Err(error) => {
                    std::thread::sleep(self.sleep);
                    if error.kind() == WouldBlock {
                        // Keep spinning.
                        continue;
                    } else {
                        println!("cap error: {}", error);
                        continue;
                    }
                }
            };

            // 转换成rgb图像数组
            let (w, h) = (self.w, self.h);
            let mut k = 0;
            for y in 0..h {
                for x in 0..w {
                    let i = self.step * y + 4 * x;
                    cap_buf[k] = buffer[i+2];
                    k+=1;
                    cap_buf[k] = buffer[i+1];
                    k+=1;
                    cap_buf[k] = buffer[i];
                    k+=1;
                }
            }
            break;
        }
    }
}