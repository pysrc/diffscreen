fn main() {
    let version = unsafe {vpx_sys::vpx_codec_version()};
    println!("VPX version: 0x{:x}", version);
}
