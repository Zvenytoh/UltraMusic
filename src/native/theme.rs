//! Native caption styling; no custom title-bar renderer or periodic work.
use std::mem::size_of;
use windows::{
    core::BOOL,
    Win32::{
        Foundation::{COLORREF, HWND},
        Graphics::Dwm::{
            DwmSetWindowAttribute, DWMWA_CAPTION_COLOR, DWMWA_TEXT_COLOR,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
        },
    },
};

pub(super) fn apply(hwnd: HWND) {
    let dark = BOOL(1);
    let background = COLORREF(0x000000);
    let text = COLORREF(0xFFFFFF);
    // SAFETY: DWM copies these correctly sized values synchronously. The window
    // belongs to the calling UI thread. Older systems may ignore unsupported colors.
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&dark as *const BOOL).cast(),
            size_of::<BOOL>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_COLOR,
            (&background as *const COLORREF).cast(),
            size_of::<COLORREF>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_TEXT_COLOR,
            (&text as *const COLORREF).cast(),
            size_of::<COLORREF>() as u32,
        );
    }
}
