use std::ptr::null_mut;
use windows_sys::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
    GetDeviceCaps, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS, GetDIBits, HORZRES, SRCCOPY, VERTRES,
};

// Import log_to_file from logger module
use crate::logger::log_to_file;

/// Saves the screen capture as app.png
pub unsafe fn take_screenshot() {
    log_to_file("Taking screenshot...");

    let screen_dc = GetDC(null_mut());
    let width = GetDeviceCaps(screen_dc, HORZRES as i32) as u32;
    let height = GetDeviceCaps(screen_dc, VERTRES as i32) as u32;

    let mem_dc = CreateCompatibleDC(screen_dc);
    let bitmap = CreateCompatibleBitmap(screen_dc, width as i32, height as i32);
    let old_bitmap = SelectObject(mem_dc, bitmap);

    BitBlt(mem_dc, 0, 0, width as i32, height as i32, screen_dc, 0, 0, SRCCOPY);

    // Prepare buffer for GetDIBits (BGRA format)
    let mut bmp_info: BITMAPINFO = std::mem::zeroed();
    bmp_info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmp_info.bmiHeader.biWidth = width as i32;
    bmp_info.bmiHeader.biHeight = -(height as i32); // Negative for top-down DIB
    bmp_info.bmiHeader.biPlanes = 1;
    bmp_info.bmiHeader.biBitCount = 32;
    bmp_info.bmiHeader.biCompression = BI_RGB;

    let stride = ((width * 4 + 3) / 4) * 4;
    let buffer_size = (stride * height) as usize;
    let mut pixels: Vec<u8> = vec![0; buffer_size];

    let lines_copied = GetDIBits(
        mem_dc,
        bitmap,
        0,
        height,
        pixels.as_mut_ptr() as *mut _,
        &mut bmp_info,
        DIB_RGB_COLORS,
    );

    SelectObject(mem_dc, old_bitmap);
    DeleteObject(bitmap);
    DeleteDC(mem_dc);
    ReleaseDC(null_mut(), screen_dc);

    if lines_copied == 0 {
        log_to_file("Failed to get bitmap data");
        return;
    }

    // Convert BGRA to RGBA and flip vertically
    let mut rgba_pixels: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height as usize {
        let row_start = y * stride as usize;
        for x in 0..width as usize {
            let offset = row_start + x * 4;
            // BGRA -> RGBA
            rgba_pixels.push(pixels[offset + 2]); // R
            rgba_pixels.push(pixels[offset + 1]); // G
            rgba_pixels.push(pixels[offset + 0]); // B
            rgba_pixels.push(255); // A
        }
    }

    // Create image buffer and save as PNG
    let _img = image::RgbaImage::from_raw(width, height, rgba_pixels)
        .expect("Failed to create image buffer");

    // if let Err(e) = _img.save("app.png") {
    //     log_to_file(&format!("Failed to save screenshot: {}", e));
    //     return;
    // }

    log_to_file(&format!(
        "Screenshot saved: app.png ({}x{})",
        width, height
    ));
}