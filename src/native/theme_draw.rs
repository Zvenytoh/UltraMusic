//! Custom paint on native input controls; their focus, keyboard and accessibility remain native.
use super::{
    commands::*,
    settings::SLIDER,
    theme::{fill, rounded},
    ui::px,
    App,
};
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, RECT},
    Graphics::Gdi::*,
    UI::{Controls::*, WindowsAndMessaging::*},
};

impl App {
    pub(super) fn draw_settings_control(&self, lp: LPARAM) -> Option<LRESULT> {
        let colors = self.colors();
        if colors.high_contrast {
            return None;
        }
        // SAFETY: WM_NOTIFY carries a message-scoped NMHDR; custom-draw payloads share its prefix.
        unsafe {
            let header = &*(lp.0 as *const NMHDR);
            if header.code != NM_CUSTOMDRAW {
                return None;
            }
            let id = header.idFrom;
            let draw = &*(lp.0 as *const NMCUSTOMDRAW);
            if id == SLIDER {
                if draw.dwDrawStage == CDDS_PREPAINT {
                    let mut bounds = RECT::default();
                    let _ = GetClientRect(header.hwndFrom, &mut bounds);
                    fill(draw.hdc, &bounds, colors.background);
                    return Some(LRESULT(CDRF_NOTIFYITEMDRAW as isize));
                }
                if draw.dwDrawStage == CDDS_ITEMPREPAINT {
                    let mut r = draw.rc;
                    if draw.dwItemSpec == TBCD_CHANNEL as usize {
                        let middle = (r.top + r.bottom) / 2;
                        r.top = middle - px(header.hwndFrom, 1);
                        r.bottom = middle + px(header.hwndFrom, 1);
                        rounded(
                            draw.hdc,
                            r,
                            colors.border,
                            colors.border,
                            px(header.hwndFrom, 3),
                        );
                        return Some(LRESULT(CDRF_SKIPDEFAULT as isize));
                    }
                    if draw.dwItemSpec == TBCD_THUMB as usize {
                        rounded(
                            draw.hdc,
                            r,
                            colors.text,
                            colors.text,
                            px(header.hwndFrom, 8),
                        );
                        return Some(LRESULT(CDRF_SKIPDEFAULT as isize));
                    }
                }
                return None;
            }
            if !matches!(
                id,
                MUTE | COMPAT | BLOCKER | UPDATE | THEME_DARK | THEME_LIGHT | THEME_SYSTEM
            ) || draw.dwDrawStage != CDDS_PREPAINT
            {
                return None;
            }
            let hwnd = header.hwndFrom;
            let mut bounds = RECT::default();
            let _ = GetClientRect(hwnd, &mut bounds);
            fill(draw.hdc, &bounds, colors.background);
            let enabled =
                windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled(hwnd).as_bool();
            let checked = SendMessageW(hwnd, BM_GETCHECK, None, None).0 == BST_CHECKED.0 as isize;
            let hot =
                draw.uItemState & (CDIS_HOT | CDIS_SELECTED) != NMCUSTOMDRAW_DRAW_STATE_FLAGS(0);
            let foreground = if enabled {
                colors.text
            } else {
                colors.secondary
            };
            let font = SendMessageW(hwnd, WM_GETFONT, None, None);
            let old = SelectObject(draw.hdc, HGDIOBJ(font.0 as *mut _));
            let mut text_bounds = bounds;
            if id == UPDATE {
                rounded(
                    draw.hdc,
                    bounds,
                    if hot { colors.hover } else { colors.background },
                    colors.border,
                    px(hwnd, 10),
                );
                text_bounds.left += px(hwnd, 12);
                text_bounds.right -= px(hwnd, 12);
            } else {
                let side = px(hwnd, 16);
                let top = (bounds.bottom - side) / 2;
                let marker = RECT {
                    left: 1,
                    top,
                    right: 1 + side,
                    bottom: top + side,
                };
                let radio = matches!(id, THEME_DARK | THEME_LIGHT | THEME_SYSTEM);
                rounded(
                    draw.hdc,
                    marker,
                    if checked {
                        foreground
                    } else if hot {
                        colors.hover
                    } else {
                        colors.background
                    },
                    if checked { foreground } else { colors.border },
                    px(hwnd, if radio { 16 } else { 4 }),
                );
                if checked {
                    let mut center = marker;
                    if radio {
                        let inset = px(hwnd, 5);
                        center.left += inset;
                        center.right -= inset;
                        center.top += inset;
                        center.bottom -= inset;
                        rounded(
                            draw.hdc,
                            center,
                            colors.background,
                            colors.background,
                            px(hwnd, 8),
                        );
                    } else {
                        SetBkMode(draw.hdc, TRANSPARENT);
                        SetTextColor(draw.hdc, colors.background);
                        DrawTextW(
                            draw.hdc,
                            &mut ['✓' as u16],
                            &mut center,
                            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                        );
                    }
                }
                text_bounds.left += px(hwnd, 28);
            }
            SetBkMode(draw.hdc, TRANSPARENT);
            SetTextColor(draw.hdc, foreground);
            let mut label: Vec<u16> = super::settings_layout::control_text(hwnd)
                .encode_utf16()
                .collect();
            let mut measured = text_bounds;
            DrawTextW(
                draw.hdc,
                &mut label,
                &mut measured,
                DT_CALCRECT | DT_WORDBREAK | DT_NOPREFIX,
            );
            text_bounds.top +=
                ((text_bounds.bottom - text_bounds.top - (measured.bottom - measured.top)) / 2)
                    .max(0);
            DrawTextW(
                draw.hdc,
                &mut label,
                &mut text_bounds,
                DT_WORDBREAK | DT_NOPREFIX | if id == UPDATE { DT_CENTER } else { DT_LEFT },
            );
            SelectObject(draw.hdc, old);
            if draw.uItemState & CDIS_FOCUS != NMCUSTOMDRAW_DRAW_STATE_FLAGS(0) {
                let inset = px(hwnd, 2);
                bounds.left += inset;
                bounds.top += inset;
                bounds.right -= inset;
                bounds.bottom -= inset;
                let _ = DrawFocusRect(draw.hdc, &bounds);
            }
            Some(LRESULT(CDRF_SKIPDEFAULT as isize))
        }
    }
    pub(super) fn draw_toolbar(&self, draw: &NMTBCUSTOMDRAW) -> LRESULT {
        let colors = self.colors();
        if colors.high_contrast {
            return LRESULT(CDRF_DODEFAULT as isize);
        }
        if draw.nmcd.dwDrawStage == CDDS_PREPAINT {
            return LRESULT(CDRF_NOTIFYITEMDRAW as isize);
        }
        if draw.nmcd.dwDrawStage != CDDS_ITEMPREPAINT {
            return LRESULT(CDRF_DODEFAULT as isize);
        }
        let id = draw.nmcd.dwItemSpec;
        let index = match id {
            BACK => 0,
            FORWARD => 1,
            super::chrome::PICKER => 2,
            SETTINGS => 3,
            _ => return LRESULT(CDRF_DODEFAULT as isize),
        };
        let dc = draw.nmcd.hdc;
        let r = draw.nmcd.rc;
        let hot =
            draw.nmcd.uItemState & (CDIS_HOT | CDIS_SELECTED) != NMCUSTOMDRAW_DRAW_STATE_FLAGS(0);
        fill(dc, &r, colors.background);
        if hot {
            rounded(dc, r, colors.hover, colors.hover, px(self.hwnd, 8));
        }
        let images = self.ui.borrow().as_ref().map(|ui| ui.images);
        // SAFETY: the image list owner remains installed for this synchronous native paint.
        unsafe {
            if let Some(images) = images {
                let side = px(self.hwnd, 16);
                let disabled =
                    draw.nmcd.uItemState & CDIS_DISABLED != NMCUSTOMDRAW_DRAW_STATE_FLAGS(0);
                let _ = ImageList_DrawEx(
                    images,
                    index,
                    dc,
                    r.left + px(self.hwnd, 6),
                    r.top + (r.bottom - r.top - side) / 2,
                    side,
                    side,
                    colors.background,
                    colors.background,
                    if disabled {
                        ILD_BLEND50
                    } else {
                        ILD_TRANSPARENT
                    },
                );
            }
            if id == super::chrome::PICKER {
                let mut arrow = r;
                arrow.left = arrow.right - px(self.hwnd, 12);
                SetBkMode(dc, TRANSPARENT);
                SetTextColor(dc, colors.secondary);
                DrawTextW(
                    dc,
                    &mut ['▾' as u16],
                    &mut arrow,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );
            }
            if draw.nmcd.uItemState & CDIS_FOCUS != NMCUSTOMDRAW_DRAW_STATE_FLAGS(0) {
                let _ = DrawFocusRect(dc, &r);
            }
        }
        LRESULT(CDRF_SKIPDEFAULT as isize)
    }
}
