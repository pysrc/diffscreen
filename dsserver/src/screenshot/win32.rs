use std::ops::Deref;
use std::{mem, ptr};

use widestring::U16CString;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, DeleteObject, GetDeviceCaps, GetObjectW, SelectObject,
    SetStretchBltMode, StretchBlt, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DESKTOPHORZRES,
    DIB_RGB_COLORS, HBITMAP, HORZRES, RGBQUAD, SRCCOPY, STRETCH_HALFTONE,
};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{BOOL, LPARAM, RECT},
        Graphics::Gdi::{
            CreateCompatibleDC, CreateDCW, CreatedHDC, DeleteDC, EnumDisplayMonitors, GetDIBits,
            GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
        },
    },
};

struct DropDC(CreatedHDC);
impl Deref for DropDC {
    type Target = CreatedHDC;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Drop for DropDC {
    fn drop(&mut self) {
        unsafe { DeleteDC(self.0) };
    }
}

struct DropHBITMAP(HBITMAP);
impl Deref for DropHBITMAP {
    type Target = HBITMAP;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for DropHBITMAP {
    fn drop(&mut self) {
        unsafe { DeleteObject(self.0) };
    }
}

fn get_scale_factor(sz_device: *const u16) -> f32 {
    let dcw_drop_box = DropDC(unsafe {
        CreateDCW(
            PCWSTR(sz_device),
            PCWSTR(sz_device),
            PCWSTR(ptr::null()),
            None,
        )
    });
    let logical_width = unsafe { GetDeviceCaps(*dcw_drop_box, HORZRES) };
    let physical_width = unsafe { GetDeviceCaps(*dcw_drop_box, DESKTOPHORZRES) };

    physical_width as f32 / logical_width as f32
}

#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
    pub is_primary: bool,
}
impl DisplayInfo {
    fn new(monitor_info_exw: &MONITORINFOEXW) -> Self {
        let sz_device = monitor_info_exw.szDevice.as_ptr();

        let sz_device_string = unsafe { U16CString::from_ptr_str(sz_device).to_string_lossy() };
        let rc_monitor = monitor_info_exw.monitorInfo.rcMonitor;
        let dw_flags = monitor_info_exw.monitorInfo.dwFlags;

        DisplayInfo {
            id: sz_device_string,
            width: (rc_monitor.right - rc_monitor.left) as u32,
            height: (rc_monitor.bottom - rc_monitor.top) as u32,
            scale_factor: get_scale_factor(sz_device),
            is_primary: dw_flags == 1u32,
        }
    }
}

fn get_monitor_info_exw(h_monitor: HMONITOR) -> MONITORINFOEXW {
    let mut monitor_info_exw: MONITORINFOEXW = unsafe { mem::zeroed() };
    monitor_info_exw.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;
    let monitor_info_exw_ptr = <*mut _>::cast(&mut monitor_info_exw);

    unsafe {
        GetMonitorInfoW(h_monitor, monitor_info_exw_ptr)
            .ok()
            .unwrap()
    };

    monitor_info_exw
}

extern "system" fn monitor_enum_proc(
    h_monitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    state: LPARAM,
) -> BOOL {
    unsafe {
        let state = Box::leak(Box::from_raw(state.0 as *mut Vec<MONITORINFOEXW>));
        let monitor_info_exw = get_monitor_info_exw(h_monitor);
        state.push(monitor_info_exw);
        BOOL::from(true)
    }
}

pub struct Screen {
    bgra: Vec<u8>,
    pub width: i32,
    pub height: i32,
    bitmap: BITMAP,
    dcw_drop_box: DropDC,
    compatible_dc_drop_box: DropDC,
    h_bitmap_drop_box: DropHBITMAP,
    bitmap_info: BITMAPINFO,
}

impl Screen {
    pub fn new() -> Self {
        let h_monitors_mut_ptr: *mut Vec<MONITORINFOEXW> = Box::into_raw(Box::default());
        let mut h_monitors = unsafe { Box::from_raw(h_monitors_mut_ptr) };
        unsafe {
            EnumDisplayMonitors(
                HDC::default(),
                None,
                Some(monitor_enum_proc),
                LPARAM(h_monitors_mut_ptr as isize),
            )
            .ok()
            .unwrap();
        };

        let mut u = h_monitors
            .iter()
            .map(DisplayInfo::new)
            .collect::<Vec<DisplayInfo>>();
        let display = u.remove(0);

        let (width, height) = (
            ((display.width as f32) * display.scale_factor) as i32,
            ((display.height as f32) * display.scale_factor) as i32,
        );

        let monitor_info_exw = h_monitors.remove(0);
        let sz_device_ptr = monitor_info_exw.szDevice.as_ptr();
        let dcw_drop_box = DropDC(unsafe {
            CreateDCW(
                PCWSTR(sz_device_ptr),
                PCWSTR(sz_device_ptr),
                PCWSTR(ptr::null()),
                None,
            )
        });
        let compatible_dc_drop_box = DropDC(unsafe { CreateCompatibleDC(*dcw_drop_box) });
        let h_bitmap_drop_box =
            DropHBITMAP(unsafe { CreateCompatibleBitmap(*dcw_drop_box, width, height) });
        unsafe {
            SelectObject(*compatible_dc_drop_box, *h_bitmap_drop_box);
            SetStretchBltMode(*dcw_drop_box, STRETCH_HALFTONE);
        };

        let bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: height as i32, // 这里可以传递负数, 但是不知道为什么会报错
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD::default(); 1],
        };

        Screen {
            bgra: vec![0u8; (width * height) as usize * 4],
            width,
            height,
            bitmap: BITMAP::default(),
            dcw_drop_box,
            compatible_dc_drop_box,
            h_bitmap_drop_box,
            bitmap_info,
        }
    }
    pub fn capture(&mut self, mut rgb: Vec<u8>) -> Result<Vec<u8>, &str> {
        let buf_prt = self.bgra.as_ptr() as *mut _;
        unsafe {
            StretchBlt(
                *self.compatible_dc_drop_box,
                0,
                0,
                self.width,
                self.height,
                *self.dcw_drop_box,
                0,
                0,
                self.width,
                self.height,
                SRCCOPY,
            )
            .ok()
            .unwrap();
        };
        let is_success = unsafe {
            GetDIBits(
                *self.compatible_dc_drop_box,
                *self.h_bitmap_drop_box,
                0,
                self.height as u32,
                Some(buf_prt),
                &mut self.bitmap_info,
                DIB_RGB_COLORS,
            ) == 0
        };
        if is_success {
            return Err("Get bgra data failed");
        }

        let bitmap_ptr = <*mut _>::cast(&mut self.bitmap);

        unsafe {
            GetObjectW(
                *self.h_bitmap_drop_box,
                mem::size_of::<BITMAP>() as i32,
                Some(bitmap_ptr),
            );
        }

        // 旋转图像,图像数据是倒置的
        let mut i = 0usize;
        let mut j = self.bgra.len();
        let wd = self.width * 4;
        loop {
            let mut e = j - wd as usize;
            for _ in 0..wd {
                let t = self.bgra[i];
                self.bgra[i] = self.bgra[e];
                self.bgra[e] = t;
                i += 1;
                e += 1;
            }
            j -= wd as usize;
            if j <= i {
                break;
            }
        }

        // 转换成rgb图像数组
        rgb.chunks_exact_mut(3)
            .zip(self.bgra.chunks_exact(4))
            .for_each(|(c, b)| {
                c[0] = b[2];
                c[1] = b[1];
                c[2] = b[0];
            });

        return Ok(rgb);
    }
}
