//! Persisted host appearance. System high contrast takes precedence over app colors.
use super::{dialog, App};
use std::{fs, io::Read, mem::size_of, path::PathBuf};
use windows::{
    core::{Error, Result},
    Win32::{
        Foundation::{COLORREF, HWND, RECT},
        Graphics::{Dwm::*, Gdi::*},
        System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_SINGLETHREADED},
        UI::{Accessibility::*, WindowsAndMessaging::*},
    },
    UI::ViewManagement::{UIColorType, UISettings},
};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Choice {
    Dark,
    Light,
    System,
}
fn path() -> Option<PathBuf> {
    Some(
        PathBuf::from(std::env::var_os("LOCALAPPDATA")?)
            .join("UltraMusic")
            .join("appearance.txt"),
    )
}
impl Choice {
    pub(super) fn load() -> Self {
        let mut text = String::new();
        let read = path()
            .and_then(|p| fs::File::open(p).ok())
            .and_then(|f| f.take(32).read_to_string(&mut text).ok());
        match (read, text.trim()) {
            (Some(_), "light") => Self::Light,
            (Some(_), "system") => Self::System,
            _ => Self::Dark,
        }
    }
    fn save(self) -> std::io::Result<()> {
        let path = path().ok_or_else(|| std::io::Error::other("LOCALAPPDATA indisponible"))?;
        let temporary = path.with_extension("tmp");
        fs::write(
            &temporary,
            match self {
                Self::Dark => "dark",
                Self::Light => "light",
                Self::System => "system",
            },
        )?;
        fs::rename(temporary, path)
    }
}
#[derive(Clone, Copy)]
pub(super) struct Colors {
    pub background: COLORREF,
    pub text: COLORREF,
    pub secondary: COLORREF,
    pub hover: COLORREF,
    pub border: COLORREF,
    pub dark: bool,
    pub high_contrast: bool,
}
pub(super) struct Palette {
    pub colors: Colors,
    pub brush: HBRUSH,
}
fn rgb(r: u32, g: u32, b: u32) -> COLORREF {
    COLORREF(r | (g << 8) | (b << 16))
}
impl Palette {
    pub(super) fn new(choice: Choice) -> Result<Self> {
        // SAFETY: correctly sized system-settings structure, with no retained pointer.
        unsafe {
            let mut contrast = HIGHCONTRASTW {
                cbSize: size_of::<HIGHCONTRASTW>() as u32,
                ..Default::default()
            };
            let _ = SystemParametersInfoW(
                SPI_GETHIGHCONTRAST,
                contrast.cbSize,
                Some((&mut contrast as *mut HIGHCONTRASTW).cast()),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            );
            let high_contrast = contrast.dwFlags & HCF_HIGHCONTRASTON != HIGHCONTRASTW_FLAGS(0);
            let dark = match choice {
                Choice::Dark => true,
                Choice::Light => false,
                Choice::System => {
                    // Match this temporary WinRT initialization on the existing UI STA.
                    if RoInitialize(RO_INIT_SINGLETHREADED).is_err() {
                        true
                    } else {
                        let dark = UISettings::new()
                            .and_then(|s| s.GetColorValue(UIColorType::Background))
                            .map(|c| u32::from(c.R) + u32::from(c.G) + u32::from(c.B) < 384)
                            .unwrap_or(true);
                        RoUninitialize();
                        dark
                    }
                }
            };
            let colors = if high_contrast {
                Colors {
                    background: COLORREF(GetSysColor(COLOR_WINDOW)),
                    text: COLORREF(GetSysColor(COLOR_WINDOWTEXT)),
                    secondary: COLORREF(GetSysColor(COLOR_GRAYTEXT)),
                    hover: COLORREF(GetSysColor(COLOR_HIGHLIGHT)),
                    border: COLORREF(GetSysColor(COLOR_WINDOWTEXT)),
                    dark,
                    high_contrast,
                }
            } else if dark {
                Colors {
                    background: rgb(33, 33, 33),
                    text: rgb(236, 236, 236),
                    secondary: rgb(166, 166, 166),
                    hover: rgb(53, 53, 53),
                    border: rgb(76, 76, 76),
                    dark,
                    high_contrast,
                }
            } else {
                Colors {
                    background: rgb(250, 250, 250),
                    text: rgb(32, 32, 32),
                    secondary: rgb(99, 99, 99),
                    hover: rgb(234, 234, 234),
                    border: rgb(186, 186, 186),
                    dark,
                    high_contrast,
                }
            };
            let brush = CreateSolidBrush(colors.background);
            if brush.is_invalid() {
                return Err(Error::from_thread());
            }
            Ok(Self { colors, brush })
        }
    }
}
impl Drop for Palette {
    fn drop(&mut self) {
        // SAFETY: this owns the brush; controls only borrow it for synchronous painting.
        unsafe {
            let _ = DeleteObject(self.brush.into());
        }
    }
}
pub(super) fn caption(hwnd: HWND, colors: Colors) {
    if hwnd.is_invalid() {
        return;
    }
    // SAFETY: DWM copies these scalar attributes synchronously; caption buttons stay native.
    unsafe {
        let dark = i32::from(colors.dark && !colors.high_contrast);
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&dark as *const i32).cast(),
            size_of::<i32>() as u32,
        );
        let background = if colors.high_contrast {
            COLORREF(0xffffffff)
        } else {
            colors.background
        };
        let text = if colors.high_contrast {
            COLORREF(0xffffffff)
        } else {
            colors.text
        };
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_COLOR,
            (&background as *const COLORREF).cast(),
            size_of::<COLORREF>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_TEXT_COLOR,
            (&text as *const COLORREF).cast(),
            size_of::<COLORREF>() as u32,
        );
    }
}
pub(super) fn fill(dc: HDC, bounds: &RECT, color: COLORREF) {
    // SAFETY: the stock DC brush is never deleted; color is local to this borrowed DC.
    unsafe {
        SetDCBrushColor(dc, color);
        FillRect(dc, bounds, HBRUSH(GetStockObject(DC_BRUSH).0));
    }
}
pub(super) fn rounded(dc: HDC, r: RECT, fill: COLORREF, border: COLORREF, radius: i32) {
    // SAFETY: both stock selections are restored before returning to the control.
    unsafe {
        let brush = SelectObject(dc, GetStockObject(DC_BRUSH));
        let pen = SelectObject(dc, GetStockObject(DC_PEN));
        SetDCBrushColor(dc, fill);
        SetDCPenColor(dc, border);
        let _ = RoundRect(dc, r.left, r.top, r.right, r.bottom, radius, radius);
        SelectObject(dc, pen);
        SelectObject(dc, brush);
    }
}
impl App {
    pub(super) fn colors(&self) -> Colors {
        self.palette.borrow().colors
    }
    pub(super) fn set_appearance(&self, choice: Choice) {
        if self.appearance.get() == choice {
            return;
        }
        if let Err(error) = choice.save() {
            dialog(
                Some(self.hwnd),
                &format!("Impossible de mémoriser l’apparence.\n\n{error}"),
                MB_OK | MB_ICONERROR,
            );
            return;
        }
        self.appearance.set(choice);
        if let Err(error) = self.refresh_appearance() {
            dialog(
                Some(self.hwnd),
                &format!("Le thème sera réappliqué au prochain lancement.\n\n{error}"),
                MB_OK | MB_ICONERROR,
            );
        }
    }
    pub(super) fn refresh_appearance(&self) -> Result<()> {
        let palette = Palette::new(self.appearance.get())?;
        let colors = palette.colors;
        let old = self.palette.replace(palette);
        caption(self.hwnd, colors);
        caption(self.settings.get(), colors);
        self.refresh_chrome_resources()?;
        self.refresh_settings();
        // SAFETY: these are owned windows; repaint after swapping the shared palette.
        unsafe {
            let _ = RedrawWindow(
                Some(self.hwnd),
                None,
                None,
                RDW_INVALIDATE | RDW_ALLCHILDREN | RDW_FRAME,
            );
            if !self.settings.get().is_invalid() {
                let _ = RedrawWindow(
                    Some(self.settings.get()),
                    None,
                    None,
                    RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN | RDW_FRAME,
                );
            }
        }
        drop(old);
        Ok(())
    }
    pub(super) fn paint_settings_background(&self, dc: HDC) {
        let mut bounds = RECT::default();
        // SAFETY: the DC is supplied for this window's synchronous erase message.
        unsafe {
            let _ = GetClientRect(self.settings.get(), &mut bounds);
        }
        fill(dc, &bounds, self.colors().background);
    }
}
