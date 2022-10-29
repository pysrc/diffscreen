use crate::config::{SKIP, SKIP_LENGTH, SKIP_TIME};
#[inline]
pub fn skip(clen: usize) {
    if SKIP {
        if clen > SKIP_LENGTH {
            std::thread::sleep(std::time::Duration::from_millis(SKIP_TIME));
        }
    }
}
