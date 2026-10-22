//! Shared actions for the tray, header and settings window.
use super::{dialog, App};
use crate::config::Service;
use windows::Win32::UI::WindowsAndMessaging::*;

pub(super) const OPEN: usize = 1;
pub(super) const RESTART: usize = 2;
pub(super) const COMPAT: usize = 3;
pub(super) const QUIT: usize = 4;
pub(super) const UPDATE: usize = 5;
pub(super) const PLAY_PAUSE: usize = 6;
pub(super) const MUTE: usize = 7;
pub(super) const VOLUME_DOWN: usize = 8;
pub(super) const VOLUME_UP: usize = 9;
pub(super) const SETTINGS: usize = 13;
pub(super) const BLOCKER: usize = 14;
pub(super) const BACK: usize = 15;
pub(super) const FORWARD: usize = 19;
pub(super) const THEME_DARK: usize = 16;
pub(super) const THEME_LIGHT: usize = 17;
pub(super) const THEME_SYSTEM: usize = 18;
pub(super) const SERVICES: [(usize, Service); 3] = [
    (11, Service::Apple),
    (10, Service::Youtube),
    (12, Service::YoutubeVideo),
];

impl App {
    pub(super) fn command(&self, command: usize) {
        // The helper has captured restart options; native controls are disabled for this short phase.
        if self.update_phase.get() == super::updates::Phase::Preparing
            && (command == COMPAT || SERVICES.iter().any(|(id, _)| *id == command))
        {
            return;
        }

        if let Some((_, service)) = SERVICES.iter().find(|(id, _)| *id == command) {
            self.select_service(*service);
            return;
        }
        match command {
            OPEN => self.restore(),
            SETTINGS => {
                self.restore();
                self.open_settings();
            }
            PLAY_PAUSE => self.play_pause(),
            MUTE => self.toggle_mute(),
            VOLUME_DOWN => self.set_volume(self.volume.get() - 0.1),
            VOLUME_UP => self.set_volume(self.volume.get() + 0.1),
            UPDATE => self.check_updates(true),
            BLOCKER => self.toggle_blocker(),
            BACK => self.go_back(),
            FORWARD => self.go_forward(),
            THEME_DARK => self.set_appearance(super::theme::Choice::Dark),
            THEME_LIGHT => self.set_appearance(super::theme::Choice::Light),
            THEME_SYSTEM => self.set_appearance(super::theme::Choice::System),
            RESTART => {
                if dialog(
                    Some(self.hwnd),
                    "Redémarrer le lecteur ? La lecture sera interrompue ; la session sera conservée.",
                    MB_YESNO | MB_ICONQUESTION,
                ) == IDYES
                    && self.start().is_err()
                {
                    self.report_failure();
                }
            }
            COMPAT => {
                self.keep_rendering.set(!self.keep_rendering.get());
                if self.hidden.get() {
                    if let Some(controller) = self.controller() {
                        // SAFETY: this controller belongs to the UI thread.
                        unsafe {
                            let _ = controller.SetIsVisible(self.keep_rendering.get());
                        }
                    }
                }
            }
            QUIT => {
                // SAFETY: command dispatch runs on the owning UI thread.
                unsafe {
                    let _ = DestroyWindow(self.hwnd);
                }
            }
            _ => {}
        }
        self.refresh_ui();
    }
}
