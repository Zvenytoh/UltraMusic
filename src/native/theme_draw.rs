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
                if draw.dwDrawStage != CDDS_PREPAINT {
                    return None;
                }
                let mut bounds = RECT::default();
                let _ = GetClientRect(header.hwndFrom, &mut bounds);
                fill(draw.hdc, &bounds, colors.background);
                let mut track = RECT::default();
                let mut thumb = RECT::default();
                SendMessageW(
                    header.hwndFrom,
                    TBM_GETCHANNELRECT,
                    None,
                    Some(windows::Win32::Foundation::LPARAM(
                        (&mut track as *mut RECT) as isize,
                    )),
                );
                SendMessageW(
                    header.hwndFrom,
                    TBM_GETTHUMBRECT,
                    None,
                    Some(windows::Win32::Foundation::LPARAM(
                        (&mut thumb as *mut RECT) as isize,
                    )),
                );
                let middle = (thumb.top + thumb.bottom) / 2;
                track.top = middle - px(header.hwndFrom, 1);
                track.bottom = middle + px(header.hwndFrom, 1);
                rounded(
                    draw.hdc,
                    track,
                    colors.border,
                    colors.border,
                    px(header.hwndFrom, 3),
                );
                let mut filled = track;
                filled.right =
                    ((thumb.left + thumb.right) / 2).clamp(track.left, track.right.max(track.left));
                if filled.right > filled.left {
                    rounded(
                        draw.hdc,
                        filled,
                        colors.secondary,
                        colors.secondary,
                        px(header.hwndFrom, 3),
                    );
                }
                rounded(
                    draw.hdc,
                    thumb,
                    colors.text,
                    colors.text,
                    px(header.hwndFrom, 8),
                );
                if windows::Win32::UI::Input::KeyboardAndMouse::GetFocus() == header.hwndFrom {
                    let _ = DrawFocusRect(draw.hdc, &bounds);
                }
                return Some(LRESULT(CDRF_SKIPDEFAULT as isize));
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
            let radio = matches!(id, THEME_DARK | THEME_LIGHT | THEME_SYSTEM);
            if radio {
                rounded(
                    draw.hdc,
                    bounds,
                    if checked || hot {
                        colors.hover
                    } else {
                        colors.background
                    },
                    if checked {
                        colors.border
                    } else {
                        colors.background
                    },
                    px(hwnd, 10),
                );
                text_bounds.left += px(hwnd, 8);
                text_bounds.right -= px(hwnd, 8);
            } else if id == UPDATE {
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
                    px(hwnd, 4),
                );
                if checked {
                    let mut center = marker;
                    SetBkMode(draw.hdc, TRANSPARENT);
                    SetTextColor(draw.hdc, colors.background);
                    DrawTextW(
                        draw.hdc,
                        &mut ['✓' as u16],
                        &mut center,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                    );
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
                DT_WORDBREAK
                    | DT_NOPREFIX
                    | if id == UPDATE || radio {
                        DT_CENTER
                    } else {
                        DT_LEFT
                    },
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
            super::theme::opaque(draw.nmcd.hdc, &draw.nmcd.rc, |dc| {
                fill(dc, &draw.nmcd.rc, colors.background)
            });
            return LRESULT(CDRF_NOTIFYITEMDRAW as isize);
        }
        if draw.nmcd.dwDrawStage != CDDS_ITEMPREPAINT {
            return LRESULT(CDRF_DODEFAULT as isize);
        }
        let mut item = *draw;
        super::theme::opaque(draw.nmcd.hdc, &draw.nmcd.rc, |dc| {
            item.nmcd.hdc = dc;
            let _ = self.paint_toolbar_item(&item);
        });
        LRESULT(CDRF_SKIPDEFAULT as isize)
    }
    fn paint_toolbar_item(&self, draw: &NMTBCUSTOMDRAW) -> LRESULT {
        let colors = self.colors();
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
                    r.left + (r.right - r.left - side) / 2,
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
            if draw.nmcd.uItemState & CDIS_FOCUS != NMCUSTOMDRAW_DRAW_STATE_FLAGS(0) {
                let _ = DrawFocusRect(dc, &r);
            }
        }
        LRESULT(CDRF_SKIPDEFAULT as isize)
    }
}
