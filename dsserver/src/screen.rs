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
    capturer: Option<Capturer>,
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
            capturer: Some(capturer),
            sleep: Duration::new(1, 0) / 60,
        }
    }
    fn reload(&mut self) {
        println!("Reload capturer");
        drop(self.capturer.take());
        let display = match Display::primary() {
            Ok(display) => display,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };

        let capturer = match Capturer::new(display) {
            Ok(capturer) => capturer,
            Err(_) => return,
        };
        self.capturer = Some(capturer);
    }
    pub fn wh(&self) -> (usize, usize) {
        return (self.w, self.h);
    }
    pub fn cap(&mut self, cap_buf: &mut [u8]) {
        loop {
            match &mut self.capturer {
                Some(capturer) => {
                    // Wait until there's a frame.
                    let cp = capturer.frame();
                    let buffer = match cp {
                        Ok(buffer) => buffer,
                        Err(error) => {
                            std::thread::sleep(self.sleep);
                            if error.kind() == WouldBlock {
                                // Keep spinning.
                                continue;
                            } else {
                                std::thread::sleep(std::time::Duration::from_millis(200));
                                self.reload();
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
                None => {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    self.reload();
                    continue;
                }
            };
        }
    }
}
