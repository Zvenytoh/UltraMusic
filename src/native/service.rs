//! Explicit provider changes recreate the player and preserve the shared profile.
use super::{dialog, App};
use crate::config::Service;
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK};

impl App {
    pub(super) fn select_service(&self, service: Service) {
        // Persist before changing the live player so a write failure leaves it intact.
        if let Err(error) = service.save() {
            dialog(
                Some(self.hwnd),
                &format!("Impossible de mémoriser le service choisi.\n\n{error}"),
                MB_OK | MB_ICONERROR,
            );
            return;
        }
        if self.service.get() == service {
            self.restore();
            return;
        }
        self.service.set(service);
        // start invalidates callbacks from the previous generation before closing it.
        if self.start().is_err() {
            self.report_failure();
        }
        self.restore();
    }
}
