// 默认帧率
use std::cell::RefCell;
use zstd::block::Compressor;
use zstd::block::Decompressor;

pub const FPS: u64 = 20;

// 传输像素保留位数（右边0越多压缩程度越大）
pub const BIT_SAVE: u8 = 0b1111_1000;

// 传输压缩水平0-21 0消耗资源最小但是压缩率小（需要带宽大） 21消耗资源最大，但但是压缩率大（需要带宽小）
pub const COMPRESS_LEVEL: i32 = 3;

// key事件 start
pub const KEY_UP: u8 = 1;
pub const KEY_DOWN: u8 = 2;
pub const MOUSE_KEY_UP: u8 = 3;
pub const MOUSE_KEY_DOWN: u8 = 4;
pub const MOUSE_WHEEL_UP: u8 = 5;
pub const MOUSE_WHEEL_DOWN: u8 = 6;
pub const MOVE: u8 = 7;
// key事件 end

thread_local! {
    static COMPRESSOR: RefCell<Compressor> = RefCell::new(Compressor::new());
    static DECOMPRESSOR: RefCell<Decompressor> = RefCell::new(Decompressor::new());
}

pub fn compress(src: &[u8], dst: &mut Vec<u8>) -> usize {
    unsafe {
        dst.set_len(0);
    }
    return COMPRESSOR.with(|c| {
        let mut comp = c.borrow_mut();
        return comp
            .compress_to_buffer(src, &mut *dst, COMPRESS_LEVEL)
            .unwrap();
    });
}

pub fn decompress(src: &[u8], dst: &mut Vec<u8>) -> usize {
    unsafe {
        dst.set_len(0);
    }
    return DECOMPRESSOR.with(|d| {
        let mut dcomp = d.borrow_mut();
        return dcomp.decompress_to_buffer(src, &mut *dst).unwrap();
    });
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
