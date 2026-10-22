//! Measured, scrollable settings geometry. The viewport always stays in the monitor work area.
use super::{
    commands::*,
    settings::{PERCENT, SLIDER, STATUS, VERSION},
    ui::px,
    App,
};
use std::mem::size_of;
use windows::Win32::{
    Foundation::{HWND, RECT, WPARAM},
    Graphics::Gdi::*,
    UI::{Controls::*, Input::KeyboardAndMouse::GetFocus, WindowsAndMessaging::*},
};
pub(super) fn control_text(hwnd: HWND) -> String {
    // SAFETY: only owned UI controls are queried; buffers include the terminator.
    unsafe {
        let mut text = vec![0; GetWindowTextLengthW(hwnd).max(0) as usize + 1];
        let read = GetWindowTextW(hwnd, &mut text);
        String::from_utf16_lossy(&text[..read as usize])
    }
}
pub(super) fn fit_to_work_area(rect: &mut RECT) {
    // SAFETY: the monitor and rectangle are synchronous scalar layout inputs.
    unsafe {
        let monitor = MonitorFromRect(rect, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(monitor, &mut info).as_bool() {
            let area = info.rcWork;
            let width = (rect.right - rect.left).clamp(1, (area.right - area.left).max(1));
            let height = (rect.bottom - rect.top).clamp(1, (area.bottom - area.top).max(1));
            rect.left = rect.left.clamp(area.left, area.right - width);
            rect.top = rect.top.clamp(area.top, area.bottom - height);
            rect.right = rect.left + width;
            rect.bottom = rect.top + height;
        }
    }
}
impl App {
    pub(super) fn layout_settings(&self, hwnd: HWND) {
        let font = self.settings_font.borrow().as_ref().map(|font| font.0);
        let Some(font) = font else {
            return;
        };
        let heading = self
            .settings_heading_font
            .borrow()
            .as_ref()
            .map(|font| font.0)
            .unwrap_or(font);
        // SAFETY: measuring uses the installed font; GDI selections/DCs are restored before scrolling.
        unsafe {
            let mut client = RECT::default();
            let _ = GetClientRect(hwnd, &mut client);
            let margin = px(hwnd, if client.right < px(hwnd, 400) { 16 } else { 24 });
            let gap = px(hwnd, 8);
            let inner = (client.right - 2 * margin).max(px(hwnd, 48));
            let dc = GetDC(Some(hwnd));
            let old = SelectObject(dc, font.into());
            let mut metrics = TEXTMETRICW::default();
            let _ = GetTextMetricsW(dc, &mut metrics);
            let line = metrics.tmHeight.max(px(hwnd, 16));
            let measure = |id: usize, width: i32| {
                let mut text: Vec<u16> = GetDlgItem(Some(hwnd), id as i32)
                    .map(control_text)
                    .unwrap_or_default()
                    .encode_utf16()
                    .collect();
                let mut rect = RECT {
                    right: width.max(1),
                    ..Default::default()
                };
                DrawTextW(
                    dc,
                    &mut text,
                    &mut rect,
                    DT_CALCRECT | DT_WORDBREAK | DT_NOPREFIX,
                );
                (rect.bottom - rect.top).max(line)
            };
            let mut positions = Vec::new();
            let mut y = margin;
            for (group, items) in [
                (56, &[THEME_DARK, THEME_LIGHT, THEME_SYSTEM][..]),
                (50, &[51, SLIDER, MUTE, COMPAT, 52][..]),
                (53, &[BLOCKER, STATUS, 54][..]),
                (55, &[VERSION, UPDATE][..]),
            ] {
                SelectObject(dc, heading.into());
                let header_height = measure(group, inner);
                SelectObject(dc, font.into());
                positions.push((
                    group,
                    RECT {
                        left: margin,
                        top: y,
                        right: client.right - margin,
                        bottom: y + header_height,
                    },
                ));
                y += header_height + px(hwnd, 12);
                if group == 56 {
                    let min_width = items
                        .iter()
                        .map(|id| {
                            let label: Vec<u16> = GetDlgItem(Some(hwnd), *id as i32)
                                .map(control_text)
                                .unwrap_or_default()
                                .encode_utf16()
                                .collect();
                            let mut size = windows::Win32::Foundation::SIZE::default();
                            let _ = GetTextExtentPoint32W(dc, &label, &mut size);
                            size.cx + px(hwnd, 40)
                        })
                        .max()
                        .unwrap_or(0);
                    let columns = if inner >= 3 * min_width + 2 * gap {
                        3
                    } else {
                        1
                    };
                    let width = (inner - (columns - 1) * gap) / columns;
                    let height = (line + gap).max(px(hwnd, 32));
                    for (index, &id) in items.iter().enumerate() {
                        let column = index as i32 % columns;
                        let row = index as i32 / columns;
                        let left = margin + column * (width + gap);
                        let top = y + row * (height + gap);
                        positions.push((
                            id,
                            RECT {
                                left,
                                top,
                                right: left + width,
                                bottom: top + height,
                            },
                        ));
                    }
                    y += ((items.len() as i32 + columns - 1) / columns) * (height + gap)
                        + px(hwnd, 16);
                    continue;
                }
                for &id in items {
                    let x = margin;
                    let (width, height) = if id == SLIDER {
                        (inner - px(hwnd, 64) - gap, px(hwnd, 32).max(line + gap))
                    } else if matches!(id, MUTE | COMPAT | BLOCKER) {
                        (inner, measure(id, inner - px(hwnd, 26)) + gap)
                    } else if id == UPDATE {
                        (
                            inner,
                            (measure(id, inner - px(hwnd, 24)) + px(hwnd, 16)).max(px(hwnd, 36)),
                        )
                    } else {
                        (inner, measure(id, inner))
                    };
                    positions.push((
                        id,
                        RECT {
                            left: x,
                            top: y,
                            right: x + width,
                            bottom: y + height,
                        },
                    ));
                    if id == SLIDER {
                        positions.push((
                            PERCENT,
                            RECT {
                                left: x + width + gap,
                                top: y + (height - line) / 2,
                                right: x + inner,
                                bottom: y + (height + line) / 2,
                            },
                        ));
                    }
                    y += height + gap;
                }
                y += px(hwnd, 16);
            }
            SelectObject(dc, old);
            ReleaseDC(Some(hwnd), dc);
            self.settings_content.set(y);
            let info = SCROLLINFO {
                cbSize: size_of::<SCROLLINFO>() as u32,
                fMask: SIF_RANGE | SIF_PAGE | SIF_POS,
                nMin: 0,
                nMax: y.saturating_sub(1),
                nPage: client.bottom.max(0) as u32,
                nPos: self
                    .settings_scroll
                    .get()
                    .clamp(0, (y - client.bottom).max(0)),
                ..Default::default()
            };
            SetScrollInfo(hwnd, SB_VERT, &info, true);
            let mut after = RECT::default();
            let _ = GetClientRect(hwnd, &mut after);
            if after.right != client.right {
                self.layout_settings(hwnd);
                return;
            }
            *self.settings_positions.borrow_mut() = positions;
            self.scroll_settings(info.nPos);
        }
    }
    pub(super) fn scroll_settings(&self, position: i32) {
        let hwnd = self.settings.get();
        if hwnd.is_invalid() {
            return;
        }
        // SAFETY: all positions refer to owned children; no RefCell borrow spans window messages.
        unsafe {
            let mut client = RECT::default();
            let _ = GetClientRect(hwnd, &mut client);
            let position = position.clamp(0, (self.settings_content.get() - client.bottom).max(0));
            self.settings_scroll.set(position);
            SetScrollInfo(
                hwnd,
                SB_VERT,
                &SCROLLINFO {
                    cbSize: size_of::<SCROLLINFO>() as u32,
                    fMask: SIF_POS,
                    nPos: position,
                    ..Default::default()
                },
                true,
            );
            let positions = self.settings_positions.borrow().clone();
            for (id, r) in positions {
                if let Ok(child) = GetDlgItem(Some(hwnd), id as i32) {
                    let _ = SetWindowPos(
                        child,
                        None,
                        r.left,
                        r.top - position,
                        r.right - r.left,
                        r.bottom - r.top,
                        SWP_NOZORDER | SWP_NOACTIVATE,
                    );
                }
            }
            let _ = InvalidateRect(Some(hwnd), None, true);
        }
    }
    pub(super) fn settings_vscroll(&self, wp: WPARAM) {
        let hwnd = self.settings.get();
        // SAFETY: reads the native scrollbar's full-width tracking position.
        unsafe {
            let mut info = SCROLLINFO {
                cbSize: size_of::<SCROLLINFO>() as u32,
                fMask: SIF_ALL,
                ..Default::default()
            };
            let _ = GetScrollInfo(hwnd, SB_VERT, &mut info);
            let pos = match wp.0 as u16 as i32 {
                code if code == SB_LINEUP.0 => info.nPos - px(hwnd, 24),
                code if code == SB_LINEDOWN.0 => info.nPos + px(hwnd, 24),
                code if code == SB_PAGEUP.0 => info.nPos - info.nPage as i32,
                code if code == SB_PAGEDOWN.0 => info.nPos + info.nPage as i32,
                code if code == SB_TOP.0 => 0,
                code if code == SB_BOTTOM.0 => info.nMax,
                code if code == SB_THUMBTRACK.0 || code == SB_THUMBPOSITION.0 => info.nTrackPos,
                _ => return,
            };
            self.scroll_settings(pos);
        }
    }
    pub(super) fn ensure_settings_focus(&self) {
        let hwnd = self.settings.get();
        if hwnd.is_invalid() {
            return;
        }
        // SAFETY: native focus and rectangles belong to the current UI thread.
        unsafe {
            let focus = GetFocus();
            let id = GetDlgCtrlID(focus) as usize;
            let bounds = self
                .settings_positions
                .borrow()
                .iter()
                .find(|(key, _)| *key == id)
                .map(|(_, r)| *r);
            if let Some(r) = bounds {
                let mut client = RECT::default();
                let _ = GetClientRect(hwnd, &mut client);
                let position = self.settings_scroll.get();
                if r.top < position {
                    self.scroll_settings(r.top);
                } else if r.bottom > position + client.bottom {
                    self.scroll_settings(r.bottom - client.bottom);
                }
            }
        }
    }
}
