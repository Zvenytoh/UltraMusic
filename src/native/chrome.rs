//! Native Windows title bar and themed client toolbar.
use super::{commands::*, drawing::*, App};
use std::mem::size_of;
use windows::{
    core::{w, Result, HSTRING},
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::*,
        UI::{
            Controls::*,
            Input::KeyboardAndMouse::{
                GetFocus, GetKeyState, SetFocus, VK_CONTROL, VK_ESCAPE, VK_F6, VK_OEM_COMMA,
            },
            WindowsAndMessaging::*,
        },
    },
};

pub(super) const PICKER: usize = 20;
pub(super) const HEIGHT: i32 = 40;

pub(super) fn button(parent: HWND, id: usize, label: &str) -> Result<HWND> {
    create_button(parent, id, label, true)
}

fn toolbar_button(parent: HWND, id: usize, label: &str) -> Result<HWND> {
    create_button(parent, id, label, false)
}

fn create_button(parent: HWND, id: usize, label: &str, owner_draw: bool) -> Result<HWND> {
    let style = if owner_draw {
        BS_OWNERDRAW
    } else {
        BS_PUSHBUTTON
    };
    // SAFETY: the button is an owned child of the live UI window.
    unsafe {
        CreateWindowExW(
            Default::default(),
            w!("BUTTON"),
            &HSTRING::from(label),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(style as u32),
            0,
            0,
            0,
            0,
            Some(parent),
            Some(HMENU(id as *mut _)),
            None,
            None,
        )
    }
}

pub(super) fn button_background(item: &DRAWITEMSTRUCT) {
    let selected = item.itemState.0 & ODS_SELECTED.0 != 0;
    let color = if selected { HOVER } else { PANEL };
    fill(item.hDC, &item.rcItem, BLACK);
    round(
        item.hDC,
        &item.rcItem,
        color,
        if item.itemState.0 & ODS_FOCUS.0 != 0 {
            WHITE
        } else {
            LINE
        },
        px(item.hwndItem, 10),
    );
}

impl App {
    pub(super) fn header_action(&self, action: isize) {
        if action == 1 {
            // SAFETY: the native service button belongs to this UI thread.
            unsafe {
                let _ = SetFocus(Some(self.back_button.get()));
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
            let in_header = focus == self.back_button.get()
                || focus == self.service_button.get()
                || focus == self.settings_button.get();
            if message.message == WM_KEYDOWN {
                if message.wParam.0 == VK_F6.0 as usize {
                    if in_header {
                        self.focus_player();
                    } else {
                        let _ = SetFocus(Some(self.back_button.get()));
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
            in_header && IsDialogMessageW(self.hwnd, message).as_bool()
        }
    }

    fn focus_player(&self) {
        if let Some(controller) = self.controller() {
            // SAFETY: controller belongs to this UI STA.
            unsafe {
                let _ = controller.MoveFocus(
                    webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_MOVE_FOCUS_REASON_PROGRAMMATIC,
                );
            }
        }
    }

    pub(super) fn create_chrome(&self) -> Result<()> {
        // SAFETY: initialize native common controls before creating toolbar buttons.
        unsafe {
            InitCommonControlsEx(&INITCOMMONCONTROLSEX {
                dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
                dwICC: ICC_WIN95_CLASSES,
            })
            .ok()?;
        }
        self.back_button
            .set(toolbar_button(self.hwnd, BACK, "← Retour")?);
        self.service_button
            .set(toolbar_button(self.hwnd, PICKER, "Choisir le service")?);
        self.settings_button
            .set(toolbar_button(self.hwnd, SETTINGS, "Paramètres")?);
        self.layout_chrome();
        self.refresh_ui();
        Ok(())
    }

    pub(super) fn layout_chrome(&self) {
        // SAFETY: child buttons belong to this window and all sizes are DPI-aware.
        unsafe {
            let mut bounds = RECT::default();
            let _ = GetClientRect(self.hwnd, &mut bounds);
            let height = px(self.hwnd, 30);
            let top = px(self.hwnd, 5);
            for (child, x, width) in [
                (self.back_button.get(), px(self.hwnd, 12), px(self.hwnd, 88)),
                (
                    self.service_button.get(),
                    px(self.hwnd, 108),
                    px(self.hwnd, 200),
                ),
                (
                    self.settings_button.get(),
                    bounds.right - px(self.hwnd, 112),
                    px(self.hwnd, 100),
                ),
            ] {
                let _ = SetWindowPos(
                    child,
                    None,
                    x,
                    top,
                    width,
                    height,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            let header = rect(0, 0, bounds.right, px(self.hwnd, HEIGHT));
            let _ = InvalidateRect(Some(self.hwnd), Some(&header), false);
        }
    }

    pub(super) fn refresh_ui(&self) {
        // SAFETY: the service button is owned by this live UI window.
        unsafe {
            let button = self.service_button.get();
            if !button.is_invalid() {
                let _ = SetWindowTextW(
                    button,
                    &HSTRING::from(format!("{} ▾", self.service.get().label())),
                );
            }
        }
        self.refresh_settings();
    }

    pub(super) fn service_menu(&self) {
        // SAFETY: the popup owns only native menu strings and is destroyed after use.
        unsafe {
            let Ok(menu) = CreatePopupMenu() else { return };
            for (id, service) in SERVICES {
                let selected = if self.service.get() == service {
                    MF_CHECKED
                } else {
                    MF_UNCHECKED
                };
                let _ = AppendMenuW(
                    menu,
                    MF_STRING | selected,
                    id,
                    &HSTRING::from(service.label()),
                );
            }
            let mut bounds = RECT::default();
            let _ = GetWindowRect(self.service_button.get(), &mut bounds);
            let _ = SetForegroundWindow(self.hwnd);
            let command = TrackPopupMenu(
                menu,
                TPM_RETURNCMD | TPM_LEFTALIGN,
                bounds.left,
                bounds.bottom,
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

    pub(super) fn chrome_message(&self, msg: u32, lp: LPARAM) -> Option<LRESULT> {
        // SAFETY: message pointers are borrowed from Windows for this dispatch only.
        unsafe {
            match msg {
                WM_GETMINMAXINFO => {
                    let info = &mut *(lp.0 as *mut MINMAXINFO);
                    info.ptMinTrackSize = windows::Win32::Foundation::POINT {
                        x: px(self.hwnd, 500),
                        y: px(self.hwnd, 440),
                    };
                    Some(LRESULT(0))
                }
                WM_PAINT => {
                    let mut paint = PAINTSTRUCT::default();
                    let dc = BeginPaint(self.hwnd, &mut paint);
                    let color = COLORREF(GetSysColor(COLOR_BTNFACE));
                    let mut header = rect(0, 0, 0, px(self.hwnd, HEIGHT));
                    let _ = GetClientRect(self.hwnd, &mut header);
                    header.bottom = px(self.hwnd, HEIGHT).min(header.bottom);
                    fill(dc, &header, color);
                    let _ = EndPaint(self.hwnd, &paint);
                    Some(LRESULT(0))
                }
                WM_ERASEBKGND => Some(LRESULT(1)),
                _ => None,
            }
        }
    }
}
