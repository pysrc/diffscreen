//! Rust interface to libvpx encoder
//!
//! This crate provides a Rust API to use
//! [libvpx](https://en.wikipedia.org/wiki/Libvpx) for encoding images.
//!
//! It it based entirely on code from [srs](https://crates.io/crates/srs).
//! Compared to the original `srs`, this code has been simplified for use as a
//! library and updated to add support for both the VP8 codec and (optionally)
//! the VP9 codec.
//!
//! # Optional features
//!
//! Compile with the cargo feature `vp9` to enable support for the VP9 codec.
//!
//! # Example
//!
//! An example of using `vpx-encode` can be found in the [`record-screen`]()
//! program. The source code for `record-screen` is in the [vpx-encode git
//! repository]().
//!
//! # Contributing
//!
//! All contributions are appreciated.

// vpx_sys is provided by the `env-libvpx-sys` crate

#![cfg_attr(
    feature = "backtrace",
    feature(error_generic_member_access, provide_any)
)]

use std::{
    mem::MaybeUninit,
    os::raw::{c_int, c_uint},
};

#[cfg(feature = "backtrace")]
use std::backtrace::Backtrace;
use std::{ptr, slice};

use thiserror::Error;

#[allow(unused_imports)]
#[cfg(feature = "vp9")]
use vpx_sys::vp8e_enc_control_id::*;
use vpx_sys::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VideoCodecId {
    VP8,
    #[cfg(feature = "vp9")]
    VP9,
}

impl Default for VideoCodecId {
    #[cfg(not(feature = "vp9"))]
    fn default() -> VideoCodecId {
        VideoCodecId::VP8
    }

    #[cfg(feature = "vp9")]
    fn default() -> VideoCodecId {
        VideoCodecId::VP9
    }
}

pub struct Decoder {
    ctx: vpx_codec_ctx_t,
}

