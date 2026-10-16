//! Notification icon and its native popup menu; no timers or rendering loop.
use super::{dialog, App, TRAY};
use std::mem::size_of;
use windows::{
    core::w,
    Win32::{
        Foundation::{LPARAM, POINT, WPARAM},
        UI::{
            Shell::{
                Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE,
                NOTIFYICONDATAW,
            },
            WindowsAndMessaging::*,
        },
    },
};

const OPEN: usize = 1;
const RESTART: usize = 2;
const COMPAT: usize = 3;
const QUIT: usize = 4;
impl App {
    fn tray_data(&self) -> NOTIFYICONDATAW {
        NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: 1,
            ..Default::default()
        }
    }
    pub(super) fn add_tray(&self) -> bool {
        let mut data = self.tray_data();
        data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        data.uCallbackMessage = TRAY;
        data.hIcon = self.icon;
        for (to, from) in data
            .szTip
            .iter_mut()
            .zip("UltraMusic — double-clic pour ouvrir".encode_utf16())
        {
            *to = from;
        }
        // SAFETY: cbSize is initialized and Shell copies this stack-allocated record.
        let added = unsafe { Shell_NotifyIconW(NIM_ADD, &data).as_bool() };
        self.tray.set(added);
        added
    }
    pub(super) fn remove_tray(&self) {
        if self.tray.replace(false) {
            // SAFETY: hwnd/uID identify only the notification icon owned by this app.
            unsafe {
                let _ = Shell_NotifyIconW(NIM_DELETE, &self.tray_data());
            }
        }
    }
    pub(super) fn menu(&self) {
        // SAFETY: the menu is owned locally and destroyed after TrackPopupMenu returns.
        // No RefCell borrow survives the nested native message loop.
        unsafe {
            let Ok(menu) = CreatePopupMenu() else { return };
            let _ = AppendMenuW(menu, MF_STRING, OPEN, w!("Ouvrir UltraMusic"));
            let _ = AppendMenuW(menu, MF_STRING, RESTART, w!("Redémarrer le lecteur…"));
            let flag = if self.keep_rendering.get() {
                MF_CHECKED
            } else {
                MF_UNCHECKED
            };
            let _ = AppendMenuW(
                menu,
                MF_STRING | flag,
                COMPAT,
                w!("Compatibilité : conserver le rendu"),
            );
            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
            let _ = AppendMenuW(menu, MF_STRING, QUIT, w!("Quitter"));
            let mut point = POINT::default();
            let _ = GetCursorPos(&mut point);
            let _ = SetForegroundWindow(self.hwnd);
            let command = TrackPopupMenu(
                menu,
                TPM_RETURNCMD | TPM_RIGHTBUTTON,
                point.x,
                point.y,
                None,
                self.hwnd,
                None,
            )
            .0 as usize;
            let _ = DestroyMenu(menu);
            let _ = PostMessageW(Some(self.hwnd), WM_NULL, WPARAM(0), LPARAM(0));
            match command {
                OPEN => self.restore(),
                RESTART => {
                    let message = "Redémarrer le lecteur ? La lecture sera interrompue ; la session sera conservée.";
                    let confirmed =
                        dialog(Some(self.hwnd), message, MB_YESNO | MB_ICONQUESTION) == IDYES;
                    if confirmed && self.start().is_err() {
                        self.report_failure();
                    }
                }
                COMPAT => {
                    self.keep_rendering.set(!self.keep_rendering.get());
                    if self.hidden.get() {
                        if let Some(c) = self.controller() {
                            let _ = c.SetIsVisible(self.keep_rendering.get());
                        }
                    }
                }
                QUIT => {
                    let _ = DestroyWindow(self.hwnd);
                }
                _ => {}
            }
        }
    }
}
