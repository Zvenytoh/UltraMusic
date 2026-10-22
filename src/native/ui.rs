//! Owned system fonts and alpha icons; handles outlive every control using them.

use std::mem::size_of;
use windows::{
    core::{Error, Result},
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
        Self::styled(hwnd, if heading { 16 } else { 14 }, heading)
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
                metrics.lfMessageFont.lfWeight = FW_MEDIUM.0 as i32;
            }
            metrics.lfMessageFont.lfQuality = CLEARTYPE_QUALITY;
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
            for icon in [Icon::Back, Icon::Forward, Icon::Workspace, Icon::Settings] {
                add_icon(hwnd, images, icon, color)?;
            }
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
#[derive(Clone, Copy)]
enum Icon {
    Back,
    Forward,
    Workspace,
    Settings,
}
fn add_icon(hwnd: HWND, images: HIMAGELIST, icon: Icon, color: COLORREF) -> Result<()> {
    // Supersampled vector strokes avoid dependence on private-use font glyphs.
    let side = px(hwnd, 16);
    let scale = 4;
    let render_side = side * scale;
    // SAFETY: both DIBs/DCs are owned locally; selections are restored before deletion.
    unsafe {
        let dc = CreateCompatibleDC(None);
        if dc.is_invalid() {
            return Err(Error::from_thread());
        }
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: render_side,
                biHeight: -render_side,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits = std::ptr::null_mut();
        let bitmap = match CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0) {
            Ok(b) => b,
            Err(e) => {
                let _ = DeleteDC(dc);
                return Err(e);
            }
        };
        let old_bitmap = SelectObject(dc, bitmap.into());
        std::slice::from_raw_parts_mut(bits.cast::<u32>(), (render_side * render_side) as usize)
            .fill(0x00ffffff);
        let pen = CreatePen(PS_SOLID, ((render_side + 6) / 12).max(1), COLORREF(0));
        if pen.is_invalid() {
            SelectObject(dc, old_bitmap);
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(dc);
            return Err(Error::from_thread());
        }
        let old_pen = SelectObject(dc, pen.into());
        let old_brush = SelectObject(dc, GetStockObject(NULL_BRUSH));
        let p = |v: i32| (v * render_side + 12) / 24;
        let stroke = |points: &[(i32, i32)]| {
            let points: Vec<windows::Win32::Foundation::POINT> = points
                .iter()
                .map(|&(x, y)| windows::Win32::Foundation::POINT { x: p(x), y: p(y) })
                .collect();
            let _ = Polyline(dc, &points);
        };
        match icon {
            Icon::Back => {
                stroke(&[(14, 5), (7, 12), (14, 19)]);
                stroke(&[(7, 12), (21, 12)]);
            }
            Icon::Forward => {
                stroke(&[(10, 5), (17, 12), (10, 19)]);
                stroke(&[(3, 12), (17, 12)]);
            }
            Icon::Workspace => {
                let _ = RoundRect(dc, p(3), p(4), p(21), p(20), p(4), p(4));
                stroke(&[(9, 4), (9, 20)]);
            }
            Icon::Settings => {
                let points: Vec<_> = (0..=32)
                    .map(|i| {
                        let angle = (i as f64 - 0.5) * std::f64::consts::TAU / 32.0;
                        let radius = if i % 4 == 1 || i % 4 == 2 { 10.0 } else { 8.0 };
                        windows::Win32::Foundation::POINT {
                            x: ((12.0 + radius * angle.cos()) * render_side as f64 / 24.0).round()
                                as i32,
                            y: ((12.0 + radius * angle.sin()) * render_side as f64 / 24.0).round()
                                as i32,
                        }
                    })
                    .collect();
                let _ = Polyline(dc, &points);
                let _ = Ellipse(dc, p(9), p(9), p(15), p(15));
            }
        }
        let _ = GdiFlush();
        SelectObject(dc, old_brush);
        SelectObject(dc, old_pen);
        SelectObject(dc, old_bitmap);
        let _ = DeleteObject(pen.into());
        info.bmiHeader.biWidth = side;
        info.bmiHeader.biHeight = -side;
        let mut output_bits = std::ptr::null_mut();
        let output = CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut output_bits, None, 0);
        let result = match output {
            Ok(output) => {
                let input = std::slice::from_raw_parts(
                    bits.cast::<u32>(),
                    (render_side * render_side) as usize,
                );
                let pixels = std::slice::from_raw_parts_mut(
                    output_bits.cast::<u32>(),
                    (side * side) as usize,
                );
                for y in 0..side {
                    for x in 0..side {
                        let mut alpha = 0;
                        for sy in 0..scale {
                            for sx in 0..scale {
                                alpha += 255
                                    - (input[((y * scale + sy) * render_side + x * scale + sx)
                                        as usize]
                                        & 255);
                            }
                        }
                        let alpha = alpha / (scale * scale) as u32;
                        let r = (color.0 & 255) * alpha / 255;
                        let g = ((color.0 >> 8) & 255) * alpha / 255;
                        let b = ((color.0 >> 16) & 255) * alpha / 255;
                        pixels[(y * side + x) as usize] = (alpha << 24) | (r << 16) | (g << 8) | b;
                    }
                }
                let index = ImageList_Add(images, output, None);
                let _ = DeleteObject(output.into());
                if index < 0 {
                    Err(Error::from_thread())
                } else {
                    Ok(())
                }
            }
            Err(error) => Err(error),
        };
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(dc);
        result
    }
}
/// Buffered-paint initialization is paired on the owning STA after all paint buffers finish.
pub(super) struct PaintRuntime;
impl PaintRuntime {
    pub(super) fn new() -> Result<Self> {
        // SAFETY: called once on the UI thread; Drop balances the successful initialization.
        unsafe {
            BufferedPaintInit()?;
        }
        Ok(Self)
    }
}
impl Drop for PaintRuntime {
    fn drop(&mut self) {
        // SAFETY: all temporary buffers have ended and this is the initializing UI thread.
        unsafe {
            let _ = BufferedPaintUnInit();
        }
    }
}
