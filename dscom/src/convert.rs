pub fn bgra_to_i420(width: usize, height: usize, src: &[u8], dest: &mut Vec<u8>) {
    let stride = src.len() / height;

    dest.clear();

    for y in 0..height {
        for x in 0..width {
            let o = y * stride + 4 * x;

            let b = src[o] as i32;
            let g = src[o + 1] as i32;
            let r = src[o + 2] as i32;

            let y = (66 * r + 129 * g + 25 * b + 128) / 256 + 16;
            dest.push(clamp(y));
        }
    }

    for y in (0..height).step_by(2) {
        for x in (0..width).step_by(2) {
            let o = y * stride + 4 * x;

            let b = src[o] as i32;
            let g = src[o + 1] as i32;
            let r = src[o + 2] as i32;

            let u = (-38 * r - 74 * g + 112 * b + 128) / 256 + 128;
            dest.push(clamp(u));
        }
    }

    for y in (0..height).step_by(2) {
        for x in (0..width).step_by(2) {
            let o = y * stride + 4 * x;

            let b = src[o] as i32;
            let g = src[o + 1] as i32;
            let r = src[o + 2] as i32;

            let v = (112 * r - 94 * g - 18 * b + 128) / 256 + 128;
            dest.push(clamp(v));
        }
    }
}

fn clamp(x: i32) -> u8 {
    x.min(255).max(0) as u8
}

#[allow(dead_code)]
pub fn i420_to_rgb(width: usize, height: usize, sy: &[u8], su: &[u8], sv: &[u8], dest: &mut [u8], crop_width: usize, crop_height: usize) {
    // 确保裁剪尺寸不超过原始尺寸
    let crop_width = crop_width.min(width);
    let crop_height = crop_height.min(height);
    let uvw = width >> 1;
    for i in 0..crop_height {
        let sw = i * width;
        let swc = i * crop_width;
        let t = (i >> 1) * uvw;
        for j in 0..crop_width {
            let rgbstart = sw + j;
            let mut rgbstartc = swc + j;
            let uvi = t + (j >> 1);

            let y = sy[rgbstart] as i32;
            let u = su[uvi] as i32 - 128;
            let v = sv[uvi] as i32 - 128;

            rgbstartc *= 3;
            dest[rgbstartc] = clamp(y + (v * 359 >> 8));
            dest[rgbstartc + 1] = clamp(y - (u * 88 >> 8) - (v * 182 >> 8));
            dest[rgbstartc + 2] = clamp(y + (u * 453 >> 8));

        }
    }

}