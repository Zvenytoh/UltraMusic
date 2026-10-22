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
pub(super) const SLIDER: usize = 40;
pub(super) const PERCENT: usize = 41;
pub(super) const STATUS: usize = 42;
pub(super) const VERSION: usize = 43;
// Native children; geometry is measured and owned by settings_layout.rs.
const ROWS: [(usize, &str, &str, u32); 17] = [
    (56, "STATIC", "Apparence", 0),
    (THEME_DARK, "BUTTON", "Sombre", BS_RADIOBUTTON as u32),
    (THEME_LIGHT, "BUTTON", "Clair", BS_RADIOBUTTON as u32),
    (THEME_SYSTEM, "BUTTON", "Système", BS_RADIOBUTTON as u32),
    (50, "STATIC", "Audio", 0),
    (51, "STATIC", "Volume de l’application", 0),
    (SLIDER, "msctls_trackbar32", "Volume", TBS_NOTICKS),
    (PERCENT, "STATIC", "100 %", 0),
    (
        MUTE,
        "BUTTON",
        "Couper le son",
        (BS_CHECKBOX | BS_MULTILINE) as u32,
    ),
    (
        COMPAT,
        "BUTTON",
        "Conserver le rendu en arrière-plan",
        (BS_CHECKBOX | BS_MULTILINE) as u32,
    ),
    (
        52,
        "STATIC",
        "Garde le rendu actif lorsque le lecteur est masqué.",
        0,
    ),
    (53, "STATIC", "Protection YouTube", 0),
    (
        BLOCKER,
        "BUTTON",
        "Bloquer les publicités YouTube",
        (BS_CHECKBOX | BS_MULTILINE) as u32,
    ),
    (STATUS, "STATIC", "Initialisation du module…", 0),
    (
        54,
        "STATIC",
        "Modifier cette option recharge la page et interrompt la lecture.",
        0,
    ),
    (55, "STATIC", "Application", 0),
    (
        VERSION,
        "STATIC",
        concat!("UltraMusic ", env!("CARGO_PKG_VERSION")),
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
                hbrBackground: GetSysColorBrush(COLOR_WINDOW),
                ..Default::default()
            };
            RegisterClassW(&class);
            let style =
                WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_CLIPCHILDREN | WS_THICKFRAME | WS_VSCROLL;
            let mut bounds = RECT {
                right: px(self.hwnd, 500),
                bottom: px(self.hwnd, 620),
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
            let mut placement = RECT {
                left: parent.left + (parent.right - parent.left - width) / 2,
                top: parent.top + (parent.bottom - parent.top - height) / 2,
                ..Default::default()
            };
            placement.right = placement.left + width;
            placement.bottom = placement.top + height;
            super::settings_layout::fit_to_work_area(&mut placement);
            let result = CreateWindowExW(
                WS_EX_CONTROLPARENT,
                CLASS,
                w!("Paramètres — UltraMusic"),
                style,
                placement.left,
                placement.top,
                placement.right - placement.left,
                placement.bottom - placement.top,
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
            super::theme::caption(hwnd, self.colors());
            self.settings_scroll.set(0);
            self.settings_wheel.set(0);
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
            for (id, class, label, style) in ROWS {
                let tab = if matches!(
                    id,
                    SLIDER | MUTE | COMPAT | BLOCKER | THEME_DARK | THEME_LIGHT | THEME_SYSTEM
                ) {
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
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_MULTILINE as u32),
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
            // Keep the native trackbar's input behavior, with palette-owned background painting.
            let _ = SetWindowTheme(slider, w!(""), w!(""));
            send(slider, TBM_SETRANGEMIN, 0, 0);
            send(slider, TBM_SETRANGEMAX, 0, 100);
            send(slider, TBM_SETPAGESIZE, 0, 10);
        }
        self.refresh_settings_font(hwnd)
    }
    fn refresh_settings_font(&self, hwnd: HWND) -> Result<()> {
        let font = ui::Font::settings(hwnd, false)?;
        let heading = ui::Font::settings(hwnd, true)?;
        // SAFETY: only known controls are updated; the new font is installed before dropping the old one.
        unsafe {
            for (id, _, _, _) in ROWS {
                if let Ok(child) = GetDlgItem(Some(hwnd), id as i32) {
                    ui::apply_font(
                        child,
                        if matches!(id, 50 | 53 | 55 | 56) {
                            heading.0
                        } else {
                            font.0
                        },
                    );
                }
            }
            if let Ok(child) = GetDlgItem(Some(hwnd), UPDATE as i32) {
                ui::apply_font(child, font.0);
            }
        }
        let old = self.settings_font.replace(Some(font));
        let old_heading = self.settings_heading_font.replace(Some(heading));
        drop(old);
        drop(old_heading);
        self.layout_settings(hwnd);
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
                (
                    THEME_DARK,
                    self.appearance.get() == super::theme::Choice::Dark,
                ),
                (
                    THEME_LIGHT,
                    self.appearance.get() == super::theme::Choice::Light,
                ),
                (
                    THEME_SYSTEM,
                    self.appearance.get() == super::theme::Choice::System,
                ),
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
            let mut changed_layout = false;
            for (id, label) in [
                (PERCENT, format!("{percent} %")),
                (STATUS, self.blocker_status().to_string()),
                (UPDATE, self.update_label().to_string()),
            ] {
                if let Ok(child) = GetDlgItem(Some(hwnd), id as i32) {
                    if super::settings_layout::control_text(child) != label {
                        let _ = SetWindowTextW(child, &HSTRING::from(label));
                        changed_layout |= id == STATUS || id == UPDATE;
                    }
                }
            }
            if let Ok(button) = GetDlgItem(Some(hwnd), COMPAT as i32) {
                let _ = windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow(
                    button,
                    self.update_phase.get() != super::updates::Phase::Preparing,
                );
            }
            if let Ok(button) = GetDlgItem(Some(hwnd), UPDATE as i32) {
                let _ = windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow(
                    button,
                    !matches!(
                        self.update_phase.get(),
                        super::updates::Phase::Downloading | super::updates::Phase::Preparing
                    ),
                );
            }
            if changed_layout {
                self.layout_settings(hwnd);
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
                    && matches!(
                        id,
                        MUTE | COMPAT | BLOCKER | UPDATE | THEME_DARK | THEME_LIGHT | THEME_SYSTEM
                    )
                {
                    app.command(id);
                }
            }
            WM_NOTIFY => {
                if let Some(result) = app.draw_settings_control(lp) {
                    return result;
                }
                return DefWindowProcW(hwnd, msg, wp, lp);
            }
            WM_ERASEBKGND => {
                app.paint_settings_background(HDC(wp.0 as *mut _));
                return LRESULT(1);
            }
            WM_SIZE => app.layout_settings(hwnd),
            WM_VSCROLL => app.settings_vscroll(wp),
            WM_MOUSEWHEEL => {
                let delta = ((wp.0 >> 16) as u16 as i16) as i32 + app.settings_wheel.get();
                app.settings_wheel.set(delta % 120);
                app.scroll_settings(app.settings_scroll.get() - (delta / 120) * px(hwnd, 48));
                return LRESULT(0);
            }
            WM_GETMINMAXINFO => {
                let info = &mut *(lp.0 as *mut MINMAXINFO);
                let mut work = RECT {
                    left: 0,
                    top: 0,
                    right: px(hwnd, 320),
                    bottom: px(hwnd, 160),
                };
                let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
                let mut bounds = MONITORINFO {
                    cbSize: size_of::<MONITORINFO>() as u32,
                    ..Default::default()
                };
                if GetMonitorInfoW(monitor, &mut bounds).as_bool() {
                    info.ptMaxTrackSize.x = bounds.rcWork.right - bounds.rcWork.left;
                    info.ptMaxTrackSize.y = bounds.rcWork.bottom - bounds.rcWork.top;
                    work.right = work.right.min(bounds.rcWork.right - bounds.rcWork.left);
                    work.bottom = work.bottom.min(bounds.rcWork.bottom - bounds.rcWork.top);
                }
                info.ptMinTrackSize.x = work.right;
                info.ptMinTrackSize.y = work.bottom;
                return LRESULT(0);
            }
            WM_HSCROLL => {
                if let Ok(slider) = GetDlgItem(Some(hwnd), SLIDER as i32) {
                    if lp.0 == slider.0 as isize {
                        app.begin_volume_gesture();
                        app.set_live_volume(send(slider, TBM_GETPOS, 0, 0).0 as f32 / 100.0);
                        if wp.0 & 0xffff == TB_ENDTRACK as usize {
                            app.finish_volume_gesture();
                        }
                        app.refresh_settings();
                    }
                }
            }
            WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => {
                let dc = HDC(wp.0 as *mut _);
                let id = GetDlgCtrlID(HWND(lp.0 as *mut _));
                let enabled = windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled(HWND(
                    lp.0 as *mut _,
                ))
                .as_bool();
                let colors = app.colors();
                SetTextColor(
                    dc,
                    if !enabled || matches!(id, 52 | 54 | 43) {
                        colors.secondary
                    } else {
                        colors.text
                    },
                );
                SetBkColor(dc, colors.background);
                SetBkMode(dc, TRANSPARENT);
                let brush = app.palette.borrow().brush;
                return LRESULT(brush.0 as isize);
            }
            WM_CLOSE => {
                let _ = DestroyWindow(hwnd);
            }
            WM_DESTROY => {
                app.finish_volume_gesture();
                app.settings_positions.borrow_mut().clear();
                if app.settings.get() == hwnd {
                    app.settings.set(HWND::default());
                }
            }
            WM_NCDESTROY => {
                let old = app.settings_font.replace(None);
                let old_heading = app.settings_heading_font.replace(None);
                drop(old);
                drop(old_heading);
                return DefWindowProcW(hwnd, msg, wp, lp);
            }
            WM_DPICHANGED => {
                let mut rect = *(lp.0 as *const RECT);
                super::settings_layout::fit_to_work_area(&mut rect);
                let r = &rect;
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    r.left,
                    r.top,
                    r.right - r.left,
                    r.bottom - r.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
                let _ = app.refresh_settings_font(hwnd);
            }
            WM_THEMECHANGED | WM_SETTINGCHANGE => {
                super::theme::caption(hwnd, app.colors());
                let _ = app.refresh_settings_font(hwnd);
                return DefWindowProcW(hwnd, msg, wp, lp);
            }
            _ => return DefWindowProcW(hwnd, msg, wp, lp),
        }
        LRESULT(0)
    }
}
