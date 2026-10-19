//! Owned modeless settings window; controls reuse the tray's application actions.
use super::{
    app,
    chrome::{button, button_background},
    commands::*,
    drawing::*,
    App,
};
use windows::{
    core::{w, HSTRING},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{Controls::*, HiDpi::AdjustWindowRectExForDpi, WindowsAndMessaging::*},
    },
};

const CLASS: windows::core::PCWSTR = w!("UltraMusic.Settings.1");
const ROWS: [(usize, &str, i32, i32, i32, i32); 5] = [
    (VOLUME_DOWN, "Diminuer le volume", 24, 92, 36, 36),
    (VOLUME_UP, "Augmenter le volume", 396, 92, 36, 36),
    (MUTE, "Couper le son", 24, 150, 408, 44),
    (COMPAT, "Conserver le rendu", 24, 234, 408, 48),
    (UPDATE, "Vérifier les mises à jour", 24, 374, 408, 42),
];

impl App {
    pub(super) fn open_settings(&self) {
        // SAFETY: this owned modeless window lives entirely on the same UI thread.
        unsafe {
            let existing = self.settings.get();
            if !existing.is_invalid() {
                let _ = SetForegroundWindow(existing);
                return;
            }
            let instance = GetModuleHandleW(None)
                .ok()
                .map(|h| windows::Win32::Foundation::HINSTANCE(h.0));
            let class = WNDCLASSW {
                hInstance: instance.unwrap_or_default(),
                lpszClassName: CLASS,
                lpfnWndProc: Some(settings_proc),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                hIcon: self.icon,
                ..Default::default()
            };
            // Reopening uses the same registered class.
            RegisterClassW(&class);
            let style = WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_CLIPCHILDREN;
            let mut bounds = rect(0, 0, px(self.hwnd, 456), px(self.hwnd, 440));
            let _ = AdjustWindowRectExForDpi(
                &mut bounds,
                style,
                false,
                WS_EX_CONTROLPARENT,
                windows::Win32::UI::HiDpi::GetDpiForWindow(self.hwnd),
            );
            let mut parent = RECT::default();
            let _ = GetWindowRect(self.hwnd, &mut parent);
            let width = bounds.right - bounds.left;
            let height = bounds.bottom - bounds.top;
            let result = CreateWindowExW(
                WS_EX_CONTROLPARENT,
                CLASS,
                w!("Paramètres — UltraMusic"),
                style,
                parent.left + (parent.right - parent.left - width) / 2,
                parent.top + (parent.bottom - parent.top - height) / 2,
                width,
                height,
                Some(self.hwnd),
                None,
                instance,
                None,
            );
            let Ok(hwnd) = result else {
                super::show_error("Impossible d’ouvrir les paramètres.");
                return;
            };
            self.settings.set(hwnd);
            super::theme::apply(hwnd);
            for (id, label, _, _, _, _) in ROWS {
                if button(hwnd, id, label).is_err() {
                    let _ = DestroyWindow(hwnd);
                    super::show_error("Impossible de créer les commandes des paramètres.");
                    return;
                }
            }
            layout(hwnd);
            self.refresh_settings();
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(
                GetDlgItem(Some(hwnd), VOLUME_DOWN as i32).ok(),
            );
        }
    }

    pub(super) fn refresh_settings(&self) {
        let hwnd = self.settings.get();
        if hwnd.is_invalid() {
            return;
        }
        // SAFETY: windows and children belong to this UI thread; no borrow spans calls.
        unsafe {
            for (id, action) in [(VOLUME_DOWN, "Diminuer"), (VOLUME_UP, "Augmenter")] {
                if let Ok(child) = GetDlgItem(Some(hwnd), id as i32) {
                    let _ = SetWindowTextW(
                        child,
                        &HSTRING::from(format!(
                            "{action} le volume ({} %)",
                            (self.volume.get() * 100.0).round() as u32
                        )),
                    );
                }
            }
            for (id, label) in [
                (
                    MUTE,
                    if self.muted.get() {
                        "Réactiver le son"
                    } else {
                        "Couper le son"
                    },
                ),
                (
                    COMPAT,
                    if self.keep_rendering.get() {
                        "Conserver le rendu : activé"
                    } else {
                        "Conserver le rendu : désactivé"
                    },
                ),
                (
                    UPDATE,
                    if self.update_busy.get() {
                        "Afficher le résultat de la vérification…"
                    } else {
                        "Vérifier les mises à jour"
                    },
                ),
            ] {
                if let Ok(child) = GetDlgItem(Some(hwnd), id as i32) {
                    let _ = SetWindowTextW(child, &HSTRING::from(label));
                }
            }
            let _ = RedrawWindow(Some(hwnd), None, None, RDW_INVALIDATE | RDW_ALLCHILDREN);
        }
    }

