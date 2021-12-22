use scrap::Capturer;
use scrap::Display;
use std::io::ErrorKind::WouldBlock;
use std::time::Duration;

/**
 * 截屏
 */
pub struct Cap {
    w: usize,
    h: usize,
    org_len: usize,
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
            org_len: w * h * 4,
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
            let mut k = 0;
            let mut n = 0;
            while n < self.org_len {
                cap_buf[k] = buffer[n + 2] & dscom::BIT_SAVE;
                cap_buf[k + 1] = buffer[n + 1] & dscom::BIT_SAVE;
                cap_buf[k + 2] = buffer[n] & dscom::BIT_SAVE;
                k += 3;
                n += 4;
            }
            break;
        }
    }
}
