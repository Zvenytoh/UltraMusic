//! System-themed native toolbar, keyboard access and provider menu.
use super::{commands::*, ui, ui::px, App};
use std::mem::size_of;
use windows::{
    core::{w, Error, Result, HSTRING, PWSTR},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM},
        Graphics::Gdi::*,
        UI::{Controls::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
};
pub(super) const PICKER: usize = 20;
pub(super) const HEIGHT: i32 = 44;
const FLEX: usize = 30;
const GAP: usize = 31;
// TB_SETBKCOLOR is the toolbar alias for this common-controls message.
const TB_SETBKCOLOR: u32 = CCM_SETBKCOLOR;
fn send(hwnd: HWND, msg: u32, wp: usize, lp: isize) -> LRESULT {
    // SAFETY: callers supply scalar messages or synchronous borrowed structures.
    unsafe { SendMessageW(hwnd, msg, Some(WPARAM(wp)), Some(LPARAM(lp))) }
}
impl App {
    pub(super) fn create_chrome(&self) -> Result<()> {
        // SAFETY: children and native controls belong to this UI thread.
        unsafe {
            InitCommonControlsEx(&INITCOMMONCONTROLSEX {
                dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
                dwICC: ICC_BAR_CLASSES,
            })
            .ok()?;
            let toolbar = CreateWindowExW(
                Default::default(),
                TOOLBARCLASSNAMEW,
                w!("Navigation UltraMusic"),
                WS_CHILD
                    | WS_VISIBLE
                    | WS_TABSTOP
                    | WINDOW_STYLE(
                        TBSTYLE_FLAT
                            | TBSTYLE_LIST
                            | TBSTYLE_TOOLTIPS
                            | CCS_NORESIZE as u32
                            | CCS_NOPARENTALIGN as u32
                            | CCS_NODIVIDER as u32,
                    ),
                0,
                0,
                0,
                0,
                Some(self.hwnd),
                None,
                None,
                None,
            )?;
            self.toolbar.set(toolbar);
            send(
                toolbar,
                TB_SETBKCOLOR,
                0,
                GetSysColor(COLOR_WINDOW) as isize,
            );
            send(toolbar, TB_BUTTONSTRUCTSIZE, size_of::<TBBUTTON>(), 0);
            send(
                toolbar,
                TB_SETEXTENDEDSTYLE,
                0,
                (TBSTYLE_EX_MIXEDBUTTONS | TBSTYLE_EX_DRAWDDARROWS | TBSTYLE_EX_DOUBLEBUFFER)
                    as isize,
            );
            let mut buttons = Vec::new();
            for (id, image, style, label) in [
                (BACK, 0, BTNS_BUTTON, "Retour"),
                (GAP, 8, BTNS_SEP, ""),
                (
                    PICKER,
                    I_IMAGENONE,
                    BTNS_WHOLEDROPDOWN | BTNS_SHOWTEXT,
                    "Choisir le service",
                ),
                (FLEX, 8, BTNS_SEP, ""),
                (SETTINGS, 1, BTNS_BUTTON, "Paramètres"),
            ] {
                let label: Vec<u16> = label.encode_utf16().chain([0, 0]).collect();
                let index = send(toolbar, TB_ADDSTRINGW, 0, label.as_ptr() as isize).0;
                buttons.push(TBBUTTON {
                    iBitmap: image,
                    idCommand: id as i32,
                    fsState: if id == BACK { 0 } else { TBSTATE_ENABLED as u8 },
                    fsStyle: style as u8,
                    iString: index,
                    ..Default::default()
                });
            }
            if send(
                toolbar,
                TB_ADDBUTTONSW,
                buttons.len(),
                buttons.as_ptr() as isize,
            )
            .0 == 0
            {
                return Err(Error::from_thread());
            }
        }
        self.refresh_chrome_resources()?;
        self.refresh_ui();
        Ok(())
    }
    pub(super) fn refresh_chrome_resources(&self) -> Result<()> {
        let resources = ui::Resources::new(self.hwnd)?;
        let toolbar = self.toolbar.get();
        // SAFETY: this owned toolbar receives a scalar system-color message on the UI thread.
        unsafe {
            send(
                toolbar,
                TB_SETBKCOLOR,
                0,
                GetSysColor(COLOR_WINDOW) as isize,
            );
        }
        ui::apply_font(toolbar, resources.font.0);
        send(toolbar, TB_SETIMAGELIST, 0, resources.images.0);
        let image_size = px(self.hwnd, 16) as u32;
        send(
            toolbar,
            TB_SETBITMAPSIZE,
            0,
            ((image_size << 16) | image_size) as isize,
        );
        let size = px(self.hwnd, 32) as u32;
        send(toolbar, TB_SETBUTTONSIZE, 0, ((size << 16) | size) as isize);
        let old = self.ui.replace(Some(resources));
        drop(old);
        self.layout_chrome();
        Ok(())
    }
    fn toolbar_width(&self, id: usize, width: i32) {
        let info = TBBUTTONINFOW {
            cbSize: size_of::<TBBUTTONINFOW>() as u32,
            dwMask: TBIF_SIZE,
            cx: width.max(0).min(u16::MAX as i32) as u16,
            ..Default::default()
        };
        send(
            self.toolbar.get(),
            TB_SETBUTTONINFOW,
            id,
            &info as *const _ as isize,
        );
    }
    pub(super) fn layout_chrome(&self) {
        let toolbar = self.toolbar.get();
        if toolbar.is_invalid() {
            return;
        }
        // SAFETY: GDI selection and DC borrowing are balanced on the UI thread.
        unsafe {
            let mut bounds = RECT::default();
            let _ = GetClientRect(self.hwnd, &mut bounds);
            let font = self.ui.borrow().as_ref().map(|r| r.font.0);
            let mut extent = SIZE::default();
            if let Some(font) = font {
                let dc = GetDC(Some(toolbar));
                let old = SelectObject(dc, font.into());
                let label: Vec<u16> = self.service.get().label().encode_utf16().collect();
                let _ = GetTextExtentPoint32W(dc, &label, &mut extent);
                SelectObject(dc, old);
                ReleaseDC(Some(toolbar), dc);
            }
            let provider = (extent.cx + px(self.hwnd, 40))
                .min((bounds.right - px(self.hwnd, 96)).max(px(self.hwnd, 64)));
            self.toolbar_width(BACK, px(self.hwnd, 32));
            self.toolbar_width(SETTINGS, px(self.hwnd, 32));
            self.toolbar_width(PICKER, provider);
            self.toolbar_width(GAP, px(self.hwnd, 8));
            self.toolbar_width(
                FLEX,
                (bounds.right - px(self.hwnd, 88) - provider).max(px(self.hwnd, 8)),
            );
            send(toolbar, TB_AUTOSIZE, 0, 0);
            let _ = SetWindowPos(
                toolbar,
                None,
                px(self.hwnd, 8),
                px(self.hwnd, 6),
                (bounds.right - px(self.hwnd, 16)).max(0),
                px(self.hwnd, 32),
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            let _ = InvalidateRect(Some(self.hwnd), None, false);
        }
    }
    pub(super) fn refresh_ui(&self) {
        let mut label: Vec<u16> = self
            .service
            .get()
            .label()
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let info = TBBUTTONINFOW {
            cbSize: size_of::<TBBUTTONINFOW>() as u32,
            dwMask: TBIF_TEXT,
            pszText: PWSTR(label.as_mut_ptr()),
            ..Default::default()
        };
        if !self.toolbar.get().is_invalid() {
            send(
                self.toolbar.get(),
                TB_ENABLEBUTTON,
                PICKER,
                isize::from(self.update_phase.get() != super::updates::Phase::Preparing),
            );
            send(
                self.toolbar.get(),
                TB_SETBUTTONINFOW,
                PICKER,
                &info as *const _ as isize,
            );
            self.layout_chrome();
        }
        self.refresh_settings();
    }
    pub(super) fn refresh_navigation(&self) {
        let mut enabled = windows::core::BOOL(0);
        if let Some(controller) = self.controller() {
            // SAFETY: the current controller and toolbar are confined to the UI STA.
            unsafe {
                if let Ok(web) = controller.CoreWebView2() {
                    let _ = web.CanGoBack(&mut enabled);
                }
            }
        }
        send(
            self.toolbar.get(),
            TB_ENABLEBUTTON,
            BACK,
            isize::from(enabled.as_bool()),
        );
    }
    pub(super) fn header_action(&self, action: isize) {
        match action {
            1 => {
                // SAFETY: toolbar is an owned child window.
                unsafe {
                    let _ = SetFocus(Some(self.toolbar.get()));
                }
                send(self.toolbar.get(), TB_SETHOTITEM, 2, 0);
            }
            2 => self.open_settings(),
            _ => {}
        }
    }
    pub(super) fn keyboard(&self, message: &MSG) -> bool {
        // SAFETY: inspecting current-thread focus; dialog navigation is scoped to native controls.
        unsafe {
            let root = GetAncestor(message.hwnd, GA_ROOT);
            if !self.settings.get().is_invalid() && root == self.settings.get() {
                if message.message == WM_KEYDOWN && message.wParam.0 == VK_ESCAPE.0 as usize {
                    let _ = DestroyWindow(root);
                    return true;
                }
                let handled = IsDialogMessageW(root, message).as_bool();
                if handled && message.message == WM_KEYDOWN {
                    self.ensure_settings_focus();
                }
                return handled;
            }
            if root != self.hwnd {
                return false;
            }
            let in_header = GetFocus() == self.toolbar.get();
            if message.message == WM_KEYDOWN {
                if message.wParam.0 == VK_F6.0 as usize {
                    if in_header {
                        self.focus_player();
                    } else {
                        self.header_action(1);
                    }
                    return true;
                }
                if in_header
                    && (message.wParam.0 == VK_ESCAPE.0 as usize
                        || message.wParam.0 == VK_TAB.0 as usize)
                {
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
            false
        }
    }
    fn focus_player(&self) {
        if let Some(controller) = self.controller() {
            // SAFETY: current STA controller owns focus transfer to the browser.
            unsafe {
                let _=controller.MoveFocus(webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_MOVE_FOCUS_REASON_PROGRAMMATIC);
            }
        }
    }
    pub(super) fn service_menu(&self) {
        // SAFETY: native menu strings are copied; the menu is destroyed after its modal loop.
        unsafe {
            let Ok(menu) = CreatePopupMenu() else { return };
            for (id, service) in SERVICES {
                let _ = AppendMenuW(
                    menu,
                    MF_STRING
                        | if self.service.get() == service {
                            MF_CHECKED
                        } else {
                            MF_UNCHECKED
                        },
                    id,
                    &HSTRING::from(service.label()),
                );
            }
            let mut bounds = RECT::default();
            send(
                self.toolbar.get(),
                TB_GETRECT,
                PICKER,
                &mut bounds as *mut _ as isize,
            );
            let mut point = POINT {
                x: bounds.left,
                y: bounds.bottom,
            };
            let _ = ClientToScreen(self.toolbar.get(), &mut point);
            let _ = SetForegroundWindow(self.hwnd);
            let command = TrackPopupMenu(
                menu,
                TPM_RETURNCMD | TPM_LEFTALIGN,
                point.x,
                point.y,
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
        // SAFETY: message-specific pointers are borrowed only for this dispatch.
        unsafe {
            match msg {
                WM_NOTIFY => {
                    let header = &*(lp.0 as *const NMHDR);
                    if header.hwndFrom == self.toolbar.get() && header.code == TBN_DROPDOWN {
                        self.service_menu();
                        return Some(LRESULT(0));
                    }
                    if header.hwndFrom == self.toolbar.get() && header.code == NM_CUSTOMDRAW {
                        let draw = &*(lp.0 as *const NMTBCUSTOMDRAW);
                        if draw.nmcd.dwDrawStage == CDDS_PREPAINT {
                            return Some(LRESULT(CDRF_NOTIFYITEMDRAW as isize));
                        }
                        if draw.nmcd.dwDrawStage == CDDS_ITEMPREPAINT
                            && matches!(draw.nmcd.dwItemSpec, FLEX | GAP)
                        {
                            // These native separator items provide spacing, not visible rules.
                            return Some(LRESULT(CDRF_SKIPDEFAULT as isize));
                        }
                    }
                    None
                }
                WM_GETMINMAXINFO => {
                    let info = &mut *(lp.0 as *mut MINMAXINFO);
                    info.ptMinTrackSize = POINT {
                        x: px(self.hwnd, 360),
                        y: px(self.hwnd, 300),
                    };
                    Some(LRESULT(0))
                }
                WM_PAINT => {
                    let mut paint = PAINTSTRUCT::default();
                    let dc = BeginPaint(self.hwnd, &mut paint);
                    let mut r = RECT::default();
                    let _ = GetClientRect(self.hwnd, &mut r);
                    r.bottom = px(self.hwnd, HEIGHT).min(r.bottom);
                    FillRect(dc, &r, GetSysColorBrush(COLOR_WINDOW));
                    let _ = EndPaint(self.hwnd, &paint);
                    Some(LRESULT(0))
                }
                WM_ERASEBKGND => Some(LRESULT(1)),
                _ => None,
            }
        }
    }
}
