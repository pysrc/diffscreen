use crate::config::{SKIP, SKIP_LENGTH, SKIP_TIME, COMPRESS_LEVEL};
use std::cell::RefCell;
use zstd::block::Compressor;


#[inline]
pub fn skip(clen: usize) {
    if SKIP {
        if clen > SKIP_LENGTH {
            std::thread::sleep(std::time::Duration::from_millis(SKIP_TIME));
        }
    }
}


thread_local! {
    static COMPRESSOR: RefCell<Compressor>  = RefCell::new(Compressor::new());
}

pub fn compress(src: &[u8], dst: &mut Vec<u8>) -> usize {
    unsafe {
        dst.set_len(0);
    }
    return COMPRESSOR.with(|c| {
        let mut comp = c.borrow_mut();
        return comp.compress_to_buffer(src, dst, COMPRESS_LEVEL).unwrap();
    });
}