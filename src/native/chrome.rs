//! DPI-aware native header. DWM retains caption buttons and Snap behavior.
use super::{commands::*, drawing::*, App};
use crate::config::Service;
use std::mem::size_of;
use windows::{
    core::{w, Result, HSTRING},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::{Dwm::*, Gdi::*},
        UI::{Controls::*, HiDpi::*, Input::KeyboardAndMouse::*, Shell::*, WindowsAndMessaging::*},
    },
};

pub(super) const PICKER: usize = 20;
pub(super) const HEIGHT: i32 = 40;

/// Native BUTTONs retain keyboard, accessibility and pressed-state behavior.
pub(super) fn button(parent: HWND, id: usize, label: &str) -> Result<HWND> {
    // SAFETY: owned child windows and static subclass procedure live on the UI thread.
    unsafe {
        let hwnd = CreateWindowExW(
            Default::default(),
            w!("BUTTON"),
            &HSTRING::from(label),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW as u32),
            0,
            0,
            0,
            0,
            Some(parent),
            Some(HMENU(id as *mut _)),
            None,
            None,
        )?;
        if !SetWindowSubclass(hwnd, Some(button_proc), 1, 0).as_bool() {
            let error = windows::core::Error::from_thread();
            let _ = DestroyWindow(hwnd);
            return Err(error);
        }
        // Tooltip text is requested from the current native button name, avoiding
        // pointers to temporary Rust strings and keeping state changes accessible.
        if let Ok(tip) = CreateWindowExW(
            WS_EX_TOPMOST,
            TOOLTIPS_CLASSW,
            None,
            WS_POPUP | WINDOW_STYLE(TTS_ALWAYSTIP | TTS_NOPREFIX),
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            Some(hwnd),
            None,
            None,
            None,
        ) {
            let tool = TTTOOLINFOW {
                cbSize: size_of::<TTTOOLINFOW>() as u32,
                uFlags: TTF_IDISHWND | TTF_SUBCLASS,
                hwnd,
                uId: hwnd.0 as usize,
                lpszText: windows::core::PWSTR(-1isize as *mut u16),
                ..Default::default()
            };
            if SendMessageW(
                tip,
                TTM_ADDTOOLW,
                Some(WPARAM(0)),
                Some(LPARAM(&tool as *const _ as isize)),
            )
            .0 == 0
            {
                let _ = DestroyWindow(tip);
            } else {
                let _ = SetWindowSubclass(hwnd, Some(button_proc), 1, tip.0 as usize);
            }
        }
        Ok(hwnd)
    }
}

