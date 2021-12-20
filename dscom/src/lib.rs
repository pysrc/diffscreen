use std::io::Read;

use bzip2::{read::{BzEncoder, BzDecoder}, Compression};

// 默认帧率
pub const FPS: u64 = 30;

// key事件 start
pub const KEY_UP: u8 = 1;
pub const KEY_DOWN: u8 = 2;
pub const MOUSE_KEY_UP: u8 = 3;
pub const MOUSE_KEY_DOWN: u8 = 4;
pub const MOUSE_WHEEL_UP:u8 = 5;
pub const MOUSE_WHEEL_DOWN:u8 = 6;
pub const MOVE:u8 = 7;
// key事件 end





pub fn compress(src: &[u8], dst: &mut Vec<u8>) -> usize {
    unsafe{
        dst.set_len(0);
    }
    let mut compressor = BzEncoder::new(src, Compression::best());
    return compressor.read_to_end(dst).unwrap();
}

pub fn decompress(src: &[u8], dst: &mut Vec<u8>) -> usize {
    unsafe {
        dst.set_len(0);
    }
    let mut decompressor = BzDecoder::new(src);
    return decompressor.read_to_end(dst).unwrap();
}

#[cfg(test)]
mod tests {
use crate::{compress, decompress};

#[test]
fn it_works() {
        let s = "Hello11111111111111111111".as_bytes();
        let mut pr = Vec::with_capacity(100);
        unsafe {
            pr.set_len(100);
        }
        let p = compress(s, &mut pr);
        assert_ne!(&pr[..p], s);
        let mut dpr = Vec::with_capacity(100);
        unsafe {
            dpr.set_len(100);
        }
        let u = decompress(&pr[..p], &mut dpr);
        assert_eq!(s, &dpr[..u]);
    }
}
