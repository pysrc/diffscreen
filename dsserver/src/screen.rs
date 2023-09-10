use scrap::Capturer;
use scrap::Display;
use std::io::ErrorKind::WouldBlock;
use std::slice::from_raw_parts;
use std::time::Duration;

/**
 * 截屏
 */
pub struct Cap {
    w: usize,
    h: usize,
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
    pub fn wh(&self) -> (usize, usize) {
        (self.w, self.h)
    }
    #[inline]
    pub fn cap(&mut self) -> &[u8] {
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
                    return unsafe { from_raw_parts(buffer.as_ptr(), buffer.len()) };
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
