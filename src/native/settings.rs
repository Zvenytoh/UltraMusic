//! Owned modeless settings using themed Windows controls and a real volume trackbar.
use super::{app, commands::*, ui, ui::px, App};
use std::mem::size_of;
use windows::{
    core::{w, Result, HSTRING},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::*,
            HiDpi::{AdjustWindowRectExForDpi, GetDpiForWindow},
            WindowsAndMessaging::*,
        },
    },
};
const CLASS: windows::core::PCWSTR = w!("UltraMusic.Settings.1");
// TBM_GETPOS is WM_USER + 0; windows metadata does not export this alias.
const TBM_GETPOS: u32 = WM_USER;
const SLIDER: usize = 40;
const PERCENT: usize = 41;
const STATUS: usize = 42;
const VERSION: usize = 43;
// Logical coordinates; native labels wrap within their measured font's layout area.
type ControlSpec = (usize, &'static str, &'static str, i32, i32, i32, i32, u32);
const ROWS: [ControlSpec; 13] = [
    (50, "BUTTON", "Audio", 16, 16, 424, 180, BS_GROUPBOX as u32),
    (51, "STATIC", "Volume de l’application", 32, 40, 280, 24, 0),
    (
        SLIDER,
        "msctls_trackbar32",
        "Volume",
        24,
        68,
        336,
        28,
        TBS_NOTICKS,
    ),
    (PERCENT, "STATIC", "100 %", 368, 72, 64, 24, 0),
    (
        MUTE,
        "BUTTON",
        "Couper le son",
        32,
        104,
        392,
        24,
        BS_CHECKBOX as u32,
    ),
    (
        COMPAT,
        "BUTTON",
        "Conserver le rendu en arrière-plan",
        32,
        136,
        392,
        24,
        BS_CHECKBOX as u32,
    ),
    (
        52,
        "STATIC",
        "Garde le rendu actif lorsque le lecteur est masqué.",
        32,
        164,
        392,
        24,
        0,
    ),
    (
        53,
        "BUTTON",
        "Protection YouTube",
        16,
        212,
        424,
        128,
        BS_GROUPBOX as u32,
    ),
    (
        BLOCKER,
        "BUTTON",
        "Bloquer les publicités YouTube",
        32,
        240,
        392,
        24,
        BS_CHECKBOX as u32,
    ),
    (
        STATUS,
        "STATIC",
        "Initialisation du module…",
        32,
        272,
        392,
        24,
        0,
    ),
    (
        54,
        "STATIC",
        "Modifier cette option recharge la page et interrompt la lecture.",
        32,
        302,
        392,
        32,
        0,
    ),
    (
        55,
        "BUTTON",
        "Application",
        16,
        356,
        424,
        100,
        BS_GROUPBOX as u32,
    ),
    (
        VERSION,
        "STATIC",
        concat!("UltraMusic ", env!("CARGO_PKG_VERSION")),
        32,
        380,
        392,
        24,
        0,
    ),
];
fn send(hwnd: HWND, msg: u32, wp: usize, lp: isize) -> LRESULT {
    // SAFETY: synchronous scalar control messages on the owning UI thread.
    unsafe { SendMessageW(hwnd, msg, Some(WPARAM(wp)), Some(LPARAM(lp))) }
}
impl App {
    pub(super) fn open_settings(&self) {
        // SAFETY: registered class and children belong to the owning UI thread.
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
                hbrBackground: GetSysColorBrush(COLOR_BTNFACE),
                ..Default::default()
            };
            RegisterClassW(&class);
            let style = WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_CLIPCHILDREN;
            let mut bounds = RECT {
                right: px(self.hwnd, 456),
                bottom: px(self.hwnd, 472),
                ..Default::default()
            };
            let _ = AdjustWindowRectExForDpi(
                &mut bounds,
                style,
                false,
                WS_EX_CONTROLPARENT,
                GetDpiForWindow(self.hwnd),
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
            if self.create_settings_controls(hwnd).is_err() {
                let _ = DestroyWindow(hwnd);
                super::show_error("Impossible de créer les paramètres.");
                return;
            }
            self.refresh_settings();
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(
                GetDlgItem(Some(hwnd), SLIDER as i32).ok(),
            );
        }
    }
    fn create_settings_controls(&self, hwnd: HWND) -> Result<()> {
        // SAFETY: native children are owned by settings; failed creation destroys their parent.
        unsafe {
            InitCommonControlsEx(&INITCOMMONCONTROLSEX {
                dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
                dwICC: ICC_BAR_CLASSES,
            })
            .ok()?;
            for (id, class, label, _, _, _, _, style) in ROWS {
                let tab = if matches!(id, SLIDER | MUTE | COMPAT | BLOCKER) {
                    WS_TABSTOP
                } else {
                    WINDOW_STYLE(0)
                };
                CreateWindowExW(
                    Default::default(),
                    &HSTRING::from(class),
                    &HSTRING::from(label),
                    WS_CHILD | WS_VISIBLE | tab | WINDOW_STYLE(style),
                    0,
                    0,
                    0,
                    0,
                    Some(hwnd),
                    Some(HMENU(id as *mut _)),
                    None,
                    None,
                )?;
            }
            CreateWindowExW(
                Default::default(),
                w!("BUTTON"),
                w!("Vérifier les mises à jour"),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                0,
                0,
                0,
                0,
                Some(hwnd),
                Some(HMENU(UPDATE as *mut _)),
                None,
                None,
            )?;
            let slider = GetDlgItem(Some(hwnd), SLIDER as i32)?;
            send(slider, TBM_SETRANGEMIN, 0, 0);
            send(slider, TBM_SETRANGEMAX, 0, 100);
            send(slider, TBM_SETPAGESIZE, 0, 10);
        }
        self.layout_settings(hwnd)
    }
    fn layout_settings(&self, hwnd: HWND) -> Result<()> {
        let font = ui::Font::new(hwnd)?;
        // SAFETY: only known controls are updated; the new font is installed before dropping the old one.
        unsafe {
            for (id, _, _, x, y, width, height, _) in ROWS {
                if let Ok(child) = GetDlgItem(Some(hwnd), id as i32) {
                    ui::apply_font(child, font.0);
                    let _ = SetWindowPos(
                        child,
                        None,
                        px(hwnd, x),
                        px(hwnd, y),
                        px(hwnd, width),
                        px(hwnd, height),
                        SWP_NOZORDER | SWP_NOACTIVATE,
                    );
                }
            }
            if let Ok(child) = GetDlgItem(Some(hwnd), UPDATE as i32) {
                ui::apply_font(child, font.0);
                let _ = SetWindowPos(
                    child,
                    None,
                    px(hwnd, 32),
                    px(hwnd, 412),
                    px(hwnd, 392),
                    px(hwnd, 28),
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
        let old = self.settings_font.replace(Some(font));
        drop(old);
        self.refresh_settings();
        Ok(())
    }
    pub(super) fn refresh_settings(&self) {
        let hwnd = self.settings.get();
        if hwnd.is_invalid() {
            return;
        }
        // SAFETY: all controls are owned by the current settings window.
        unsafe {
            for (id, checked) in [
                (MUTE, self.muted.get()),
                (COMPAT, self.keep_rendering.get()),
                (BLOCKER, self.blocker_enabled.get()),
            ] {
                if let Ok(child) = GetDlgItem(Some(hwnd), id as i32) {
                    send(
                        child,
                        BM_SETCHECK,
                        if checked {
                            BST_CHECKED.0 as usize
                        } else {
                            BST_UNCHECKED.0 as usize
                        },
                        0,
                    );
                }
            }
            let percent = (self.volume.get() * 100.0).round() as usize;
            if let Ok(slider) = GetDlgItem(Some(hwnd), SLIDER as i32) {
                send(slider, TBM_SETPOS, 1, percent as isize);
            }
            for (id, label) in [
                (PERCENT, format!("{percent} %")),
                (STATUS, self.blocker_status().to_string()),
                (
                    UPDATE,
                    if self.update_busy.get() {
                        "Afficher le résultat de la vérification…".to_string()
                    } else {
                        "Vérifier les mises à jour".to_string()
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
}
unsafe extern "system" fn settings_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    // SAFETY: Windows invokes this on the UI thread with message-scoped payloads.
    unsafe {
        let Some(app) = app() else {
            return DefWindowProcW(hwnd, msg, wp, lp);
        };
        match msg {
            WM_COMMAND => {
                let id = wp.0 & 0xffff;
                if wp.0 >> 16 == BN_CLICKED as usize
                    && matches!(id, MUTE | COMPAT | BLOCKER | UPDATE)
                {
                    app.command(id);
                }
            }
            WM_HSCROLL => {
                if let Ok(slider) = GetDlgItem(Some(hwnd), SLIDER as i32) {
                    if lp.0 == slider.0 as isize {
                        app.set_volume(send(slider, TBM_GETPOS, 0, 0).0 as f32 / 100.0);
                        app.refresh_ui();
                    }
                }
            }
            WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => {
                let dc = HDC(wp.0 as *mut _);
                SetTextColor(
                    dc,
                    windows::Win32::Foundation::COLORREF(GetSysColor(COLOR_BTNTEXT)),
                );
                SetBkColor(
                    dc,
                    windows::Win32::Foundation::COLORREF(GetSysColor(COLOR_BTNFACE)),
                );
                return LRESULT(GetSysColorBrush(COLOR_BTNFACE).0 as isize);
            }
            WM_CLOSE => {
                let _ = DestroyWindow(hwnd);
            }
            WM_DESTROY => {
                if app.settings.get() == hwnd {
                    app.settings.set(HWND::default());
                }
            }
            WM_NCDESTROY => {
                let old = app.settings_font.replace(None);
                drop(old);
                return DefWindowProcW(hwnd, msg, wp, lp);
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
                let _ = app.layout_settings(hwnd);
            }
            WM_THEMECHANGED | WM_SETTINGCHANGE => {
                let _ = app.layout_settings(hwnd);
                return DefWindowProcW(hwnd, msg, wp, lp);
            }
            _ => return DefWindowProcW(hwnd, msg, wp, lp),
        }
        LRESULT(0)
    }
}
