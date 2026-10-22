//! DWM title-row integration. Windows still owns all caption buttons and resize actions.
use super::{chrome::HEIGHT, ui::px, App};
use std::mem::size_of;
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, POINT, RECT, WPARAM},
    Graphics::{Dwm::*, Gdi::*},
    UI::{
        Controls::MARGINS,
        HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi},
        WindowsAndMessaging::*,
    },
};
impl App {
    pub(super) fn caption_bounds(&self) -> Option<RECT> {
        // SAFETY: DWM writes a RECT; the synchronous conversion stays on the owning UI thread.
        unsafe {
            let mut caption = RECT::default();
            if DwmGetWindowAttribute(
                self.hwnd,
                DWMWA_CAPTION_BUTTON_BOUNDS,
                (&mut caption as *mut RECT).cast(),
                size_of::<RECT>() as u32,
            )
            .is_err()
            {
                return None;
            }
            let mut window = RECT::default();
            GetWindowRect(self.hwnd, &mut window).ok()?;
            let mut origin = POINT {
                x: window.left,
                y: window.top,
            };
            let _ = ScreenToClient(self.hwnd, &mut origin);
            // Maximization clips the resize frame, while DWM keeps its caption bounds uninset.
            if IsZoomed(self.hwnd).as_bool() {
                let dpi = GetDpiForWindow(self.hwnd);
                origin.y += GetSystemMetricsForDpi(SM_CYSIZEFRAME, dpi)
                    + GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi);
            }
            caption.left += origin.x;
            caption.right += origin.x;
            caption.top += origin.y;
            caption.bottom += origin.y;
            (caption.right > caption.left && caption.bottom > caption.top).then_some(caption)
        }
    }
    pub(super) fn clear_caption_surface(&self, dc: HDC) {
        if let Some(caption) = self.caption_bounds() {
            // Zero-alpha black exposes the native DWM caption buttons above our painted surface.
            super::theme::fill(dc, &caption, windows::Win32::Foundation::COLORREF(0));
        }
    }
    pub(super) fn configure_frame(&self) {
        // SAFETY: all frame updates are synchronous and confined to the owning UI thread.
        unsafe {
            let _ = DwmExtendFrameIntoClientArea(
                self.hwnd,
                &MARGINS {
                    cyTopHeight: px(self.hwnd, HEIGHT),
                    ..Default::default()
                },
            );
            let _ = SetWindowPos(
                self.hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
        }
    }
    pub(super) fn frame_message(&self, msg: u32, wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
        // SAFETY: message-specific pointers are valid only during this window-procedure dispatch.
        unsafe {
            let mut result = LRESULT(0);
            // DWM must see non-client messages first, including WM_NCMOUSELEAVE.
            if DwmDefWindowProc(self.hwnd, msg, wp, lp, &mut result).as_bool() {
                return Some(result);
            }
            match msg {
                WM_NCCALCSIZE if wp.0 != 0 => {
                    let top = (*(lp.0 as *mut NCCALCSIZE_PARAMS)).rgrc[0].top;
                    DefWindowProcW(self.hwnd, msg, wp, lp);
                    let dpi = GetDpiForWindow(self.hwnd);
                    let inset = if IsZoomed(self.hwnd).as_bool() {
                        GetSystemMetricsForDpi(SM_CYSIZEFRAME, dpi)
                            + GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi)
                    } else {
                        0
                    };
                    (*(lp.0 as *mut NCCALCSIZE_PARAMS)).rgrc[0].top = top + inset;
                    Some(LRESULT(0))
                }
                WM_NCHITTEST => {
                    let point = POINT {
                        x: (lp.0 as u16 as i16) as i32,
                        y: ((lp.0 >> 16) as u16 as i16) as i32,
                    };
                    let mut client = point;
                    let _ = ScreenToClient(self.hwnd, &mut client);
                    let default = DefWindowProcW(self.hwnd, msg, wp, lp);
                    if default.0 != HTCLIENT as isize && default.0 != HTCAPTION as isize {
                        return Some(default);
                    }
                    if !IsZoomed(self.hwnd).as_bool() && client.y < px(self.hwnd, 6) {
                        let mut bounds = RECT::default();
                        let _ = GetClientRect(self.hwnd, &mut bounds);
                        return Some(LRESULT(if client.x < px(self.hwnd, 8) {
                            HTTOPLEFT
                        } else if client.x >= bounds.right - px(self.hwnd, 8) {
                            HTTOPRIGHT
                        } else {
                            HTTOP
                        } as isize));
                    }
                    // Reserve the exact DWM caption rectangle even if the toolbar is resized.
                    if let Some(caption) = self.caption_bounds() {
                        if client.x >= caption.left
                            && client.x < caption.right
                            && client.y >= caption.top
                            && client.y < caption.bottom
                        {
                            let slot = (client.x - caption.left) * 3
                                / (caption.right - caption.left).max(1);
                            return Some(LRESULT(match slot {
                                0 => HTMINBUTTON,
                                1 => HTMAXBUTTON,
                                _ => HTCLOSE,
                            } as isize));
                        }
                    }
                    if client.y >= 0 && client.y < px(self.hwnd, HEIGHT) {
                        Some(LRESULT(HTCAPTION as isize))
                    } else {
                        Some(default)
                    }
                }
                _ => None,
            }
        }
    }
}
