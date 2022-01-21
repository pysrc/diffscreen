use std::cell::RefCell;
use zstd::block::Decompressor;

thread_local! {
    static DECOMPRESSOR: RefCell<Decompressor> = RefCell::new(Decompressor::new());
}

pub fn decompress(src: &[u8], dst: &mut Vec<u8>) -> usize {
    unsafe {
        dst.set_len(0);
    }
    return DECOMPRESSOR.with(|d| {
        let mut dcomp = d.borrow_mut();
        return dcomp.decompress_to_buffer(src, dst).unwrap();
    });
}
