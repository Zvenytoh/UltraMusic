//! Update UI coordination. Workers exchange owned data; no COM crosses threads.
use super::{dialog, App, UPDATE_READY};
use crate::update::{self, Download, Release};
use std::sync::mpsc;
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{
        DestroyWindow, PostMessageW, IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION,
        MB_OK, MB_YESNO,
    },
};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Phase {
    Idle,
    Checking,
    Downloading,
    Preparing,
}

pub(super) enum Event {
    Checked(Result<Option<Release>, String>),
    Downloaded(Result<Download, String>),
    Prepared(Result<update::install::Prepared, String>),
}

impl App {
    pub(super) fn update_label(&self) -> &'static str {
        match self.update_phase.get() {
            Phase::Idle => "Vérifier les mises à jour",
            Phase::Checking => "Afficher le résultat de la vérification…",
            Phase::Downloading => "Téléchargement de la mise à jour…",
            Phase::Preparing => "Préparation de l’installation…",
        }
    }

    pub(super) fn check_updates(&self, manual: bool) {
        if self.update_phase.get() != Phase::Idle {
            // A startup check is silent, but a deliberate menu click should still
            // make its result visible instead of being discarded.
            if manual {
                self.update_manual.set(true);
            }
            return;
        }
        self.update_manual.set(manual);
        self.update_task(Phase::Checking, || Event::Checked(update::check()));
    }

    fn update_task(&self, phase: Phase, work: impl FnOnce() -> Event + Send + 'static) {
        let (sender, receiver) = mpsc::channel();
        *self.update_receiver.borrow_mut() = Some(receiver);
        self.update_phase.set(phase);
        self.refresh_ui();
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
            self.update_phase.set(Phase::Idle);
            self.refresh_ui();
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
                    self.update_task(Phase::Downloading, move || {
                        Event::Downloaded(update::download(release))
                    });
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
                let service = self.service.get().argument().to_string();
                let mut options = vec![service];
                if self.keep_rendering.get() {
                    options.push("--keep-rendering".to_string());
                }
                self.update_task(Phase::Preparing, move || {
                    Event::Prepared(update::install::prepare(download, &options))
                });
                return;
            }
            Event::Prepared(Ok(prepared)) => {
                match prepared.launch() {
                    Ok(()) => {
                        // SAFETY: this event is dispatched on the window's owning thread.
                        unsafe {
                            let _ = DestroyWindow(self.hwnd);
                        }
                    }
                    Err(error) => self.update_error(&error),
                }
            }
            Event::Downloaded(Err(error)) | Event::Prepared(Err(error)) => {
                self.update_error(&error)
            }
            _ => {}
        }
        self.update_phase.set(Phase::Idle);
        self.refresh_ui();
    }

    fn update_error(&self, error: &str) {
        dialog(
            Some(self.hwnd),
            &format!("Mise à jour impossible.\n\n{error}"),
            MB_OK | MB_ICONERROR,
        );
    }
}
