//! Owned system fonts and alpha icons; handles outlive every control using them.

use std::mem::size_of;
use windows::{
    core::{w, Error, Result},
    Win32::{
        Foundation::{COLORREF, HWND},
        Graphics::Gdi::*,
        UI::{
            Controls::*,
            HiDpi::{GetDpiForWindow, SystemParametersInfoForDpi},
            WindowsAndMessaging::*,
        },
    },
};

pub(super) fn px(hwnd: HWND, value: i32) -> i32 {
    // SAFETY: the UI thread owns hwnd; 96 DPI is the fallback for a missing window.
    let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96);
    ((i64::from(value) * i64::from(dpi) + 48) / 96) as i32
}

pub(super) struct Font(pub HFONT);
impl Font {
    pub(super) fn new(hwnd: HWND) -> Result<Self> {
        Self::styled(hwnd, 0, false)
    }
    pub(super) fn settings(hwnd: HWND, heading: bool) -> Result<Self> {
        Self::styled(hwnd, if heading { 18 } else { 14 }, heading)
    }
    fn styled(hwnd: HWND, minimum: i32, heading: bool) -> Result<Self> {
        // SAFETY: Windows fills a correctly sized structure for this window's DPI.
        unsafe {
            let mut metrics = NONCLIENTMETRICSW {
                cbSize: size_of::<NONCLIENTMETRICSW>() as u32,
                ..Default::default()
            };
            SystemParametersInfoForDpi(
                SPI_GETNONCLIENTMETRICS.0,
                metrics.cbSize,
                Some((&mut metrics as *mut NONCLIENTMETRICSW).cast()),
                0,
                GetDpiForWindow(hwnd).max(96),
            )?;
            let face: Vec<u16> = "Segoe UI".encode_utf16().chain(Some(0)).collect();
            metrics.lfMessageFont.lfFaceName.fill(0);
            metrics.lfMessageFont.lfFaceName[..face.len()].copy_from_slice(&face);
            if minimum > 0 {
                metrics.lfMessageFont.lfHeight =
                    -metrics.lfMessageFont.lfHeight.abs().max(px(hwnd, minimum));
            }
            if heading {
                metrics.lfMessageFont.lfWeight = FW_SEMIBOLD.0 as i32;
            }
            let font = CreateFontIndirectW(&metrics.lfMessageFont);
            if font.is_invalid() {
                return Err(Error::from_thread());
            }
            Ok(Self(font))
        }
    }
}
impl Drop for Font {
    fn drop(&mut self) {
        // SAFETY: callers replace control fonts or destroy the controls before dropping this owner.
        unsafe {
            let _ = DeleteObject(self.0.into());
        }
    }
}
pub(super) fn apply_font(child: HWND, font: HFONT) {
    // SAFETY: the owner retains the font for the lifetime of this control.
    unsafe {
        SendMessageW(
            child,
            WM_SETFONT,
            Some(windows::Win32::Foundation::WPARAM(font.0 as usize)),
            Some(windows::Win32::Foundation::LPARAM(1)),
        );
    }
}
pub(super) struct Resources {
    pub font: Font,
    pub images: HIMAGELIST,
}
impl Resources {
    pub(super) fn new(hwnd: HWND, color: COLORREF) -> Result<Self> {
        let font = Font::new(hwnd)?;
        // SAFETY: image list and its bitmaps are created and used on the UI thread.
        unsafe {
            let images = ImageList_Create(px(hwnd, 16), px(hwnd, 16), ILC_COLOR32, 4, 0);
            if images.is_invalid() {
                return Err(Error::from_thread());
            }
            let resources = Self { font, images };
            add_glyph(hwnd, images, '\u{E72B}', color)?;
            add_glyph(hwnd, images, '\u{E72A}', color)?;
            add_glyph(hwnd, images, '\u{E8A5}', color)?;
            add_glyph(hwnd, images, '\u{E713}', color)?;
            Ok(resources)
        }
    }
}
impl Drop for Resources {
    fn drop(&mut self) {
        // SAFETY: toolbar has been destroyed or switched to the replacement image list.
        unsafe {
            let _ = ImageList_Destroy(Some(self.images));
        }
    }
}
fn add_glyph(hwnd: HWND, images: HIMAGELIST, glyph: char, color: COLORREF) -> Result<()> {
    let side = px(hwnd, 16);
    // SAFETY: every GDI selection is restored; pixels remain valid until bitmap deletion.
    unsafe {
        let dc = CreateCompatibleDC(None);
        if dc.is_invalid() {
            return Err(Error::from_thread());
        }
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: side,
                biHeight: -side,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits = std::ptr::null_mut();
        let bitmap = match CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0) {
            Ok(bitmap) => bitmap,
            Err(error) => {
                let _ = DeleteDC(dc);
                return Err(error);
            }
        };
        let font = CreateFontW(
            -side,
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            ANTIALIASED_QUALITY,
            DEFAULT_PITCH.0 as u32,
            w!("Segoe Fluent Icons"),
        );
        if font.is_invalid() {
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(dc);
            return Err(Error::from_thread());
        }
        let old_bitmap = SelectObject(dc, bitmap.into());
        let old_font = SelectObject(dc, font.into());
        std::slice::from_raw_parts_mut(bits.cast::<u32>(), (side * side) as usize).fill(0x00FFFFFF);
        SetBkMode(dc, TRANSPARENT);
        SetTextColor(dc, COLORREF(0));
        let mut bounds = windows::Win32::Foundation::RECT {
            right: side,
            bottom: side,
            ..Default::default()
        };
        DrawTextW(
            dc,
            &mut [glyph as u16],
            &mut bounds,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
        );
        let _ = GdiFlush();
        let color = color.0;
        for pixel in std::slice::from_raw_parts_mut(bits.cast::<u32>(), (side * side) as usize) {
            let alpha = 255 - (*pixel & 255);
            let r = (color & 255) * alpha / 255;
            let g = ((color >> 8) & 255) * alpha / 255;
            let b = ((color >> 16) & 255) * alpha / 255;
            *pixel = (alpha << 24) | (r << 16) | (g << 8) | b;
        }
        SelectObject(dc, old_font);
        SelectObject(dc, old_bitmap);
        let index = ImageList_Add(images, bitmap, None);
        let _ = DeleteObject(font.into());
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(dc);
        if index < 0 {
            Err(Error::from_thread())
        } else {
            Ok(())
        }
    }
}
