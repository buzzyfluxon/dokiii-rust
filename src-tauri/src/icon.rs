use std::io::Cursor;

use base64::Engine;

pub fn rgba_to_png_data_url(rgba: Vec<u8>, width: u32, height: u32) -> Option<String> {
    let image = image::RgbaImage::from_raw(width, height, rgba)?;
    let mut buffer = Cursor::new(Vec::new());
    image.write_to(&mut buffer, image::ImageFormat::Png).ok()?;
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(buffer.into_inner())
    ))
}

pub fn bytes_to_icon_data_url(bytes: &[u8]) -> Option<String> {
    if bytes.len() < 80 {
        return None;
    }
    let decoded = image::load_from_memory(bytes).ok()?;
    if decoded.width() < 16 {
        return None;
    }
    let resized = if decoded.width() > 128 {
        decoded.resize(128, 128, image::imageops::FilterType::Lanczos3)
    } else {
        decoded
    };
    let rgba = resized.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());
    rgba_to_png_data_url(rgba.into_raw(), width, height)
}

#[cfg(windows)]
pub fn file_icon_data_url(path: &str, size: i32) -> Option<String> {
    use std::ffi::c_void;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{HWND, SIZE};
    use windows::Win32::Graphics::Gdi::{
        DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
        DIB_RGB_COLORS, HGDIOBJ,
    };
    use windows::Win32::System::Com::{CoInitializeEx, IBindCtx, COINIT_MULTITHREADED};
    use windows::Win32::UI::Shell::{
        IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY,
    };

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let factory: IShellItemImageFactory =
            SHCreateItemFromParsingName(PCWSTR(wide.as_ptr()), None::<&IBindCtx>).ok()?;
        let hbitmap = factory
            .GetImage(SIZE { cx: size, cy: size }, SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK)
            .ok()?;

        let mut bitmap = BITMAP::default();
        let got = GetObjectW(
            HGDIOBJ(hbitmap.0),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bitmap as *mut BITMAP as *mut c_void),
        );
        if got == 0 || bitmap.bmWidth <= 0 || bitmap.bmHeight <= 0 {
            let _ = DeleteObject(HGDIOBJ(hbitmap.0));
            return None;
        }
        let width = bitmap.bmWidth;
        let height = bitmap.bmHeight;

        let mut info = BITMAPINFO::default();
        info.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: 0,
            ..Default::default()
        };
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        let hdc = GetDC(HWND::default());
        let lines = GetDIBits(
            hdc,
            hbitmap,
            0,
            height as u32,
            Some(pixels.as_mut_ptr() as *mut c_void),
            &mut info,
            DIB_RGB_COLORS,
        );
        ReleaseDC(HWND::default(), hdc);
        let _ = DeleteObject(HGDIOBJ(hbitmap.0));
        if lines == 0 {
            return None;
        }

        let any_alpha = pixels.chunks_exact(4).any(|px| px[3] != 0);
        for px in pixels.chunks_exact_mut(4) {
            px.swap(0, 2);
            if !any_alpha {
                px[3] = 255;
            }
        }
        rgba_to_png_data_url(pixels, width as u32, height as u32)
    }
}

#[cfg(not(windows))]
pub fn file_icon_data_url(_path: &str, _size: i32) -> Option<String> {
    None
}
