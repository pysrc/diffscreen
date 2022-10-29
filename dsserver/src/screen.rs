use scrap::Capturer;
use scrap::Display;
use std::io::ErrorKind::WouldBlock;
use std::time::Duration;
use rayon::prelude::*;

use crate::config;

/**
 * 截屏
 */
pub struct Cap {
    w: usize,
    h: usize,
    // org_len: usize,
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
            // org_len: w * h * 4,
            capturer: Some(capturer),
            sleep: Duration::new(1, 0) / 60,
        }
    }
    fn reload(&mut self) {
        println!("Reload capturer");
        drop(self.capturer.take());
        let display = match Display::primary() {
            Ok(display) => display,
            Err(_) => {
                return;
            }
        };

        let capturer = match Capturer::new(display) {
            Ok(capturer) => capturer,
            Err(_) => return,
        };
        self.capturer = Some(capturer);
    }
    #[inline]
    pub fn wh(&self) -> (usize, usize) {
        return (self.w, self.h);
    }
    #[inline]
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
                    cap_buf.par_chunks_exact_mut(3).zip(buffer.par_chunks_exact(4)).for_each(|(c, b)|{
                        c[0] = b[2] & config::BIT_MASK;
                        c[1] = b[1] & config::BIT_MASK;
                        c[2] = b[0] & config::BIT_MASK;
                    });
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