unsafe extern "system" fn button_proc(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
    _: usize,
    tip: usize,
) -> LRESULT {
    // SAFETY: Windows invokes this only for a live subclassed BUTTON.
    unsafe {
        match msg {
            WM_NOTIFY => {
                let header = &*(lp.0 as *const NMHDR);
                if header.code == TTN_GETDISPINFOW {
                    let info = &mut *(lp.0 as *mut NMTTDISPINFOW);
                    GetWindowTextW(hwnd, &mut info.szText);
                    info.lpszText = windows::core::PWSTR(info.szText.as_mut_ptr());
                    return LRESULT(0);
                }
            }
            WM_MOUSEMOVE => {
                let mut track = TRACKMOUSEEVENT {
                    cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    ..Default::default()
                };
                let _ = TrackMouseEvent(&mut track);
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
            WM_MOUSELEAVE | WM_SETFOCUS | WM_KILLFOCUS => {
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
            WM_NCDESTROY => {
                let _ = RemoveWindowSubclass(hwnd, Some(button_proc), 1);
                let tooltip = HWND(tip as *mut _);
                if !tooltip.is_invalid() && IsWindow(Some(tooltip)).as_bool() {
                    let _ = DestroyWindow(tooltip);
                }
            }
            _ => {}
        }
        DefSubclassProc(hwnd, msg, wp, lp)
    }
}

pub(super) fn button_background(item: &DRAWITEMSTRUCT) {
    // SAFETY: owner-draw data comes from the live button's WM_DRAWITEM.
    unsafe {
        let mut point = POINT::default();
        let mut bounds = RECT::default();
        let hovered = GetCursorPos(&mut point).is_ok()
            && GetWindowRect(item.hwndItem, &mut bounds).is_ok()
            && PtInRect(&bounds, point).as_bool();
        let color = if hovered || item.itemState.0 & ODS_SELECTED.0 != 0 {
            HOVER
        } else if item.CtlID as usize == SETTINGS {
            BLACK
        } else {
            PANEL
        };
        fill(item.hDC, &item.rcItem, BLACK);
        round(
            item.hDC,
            &item.rcItem,
            color,
            if item.itemState.0 & ODS_FOCUS.0 != 0 {
                WHITE
            } else if matches!(item.CtlID as usize, SETTINGS | PICKER) {
                color
            } else {
                LINE
            },
            px(item.hwndItem, 10),
        );
    }
}

impl App {
    pub(super) fn header_action(&self, action: isize) {
        if action == 1 {
            // SAFETY: the native service button belongs to this UI thread.
            unsafe {
                let _ = SetFocus(Some(self.service_button.get()));
            }
        } else if action == 2 {
            self.open_settings();
        }
    }

    pub(super) fn keyboard(&self, message: &MSG) -> bool {
        // SAFETY: message and focus handles are inspected only on the UI thread.
        unsafe {
            let root = GetAncestor(message.hwnd, GA_ROOT);
            if !self.settings.get().is_invalid() && root == self.settings.get() {
                if message.message == WM_KEYDOWN && message.wParam.0 == VK_ESCAPE.0 as usize {
                    let _ = DestroyWindow(root);
                    return true;
                }
                return IsDialogMessageW(root, message).as_bool();
            }
            if root != self.hwnd {
                return false;
            }
            let focus = GetFocus();
            let in_header =
                focus == self.service_button.get() || focus == self.settings_button.get();
            if message.message == WM_KEYDOWN {
                if message.wParam.0 == VK_F6.0 as usize {
                    if in_header {
                        self.focus_player();
                    } else {
                        let _ = SetFocus(Some(self.service_button.get()));
                    }
                    return true;
                }
                if in_header && message.wParam.0 == VK_ESCAPE.0 as usize {
                    self.focus_player();
                    return true;
                }
                if message.wParam.0 == VK_OEM_COMMA.0 as usize
                    && GetKeyState(VK_CONTROL.0 as i32) < 0
                {
                    self.open_settings();
                    return true;
                }
            }
            // Restrict dialog navigation to the header so webpage keyboard input stays native.
            in_header && IsDialogMessageW(self.hwnd, message).as_bool()
        }
    }

    fn focus_player(&self) {
        if let Some(controller) = self.controller() {
            // SAFETY: controller belongs to this UI STA.
            unsafe {
                let _ = controller.MoveFocus(webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_MOVE_FOCUS_REASON_PROGRAMMATIC);
            }
        }
    }

    pub(super) fn create_chrome(&self) -> Result<()> {
        // SAFETY: initialize the native tooltip class before creating header controls.
        unsafe {
            InitCommonControlsEx(&INITCOMMONCONTROLSEX {
                dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
                dwICC: ICC_WIN95_CLASSES,
            })
            .ok()?;
        }
        self.service_button
            .set(button(self.hwnd, PICKER, "Choisir le service")?);
        self.settings_button
            .set(button(self.hwnd, SETTINGS, "Paramètres")?);
        self.extend_frame();
        // SAFETY: recalculate nonclient layout now that App exists for WM_NCCALCSIZE.
        unsafe {
            SetWindowPos(
                self.hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
        }
        self.layout_chrome();
        self.refresh_ui();
        Ok(())
    }

    pub(super) fn extend_frame(&self) {
        // SAFETY: the margins are synchronous inputs to DWM for our live window.
        unsafe {
            let _ = DwmExtendFrameIntoClientArea(
                self.hwnd,
                &MARGINS {
                    // The full caption height is needed for DWM to draw its buttons.
                    cyTopHeight: px(self.hwnd, HEIGHT),
                    ..Default::default()
                },
            );
        }
    }

    fn caption_width(&self) -> i32 {
        let mut bounds = RECT::default();
        // SAFETY: DWM writes only the supplied RECT buffer.
        let result = unsafe {
            DwmGetWindowAttribute(
                self.hwnd,
                DWMWA_CAPTION_BUTTON_BOUNDS,
                &mut bounds as *mut _ as *mut _,
                size_of::<RECT>() as u32,
            )
        };
        if result.is_ok() && bounds.right > bounds.left {
            bounds.right - bounds.left
        } else {
            px(self.hwnd, 138)
        }
    }

    pub(super) fn layout_chrome(&self) {
        // SAFETY: both children belong to this window on the UI thread.
        unsafe {
            let mut bounds = RECT::default();
            let _ = GetClientRect(self.hwnd, &mut bounds);
            let x = bounds.right - self.caption_width() - px(self.hwnd, 40);
            let _ = SetWindowPos(
                self.settings_button.get(),
                None,
                x,
                px(self.hwnd, 6),
                px(self.hwnd, 28),
                px(self.hwnd, 28),
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            let _ = SetWindowPos(
                self.service_button.get(),
                None,
                px(self.hwnd, 44),
                px(self.hwnd, 6),
                px(self.hwnd, 188),
                px(self.hwnd, 28),
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            let header = rect(0, 0, bounds.right, px(self.hwnd, HEIGHT));
            let _ = InvalidateRect(Some(self.hwnd), Some(&header), false);
        }
    }

    pub(super) fn refresh_ui(&self) {
        // SAFETY: optional owned windows are used only while alive on this thread.
        unsafe {
            let button = self.service_button.get();
            if !button.is_invalid() {
                let _ = SetWindowTextW(
                    button,
                    &HSTRING::from(format!(
                        "Choisir le service : {}",
                        self.service.get().label()
                    )),
                );
                let _ = InvalidateRect(Some(button), None, false);
            }
        }
        self.refresh_settings();
    }

    pub(super) fn service_menu(&self) {
        // SAFETY: the popup owns no Rust pointers; it is destroyed after its modal loop.
        unsafe {
            let Ok(menu) = CreatePopupMenu() else { return };
            for (id, service) in SERVICES {
                // Keep a copied native label for accessibility, then enable custom painting.
                let _ = AppendMenuW(menu, MF_STRING, id, &HSTRING::from(service.label()));
                let info = MENUITEMINFOW {
                    cbSize: size_of::<MENUITEMINFOW>() as u32,
                    fMask: MIIM_FTYPE,
                    fType: MFT_OWNERDRAW,
                    ..Default::default()
                };
                let _ = SetMenuItemInfoW(menu, id as u32, false, &info);
            }
            let mut bounds = RECT::default();
            let _ = GetWindowRect(self.service_button.get(), &mut bounds);
            let _ = SetForegroundWindow(self.hwnd);
            let command = TrackPopupMenu(
                menu,
                TPM_RETURNCMD | TPM_LEFTALIGN,
                bounds.left,
                bounds.bottom + px(self.hwnd, 6),
                None,
                self.hwnd,
                None,
            )
            .0 as usize;
            let _ = DestroyMenu(menu);
            let _ = PostMessageW(Some(self.hwnd), WM_NULL, WPARAM(0), LPARAM(0));
            if command != 0 {
                self.command(command);
            }
        }
    }

    pub(super) fn draw_header_button(&self, item: &DRAWITEMSTRUCT) {
        let hwnd = item.hwndItem;
        button_background(item);
        if item.CtlID as usize == SETTINGS {
            icon(item.hDC, hwnd, Icon::Gear, px(hwnd, 3), px(hwnd, 3), WHITE);
        } else {
            icon(
                item.hDC,
                hwnd,
                service_icon(self.service.get()),
                px(hwnd, 6),
                px(hwnd, 3),
                RED,
            );
            text(
                item.hDC,
                hwnd,
                self.service.get().label(),
                rect(px(hwnd, 34), 0, px(hwnd, 124), px(hwnd, 28)),
                12,
                WHITE,
            );
            icon(
                item.hDC,
                hwnd,
                Icon::Chevron,
                px(hwnd, 160),
                px(hwnd, 3),
                MUTED,
            );
        }
    }

    pub(super) fn chrome_message(&self, msg: u32, wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
        // SAFETY: message-specific pointers come from Windows and live for this dispatch.
        unsafe {
            // DWM needs caption mouse/paint messages too, not only hit testing.
            // Our layout and client painting still take precedence below.
            let mut dwm_result = LRESULT(0);
            let dwm_handled = DwmDefWindowProc(self.hwnd, msg, wp, lp, &mut dwm_result).as_bool();
            if dwm_handled && !matches!(msg, WM_NCCALCSIZE | WM_PAINT | WM_GETMINMAXINFO) {
                return Some(dwm_result);
            }
            match msg {
                WM_NCCALCSIZE if wp.0 != 0 => {
                    if IsZoomed(self.hwnd).as_bool() {
                        let params = &mut *(lp.0 as *mut NCCALCSIZE_PARAMS);
                        let dpi = GetDpiForWindow(self.hwnd);
                        let x = GetSystemMetricsForDpi(SM_CXSIZEFRAME, dpi)
                            + GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi);
                        let y = GetSystemMetricsForDpi(SM_CYSIZEFRAME, dpi)
                            + GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi);
                        params.rgrc[0].left += x;
                        params.rgrc[0].right -= x;
                        params.rgrc[0].top += y;
                        params.rgrc[0].bottom -= y;
                    }
                    return Some(LRESULT(0));
                }
                WM_NCHITTEST => {
                    let mut point = POINT {
                        x: lp.0 as i16 as i32,
                        y: (lp.0 >> 16) as i16 as i32,
                    };
                    let _ = ScreenToClient(self.hwnd, &mut point);
                    let mut bounds = RECT::default();
                    let _ = GetClientRect(self.hwnd, &mut bounds);
                    let edge = px(self.hwnd, 6);
                    if !IsZoomed(self.hwnd).as_bool() {
                        let left = point.x < edge;
                        let right = point.x >= bounds.right - edge;
                        let top = point.y < edge;
                        let bottom = point.y >= bounds.bottom - edge;
                        let hit = match (left, right, top, bottom) {
                            (true, _, true, _) => HTTOPLEFT,
                            (_, true, true, _) => HTTOPRIGHT,
                            (true, _, _, true) => HTBOTTOMLEFT,
                            (_, true, _, true) => HTBOTTOMRIGHT,
                            (true, _, _, _) => HTLEFT,
                            (_, true, _, _) => HTRIGHT,
                            (_, _, true, _) => HTTOP,
                            (_, _, _, true) => HTBOTTOM,
                            _ => HTCLIENT,
                        };
                        if hit != HTCLIENT {
                            return Some(LRESULT(hit as isize));
                        }
                    }
                    if point.y < px(self.hwnd, HEIGHT) {
                        // Child buttons handle their own input; the surrounding header drags.
                        return Some(LRESULT(HTCAPTION as isize));
                    }
                    return Some(LRESULT(HTCLIENT as isize));
                }
                WM_GETMINMAXINFO => {
                    let info = &mut *(lp.0 as *mut MINMAXINFO);
                    info.ptMinTrackSize = POINT {
                        x: px(self.hwnd, 500),
                        y: px(self.hwnd, 440),
                    };
                    return Some(LRESULT(0));
                }
                WM_PAINT => {
                    let mut paint = PAINTSTRUCT::default();
                    let dc = BeginPaint(self.hwnd, &mut paint);
                    let mut bounds = RECT::default();
                    let _ = GetClientRect(self.hwnd, &mut bounds);
                    fill(dc, &bounds, BLACK);
                    let _ = DrawIconEx(
                        dc,
                        px(self.hwnd, 12),
                        px(self.hwnd, 10),
                        self.icon,
                        px(self.hwnd, 20),
                        px(self.hwnd, 20),
                        0,
                        None,
                        DI_NORMAL,
                    );
                    let _ = EndPaint(self.hwnd, &paint);
                    return Some(LRESULT(0));
                }
                WM_ERASEBKGND => return Some(LRESULT(1)),
                WM_MEASUREITEM => {
                    let item = &mut *(lp.0 as *mut MEASUREITEMSTRUCT);
                    if item.CtlType == ODT_MENU {
                        item.itemWidth = px(self.hwnd, 224) as u32;
                        item.itemHeight = px(self.hwnd, 36) as u32;
                        return Some(LRESULT(1));
                    }
                }
                WM_DRAWITEM => {
                    let item = &*(lp.0 as *const DRAWITEMSTRUCT);
                    if item.CtlType == ODT_BUTTON {
                        self.draw_header_button(item);
                    } else if item.CtlType == ODT_MENU {
                        let color = if item.itemState.0 & ODS_SELECTED.0 != 0 {
                            HOVER
                        } else {
                            PANEL
                        };
                        fill(item.hDC, &item.rcItem, color);
                        if let Some((_, service)) =
                            SERVICES.iter().find(|(id, _)| *id == item.itemID as usize)
                        {
                            let x = item.rcItem.left;
                            let y = item.rcItem.top;
                            icon(
                                item.hDC,
                                self.hwnd,
                                service_icon(*service),
                                x + px(self.hwnd, 8),
                                y + px(self.hwnd, 7),
                                RED,
                            );
                            text(
                                item.hDC,
                                self.hwnd,
                                service.label(),
                                rect(
                                    x + px(self.hwnd, 38),
                                    y,
                                    px(self.hwnd, 152),
                                    px(self.hwnd, 36),
                                ),
                                13,
                                WHITE,
                            );
                            if self.service.get() == *service {
                                icon(
                                    item.hDC,
                                    self.hwnd,
                                    Icon::Check,
                                    x + px(self.hwnd, 194),
                                    y + px(self.hwnd, 7),
                                    WHITE,
                                );
                            }
                        }
                    } else {
                        return None;
                    }
                    return Some(LRESULT(1));
                }
                WM_ACTIVATE | WM_DWMCOMPOSITIONCHANGED => self.extend_frame(),
                _ => {}
            }
        }
        None
    }
}

fn service_icon(service: Service) -> Icon {
    if service == Service::YoutubeVideo {
        Icon::Video
    } else {
        Icon::Music
    }
}