    fn paint_settings(&self, hwnd: HWND, dc: HDC) {
        let r = |x, y, w, h| rect(px(hwnd, x), px(hwnd, y), px(hwnd, w), px(hwnd, h));
        fill(dc, &r(0, 0, 456, 440), BLACK);
        text(dc, hwnd, "Audio", r(24, 18, 408, 28), 19, WHITE);
        text(
            dc,
            hwnd,
            &format!(
                "Volume de l’application · {} %",
                (self.volume.get() * 100.0).round() as u32
            ),
            r(24, 54, 408, 28),
            13,
            MUTED,
        );
        round(dc, &r(80, 107, 296, 6), LINE, LINE, px(hwnd, 6));
        round(
            dc,
            &r(80, 107, (296.0 * self.volume.get()).round() as i32, 6),
            RED,
            RED,
            px(hwnd, 6),
        );
        round(
            dc,
            &r(75 + (296.0 * self.volume.get()).round() as i32, 102, 12, 16),
            WHITE,
            WHITE,
            px(hwnd, 10),
        );
        text(
            dc,
            hwnd,
            "Lecture en arrière-plan",
            r(24, 204, 408, 26),
            13,
            MUTED,
        );
        text(
            dc,
            hwnd,
            "Garde le rendu actif lorsque la fenêtre est masquée.",
            r(24, 288, 408, 24),
            11,
            MUTED,
        );
        text(dc, hwnd, "Application", r(24, 326, 408, 24), 13, MUTED);
        text(
            dc,
            hwnd,
            concat!("UltraMusic ", env!("CARGO_PKG_VERSION")),
            r(24, 350, 408, 20),
            12,
            WHITE,
        );
    }

    fn draw_setting(&self, item: &DRAWITEMSTRUCT) {
        let hwnd = item.hwndItem;
        button_background(item);
        let id = item.CtlID as usize;
        if id == VOLUME_DOWN || id == VOLUME_UP {
            icon(
                item.hDC,
                hwnd,
                if id == VOLUME_DOWN {
                    Icon::Minus
                } else {
                    Icon::Plus
                },
                px(hwnd, 7),
                px(hwnd, 7),
                WHITE,
            );
        } else {
            let selected =
                (id == MUTE && self.muted.get()) || (id == COMPAT && self.keep_rendering.get());
            if selected {
                icon(
                    item.hDC,
                    hwnd,
                    Icon::Check,
                    px(hwnd, 14),
                    (item.rcItem.bottom - px(hwnd, 22)) / 2,
                    RED,
                );
            }
            let label = match id {
                MUTE => {
                    if self.muted.get() {
                        "Réactiver le son"
                    } else {
                        "Couper le son"
                    }
                }
                COMPAT => "Conserver le rendu",
                _ => {
                    if self.update_busy.get() {
                        "Afficher le résultat de la vérification…"
                    } else {
                        "Vérifier les mises à jour"
                    }
                }
            };
            text(
                item.hDC,
                hwnd,
                label,
                rect(
                    px(hwnd, 46),
                    0,
                    item.rcItem.right - px(hwnd, 60),
                    item.rcItem.bottom,
                ),
                13,
                WHITE,
            );
        }
    }
}

fn layout(hwnd: HWND) {
    // SAFETY: fixed known child IDs belong to this settings window.
    unsafe {
        for (id, _, x, y, w, h) in ROWS {
            if let Ok(child) = GetDlgItem(Some(hwnd), id as i32) {
                let _ = SetWindowPos(
                    child,
                    None,
                    px(hwnd, x),
                    px(hwnd, y),
                    px(hwnd, w),
                    px(hwnd, h),
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
    }
}

unsafe extern "system" fn settings_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    // SAFETY: called by Windows; payload pointers are borrowed only during dispatch.
    unsafe {
        let Some(app) = app() else {
            return DefWindowProcW(hwnd, msg, wp, lp);
        };
        match msg {
            WM_COMMAND => {
                let id = wp.0 & 0xffff;
                if ROWS.iter().any(|(command, ..)| *command == id) {
                    app.command(id);
                }
            }
            WM_CLOSE => {
                let _ = DestroyWindow(hwnd);
            }
            WM_DESTROY => {
                if app.settings.get() == hwnd {
                    app.settings.set(HWND::default());
                }
            }
            WM_ERASEBKGND => return LRESULT(1),
            WM_PAINT => {
                let mut paint = PAINTSTRUCT::default();
                let dc = BeginPaint(hwnd, &mut paint);
                app.paint_settings(hwnd, dc);
                let _ = EndPaint(hwnd, &paint);
            }
            WM_DRAWITEM => {
                app.draw_setting(&*(lp.0 as *const DRAWITEMSTRUCT));
                return LRESULT(1);
            }
            WM_LBUTTONDOWN => {
                let x = lp.0 as i16 as i32;
                let y = (lp.0 >> 16) as i16 as i32;
                if x >= px(hwnd, 80)
                    && x <= px(hwnd, 376)
                    && y >= px(hwnd, 96)
                    && y <= px(hwnd, 128)
                {
                    app.set_volume((x - px(hwnd, 80)) as f32 / px(hwnd, 296) as f32);
                    app.refresh_ui();
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
                layout(hwnd);
                app.refresh_settings();
            }
            WM_THEMECHANGED | WM_SETTINGCHANGE => super::theme::apply(hwnd),
            _ => return DefWindowProcW(hwnd, msg, wp, lp),
        }
        LRESULT(0)
    }
}
