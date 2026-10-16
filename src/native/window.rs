//! Native window transitions and dispatch. Modal UI runs outside COM callbacks.
use super::{app, dialog, show_error, App, EXTERNAL, FAILED, RESTORE, TRAY};
use webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_MOVE_FOCUS_REASON_PROGRAMMATIC;
use windows::{
    core::{w, HSTRING},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        UI::{Shell::ShellExecuteW, WindowsAndMessaging::*},
    },
};
impl App {
    pub(super) fn resize(&self) {
        if let Some(c) = self.controller() {
            let mut rect = RECT::default();
            // SAFETY: the live controller and its parent are owned by this UI thread.
            unsafe {
                if GetClientRect(self.hwnd, &mut rect).is_ok() {
                    let _ = c.SetBounds(rect);
                }
            }
        }
    }
    pub(super) fn restore(&self) {
        self.hidden.set(false);
        // SAFETY: all handles and COM interfaces belong to this application's STA.
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_RESTORE);
            if let Some(c) = self.controller() {
                let _ = c.SetIsVisible(true);
                let _ = c.MoveFocus(COREWEBVIEW2_MOVE_FOCUS_REASON_PROGRAMMATIC);
            }
            let _ = SetForegroundWindow(self.hwnd);
        }
        self.resize();
    }
    pub(super) fn hide(&self) {
        // Never strand a window if Shell_NotifyIcon failed.
        if !self.tray.get() && !self.add_tray() {
            let message = "La zone de notification est indisponible. Quitter UltraMusic et arrêter la lecture ?";
            if dialog(Some(self.hwnd), message, MB_YESNO | MB_ICONQUESTION) == IDYES {
                // SAFETY: the window was created on this thread and is still owned here.
                unsafe {
                    let _ = DestroyWindow(self.hwnd);
                }
            }
            return;
        }
        self.hidden.set(true);
        // SAFETY: visibility is updated on the owning UI thread without suspending audio.
        unsafe {
            if let Some(c) = self.controller() {
                let _ = c.SetIsVisible(self.keep_rendering.get());
            }
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }
}

/// Dispatches messages for the registered UltraMusic window class.
///
/// # Safety
/// Must be invoked by Windows on the owning UI thread with valid message payloads,
/// including the borrowed RECT pointer carried by WM_DPICHANGED.
pub(super) unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> LRESULT {
    // SAFETY: called by Windows for this registered class on the UI thread.
    // WM_DPICHANGED supplies a RECT valid for the duration of this dispatch.
    unsafe {
        let Some(a) = app() else {
            return DefWindowProcW(hwnd, msg, wp, lp);
        };
        if a.taskbar_created != 0 && msg == a.taskbar_created {
            if !a.add_tray() {
                a.restore();
            }
            return LRESULT(0);
        }
        match msg {
            WM_QUERYENDSESSION => return LRESULT(1),
            WM_ENDSESSION => {
                if wp.0 != 0 {
                    let _ = DestroyWindow(hwnd);
                }
            }
            WM_CLOSE => a.hide(),
            RESTORE => a.restore(),
            WM_SIZE => {
                if wp.0 == SIZE_MINIMIZED as usize {
                    a.hide();
                } else {
                    a.resize();
                }
            }
            WM_MOVE => {
                if let Some(c) = a.controller() {
                    let _ = c.NotifyParentWindowPositionChanged();
                }
            }
            WM_DPICHANGED => {
                let r = &*(lp.0 as *const RECT);
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    r.left,
                    r.top,
                    r.right - r.left,
                    r.bottom - r.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
                a.resize();
            }
            TRAY => match lp.0 as u32 {
                WM_LBUTTONDBLCLK => a.restore(),
                WM_RBUTTONUP | WM_CONTEXTMENU => a.menu(),
                _ => {}
            },
            FAILED if wp.0 == a.generation.get() => {
                a.restore();
                dialog(Some(hwnd), "Le lecteur WebView2 n’a pas pu démarrer ou un de ses processus a échoué. Vérifiez le runtime Microsoft Edge WebView2 et la connexion réseau. Utilisez « Redémarrer le lecteur » dans la zone de notification, ou relancez UltraMusic. Aucun redémarrage automatique n’est effectué pendant l’écoute.", MB_OK | MB_ICONERROR);
            }
            EXTERNAL if wp.0 == a.generation.get() => {
                let uri = a.external.borrow_mut().take();
                if let Some(uri) = uri {
                    let host = url::Url::parse(&uri)
                        .ok()
                        .and_then(|u| u.host_str().map(str::to_owned))
                        .unwrap_or_default();
                    if dialog(
                        Some(hwnd),
                        &format!("Ouvrir un lien vers {host} dans votre navigateur par défaut ?"),
                        MB_YESNO | MB_ICONQUESTION,
                    ) == IDYES
                        && wp.0 == a.generation.get()
                    {
                        let result = ShellExecuteW(
                            Some(hwnd),
                            w!("open"),
                            &HSTRING::from(uri),
                            None,
                            None,
                            SW_SHOWNORMAL,
                        );
                        if result.0 as isize <= 32 {
                            show_error("Impossible d’ouvrir le navigateur par défaut.");
                        }
                    }
                }
            }
            WM_DESTROY => {
                a.close_webview();
                a.remove_tray();
                PostQuitMessage(0);
            }
            _ => return DefWindowProcW(hwnd, msg, wp, lp),
        }
        LRESULT(0)
    }
}
