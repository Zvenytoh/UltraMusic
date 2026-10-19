//! Event-driven GDI drawing. Every selected object is restored before deletion.
use windows::{
    core::w,
    Win32::{
        Foundation::{COLORREF, HWND, POINT, RECT},
        Graphics::Gdi::*,
        UI::HiDpi::GetDpiForWindow,
    },
};

pub(super) const BLACK: COLORREF = COLORREF(0x000000);
pub(super) const PANEL: COLORREF = COLORREF(0x181818);
pub(super) const HOVER: COLORREF = COLORREF(0x303030);
pub(super) const LINE: COLORREF = COLORREF(0x393939);
pub(super) const WHITE: COLORREF = COLORREF(0xF5F5F5);
pub(super) const MUTED: COLORREF = COLORREF(0xAAAAAA);
pub(super) const RED: COLORREF = COLORREF(0x3626E3);

pub(super) fn px(hwnd: HWND, value: i32) -> i32 {
    // SAFETY: caller supplies a live UI window; 96 is a safe fallback on failure.
    let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96);
    ((i64::from(value) * i64::from(dpi) + 48) / 96) as i32
}

pub(super) fn rect(x: i32, y: i32, width: i32, height: i32) -> RECT {
    RECT {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    }
}

struct Selection {
    dc: HDC,
    old: HGDIOBJ,
    object: HGDIOBJ,
}
impl Selection {
    fn new(dc: HDC, object: HGDIOBJ) -> Self {
        // SAFETY: the object lives until this selection is dropped on this thread.
        Self {
            dc,
            old: unsafe { SelectObject(dc, object) },
            object,
        }
    }
}
impl Drop for Selection {
    fn drop(&mut self) {
        // SAFETY: restore the prior object before releasing the owned GDI object.
        unsafe {
            SelectObject(self.dc, self.old);
            let _ = DeleteObject(self.object);
        }
    }
}

pub(super) fn fill(dc: HDC, rect: &RECT, color: COLORREF) {
    // SAFETY: paint callbacks own dc; FillRect copies the brush synchronously.
    unsafe {
        let brush = CreateSolidBrush(color);
        FillRect(dc, rect, brush);
        let _ = DeleteObject(brush.into());
    }
}

pub(super) fn round(dc: HDC, rect: &RECT, color: COLORREF, border: COLORREF, radius: i32) {
    if rect.right <= rect.left || rect.bottom <= rect.top {
        return;
    }
    // SAFETY: selections are scoped to the paint call and released in reverse order.
    unsafe {
        let _brush = Selection::new(dc, CreateSolidBrush(color).into());
        let _pen = Selection::new(dc, CreatePen(PS_SOLID, 1, border).into());
        let _ = RoundRect(
            dc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            radius,
            radius,
        );
    }
}

pub(super) fn text(dc: HDC, hwnd: HWND, value: &str, mut rect: RECT, size: i32, color: COLORREF) {
    let mut utf16: Vec<u16> = value.encode_utf16().collect();
    // SAFETY: the UTF-16 buffer and selected font remain valid through DrawTextW.
    unsafe {
        let font = CreateFontW(
            -px(hwnd, size),
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH.0 as u32,
            w!("Segoe UI"),
        );
        let _font = Selection::new(dc, font.into());
        let old_mode = SetBkMode(dc, TRANSPARENT);
        let old_color = SetTextColor(dc, color);
        DrawTextW(
            dc,
            &mut utf16,
            &mut rect,
            DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
        );
        SetTextColor(dc, old_color);
        SetBkMode(dc, BACKGROUND_MODE(old_mode as u32));
    }
}

#[derive(Clone, Copy)]
pub(super) enum Icon {
    Music,
    Video,
    Gear,
    Chevron,
    Check,
    Minus,
    Plus,
}

pub(super) fn icon(
    dc: HDC,
    hwnd: HWND,
    icon: Icon,
    x: i32,
    y: i32,
    color: COLORREF,
    background: COLORREF,
) {
    let point = |a, b| POINT {
        x: x + px(hwnd, a),
        y: y + px(hwnd, b),
    };
    // SAFETY: all coordinates and GDI objects are local to a valid paint context.
    unsafe {
        let _pen = Selection::new(dc, CreatePen(PS_SOLID, px(hwnd, 2), color).into());
        let _brush = Selection::new(dc, CreateSolidBrush(background).into());
        match icon {
            Icon::Music => {
                let _ = Polyline(
                    dc,
                    &[point(8, 16), point(8, 5), point(18, 3), point(18, 14)],
                );
                let _ = Ellipse(
                    dc,
                    x + px(hwnd, 2),
                    y + px(hwnd, 14),
                    x + px(hwnd, 8),
                    y + px(hwnd, 19),
                );
                let _ = Ellipse(
                    dc,
                    x + px(hwnd, 12),
                    y + px(hwnd, 12),
                    x + px(hwnd, 18),
                    y + px(hwnd, 17),
                );
            }
            Icon::Video => {
                let _ = RoundRect(
                    dc,
                    x + px(hwnd, 1),
                    y + px(hwnd, 4),
                    x + px(hwnd, 21),
                    y + px(hwnd, 18),
                    px(hwnd, 6),
                    px(hwnd, 6),
                );
                let _ = Polyline(dc, &[point(9, 8), point(14, 11), point(9, 14), point(9, 8)]);
            }
            Icon::Gear => {
                let points: Vec<POINT> = (0..32)
                    .map(|i| {
                        let angle = f64::from(i) * std::f64::consts::TAU / 32.0;
                        let radius = if i % 4 == 0 || i % 4 == 3 { 10.0 } else { 8.0 };
                        POINT {
                            x: x + px(hwnd, 11)
                                + (angle.cos() * f64::from(px(hwnd, radius as i32))) as i32,
                            y: y + px(hwnd, 11)
                                + (angle.sin() * f64::from(px(hwnd, radius as i32))) as i32,
                        }
                    })
                    .collect();
                let _ = Polygon(dc, &points);
                let _ = Ellipse(
                    dc,
                    x + px(hwnd, 7),
                    y + px(hwnd, 7),
                    x + px(hwnd, 15),
                    y + px(hwnd, 15),
                );
            }
            Icon::Chevron => {
                let _ = Polyline(dc, &[point(6, 9), point(11, 14), point(16, 9)]);
            }
            Icon::Check => {
                let _ = Polyline(dc, &[point(4, 11), point(9, 16), point(18, 6)]);
            }
            Icon::Minus | Icon::Plus => {
                let _ = Polyline(dc, &[point(4, 11), point(18, 11)]);
                if matches!(icon, Icon::Plus) {
                    let _ = Polyline(dc, &[point(11, 4), point(11, 18)]);
                }
            }
        }
    }
}
