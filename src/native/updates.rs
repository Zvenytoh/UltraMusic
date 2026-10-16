//! Update UI coordination. Workers exchange owned data; no COM crosses threads.
use super::{dialog, App, UPDATE_READY};
use crate::{
    config::Service,
    update::{self, Download, Release},
};
use std::sync::mpsc;
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{
        DestroyWindow, PostMessageW, IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION,
        MB_OK, MB_YESNO,
    },
};

pub(super) enum Event {
    Checked(Result<Option<Release>, String>),
    Downloaded(Result<Download, String>),
}

impl App {
    pub(super) fn check_updates(&self, manual: bool) {
        if self.update_busy.get() {
            return;
        }
        self.update_manual.set(manual);
        self.update_task(|| Event::Checked(update::check()));
    }

    fn update_task(&self, work: impl FnOnce() -> Event + Send + 'static) {
        let (sender, receiver) = mpsc::channel();
        *self.update_receiver.borrow_mut() = Some(receiver);
        self.update_busy.set(true);
        let address = self.hwnd.0 as isize;
        if std::thread::Builder::new()
            .name("ultramusic-update".into())
            .spawn(move || {
                let event = work();
                if sender.send(event).is_ok() {
                    // SAFETY: this is a pointer-free wake-up message. No UI/COM state is
                    // dereferenced on this worker, and it is safe if the window has exited.
                    unsafe {
                        let _ = PostMessageW(
                            Some(HWND(address as *mut _)),
                            UPDATE_READY,
                            WPARAM(0),
                            LPARAM(0),
                        );
                    }
                }
            })
            .is_err()
        {
            self.update_receiver.borrow_mut().take();
            self.update_busy.set(false);
            if self.update_manual.get() {
                self.update_error("Impossible de démarrer la vérification des mises à jour.");
            }
        }
    }

    pub(super) fn update_event(&self) {
        let event = self
            .update_receiver
            .borrow()
            .as_ref()
            .and_then(|receiver| receiver.try_recv().ok());
        let Some(event) = event else { return };
        self.update_receiver.borrow_mut().take();
        match event {
            Event::Checked(Ok(Some(release))) => {
                let message = format!("UltraMusic {} est disponible (version installée : {}).\n\nInstaller maintenant ? Le lecteur sera fermé et la lecture interrompue après le téléchargement. Votre session sera conservée.", release.version, env!("CARGO_PKG_VERSION"));
                if dialog(Some(self.hwnd), &message, MB_YESNO | MB_ICONQUESTION) == IDYES {
                    self.update_manual.set(true);
                    self.update_task(move || Event::Downloaded(update::download(release)));
                    return;
                }
            }
            Event::Checked(Ok(None)) if self.update_manual.get() => {
                dialog(
                    Some(self.hwnd),
                    concat!(
                        "Aucune mise à jour compatible disponible.\nVersion installée : ",
                        env!("CARGO_PKG_VERSION")
                    ),
                    MB_OK | MB_ICONINFORMATION,
                );
            }
            Event::Checked(Err(error)) if self.update_manual.get() => self.update_error(&error),
            Event::Downloaded(Ok(download)) => {
                let service = match self.config.service {
                    Service::Apple => "--apple",
                    Service::Youtube => "--youtube",
                };
                let mut options = vec![service];
                if self.keep_rendering.get() {
                    options.push("--keep-rendering");
                }
                match update::install::launch(download, &options) {
                    Ok(()) => {
                        // SAFETY: this event is dispatched on the window's owning thread.
                        unsafe {
                            let _ = DestroyWindow(self.hwnd);
                        }
                    }
                    Err(error) => self.update_error(&error),
                }
            }
            Event::Downloaded(Err(error)) => self.update_error(&error),
            _ => {}
        }
        self.update_busy.set(false);
    }

    fn update_error(&self, error: &str) {
        dialog(
            Some(self.hwnd),
            &format!("Mise à jour impossible.\n\n{error}"),
            MB_OK | MB_ICONERROR,
        );
    }
}