#[derive(Debug, Error)]
#[error("VPX encode error: {msg}")]
pub struct Error {
    msg: String,
    #[cfg(feature = "backtrace")]
    #[backtrace]
    backtrace: Backtrace,
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Self {
            msg,
            #[cfg(feature = "backtrace")]
            backtrace: Backtrace::capture(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! call_vpx {
    ($x:expr) => {{
        let result = unsafe { $x }; // original expression
        let result_int = unsafe { std::mem::transmute::<_, i32>(result) };
        // if result != VPX_CODEC_OK {
        if result_int != 0 {
            return Err(Error::from(format!(
                "Function call failed (error code {}).",
                result_int
            )));
        }
        result
    }};
}

macro_rules! call_vpx_ptr {
    ($x:expr) => {{
        let result = unsafe { $x }; // original expression
        if result.is_null() {
            return Err(Error::from("Bad pointer.".to_string()));
        }
        result
    }};
}

impl Decoder {
    pub fn new(config: Config) -> Result<Self> {
        let i = match config.codec {
            VideoCodecId::VP8 => call_vpx_ptr!(vpx_codec_vp8_dx()),
            #[cfg(feature = "vp9")]
            VideoCodecId::VP9 => call_vpx_ptr!(vpx_codec_vp9_dx()),
        };

        if config.width % 2 != 0 {
            return Err(Error::from("Width must be divisible by 2".to_string()));
        }
        if config.height % 2 != 0 {
            return Err(Error::from("Height must be divisible by 2".to_string()));
        }

        let cfg = vpx_codec_dec_cfg_t {
            threads: 1,
            w: 0,
            h: 0,
        };

        let ctx = MaybeUninit::zeroed();
        let mut ctx = unsafe { ctx.assume_init() };

        match config.codec {
            VideoCodecId::VP8 => {
                call_vpx!(vpx_codec_dec_init_ver(
                    &mut ctx,
                    i,
                    &cfg,
                    0,
                    vpx_sys::VPX_DECODER_ABI_VERSION as i32
                ));
            }
            #[cfg(feature = "vp9")]
            VideoCodecId::VP9 => {
                call_vpx!(vpx_codec_dec_init_ver(
                    &mut ctx,
                    i,
                    &cfg,
                    0,
                    vpx_sys::VPX_DECODER_ABI_VERSION as i32
                ));
            }
        };

        Ok(Self {
            ctx,
        })
    }

    pub fn decode(&mut self, data: &[u8]) -> Result<Packets> {
        call_vpx!(vpx_codec_decode(
            &mut self.ctx,
            data.as_ptr(),
            data.len() as _,
            ptr::null_mut(),
            0,
        ));

        Ok(Packets {
            ctx: &mut self.ctx,
            iter: ptr::null(),
        })
    }

    pub fn finish(mut self) -> Result<Finish> {
        call_vpx!(vpx_codec_decode(
            &mut self.ctx,
            ptr::null(),
            0, // PTS
            ptr::null_mut(),
            0,  // Flags
        ));

        Ok(Finish {
            enc: self,
            iter: ptr::null(),
        })
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe {
            let result = vpx_codec_destroy(&mut self.ctx);
            if result != vpx_sys::VPX_CODEC_OK {
                eprintln!("failed to destroy vpx codec: {result:?}");
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Frame<'a> {
    /// Compressed data.
    pub data: &'a [u8],
    /// Whether the frame is a keyframe.
    pub key: bool,
    /// Presentation timestamp (in timebase units).
    pub pts: i64,
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// The width (in pixels).
    pub width: c_uint,
    /// The height (in pixels).
    pub height: c_uint,
    /// The timebase numerator and denominator (in seconds).
    pub timebase: [c_int; 2],
    /// The target bitrate (in kilobits per second).
    pub bitrate: c_uint,
    /// The codec
    pub codec: VideoCodecId,
}

pub struct Image(*mut vpx_image_t);
impl Image {
    #[inline]
    pub fn new() -> Self {
        Self(std::ptr::null_mut())
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    #[inline]
    pub fn format(&self) -> vpx_img_fmt_t {
        // VPX_IMG_FMT_I420
        self.inner().fmt
    }

    #[inline]
    pub fn inner(&self) -> &vpx_image_t {
        unsafe { &*self.0 }
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.stride()[0] as usize
        // self.inner().d_w as _
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.inner().d_h as _
    }

    #[inline]
    pub fn stride(&self) -> Vec<i32> {
        self.inner().stride.iter().map(|x| *x as i32).collect()
    }

    #[inline]
    fn planes(&self) -> Vec<*mut u8> {
        self.inner().planes.iter().map(|p| *p as *mut u8).collect()
    }

    #[inline]
    pub fn data(&self) -> (&[u8], &[u8], &[u8]) {
        unsafe {
            let stride = self.stride();
            let planes = self.planes();
            let h = (self.height() as usize + 1) & !1;
            let n = stride[0] as usize * h;
            let y = slice::from_raw_parts(planes[0], n);
            let n = stride[1] as usize * (h >> 1);
            let u = slice::from_raw_parts(planes[1], n);
            let v = slice::from_raw_parts(planes[2], n);
            (y, u, v)
        }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { vpx_img_free(self.0) };
        }
    }
}

pub struct Packets<'a> {
    ctx: &'a mut vpx_codec_ctx_t,
    iter: vpx_codec_iter_t,
}

impl<'a> Iterator for Packets<'a> {
    type Item = Image;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            unsafe {
                let pkt = vpx_codec_get_frame(self.ctx, &mut self.iter);
                if pkt.is_null() {
                    return None;
                } else {
                    return Some(Image(pkt));
                }
            }
        }
    }
}

pub struct Finish {
    enc: Decoder,
    iter: vpx_codec_iter_t,
}

impl Finish {
    pub fn next(&mut self) -> Result<Option<Image>> {
        let mut tmp = Packets {
            ctx: &mut self.enc.ctx,
            iter: self.iter,
        };

        if let Some(packet) = tmp.next() {
            self.iter = tmp.iter;
            Ok(Some(packet))
        } else {
            call_vpx!(vpx_codec_decode(
                tmp.ctx,
                ptr::null(),
                0,
                ptr::null_mut(),
                0,
            ));

            tmp.iter = ptr::null();
            if let Some(packet) = tmp.next() {
                self.iter = tmp.iter;
                Ok(Some(packet))
            } else {
                Ok(None)
            }
        }
    }
}

