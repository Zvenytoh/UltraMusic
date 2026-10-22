//! Bundled uBO Lite lifecycle. Status describes extension setup, not ad removal.
use super::{current, dialog, App};
use crate::config::Service;
use std::{
    ffi::OsString,
    fs,
    io::Read,
    path::{Component, PathBuf, Prefix},
};
use webview2_com::{
    BrowserExtensionEnableCompletedHandler,
    Microsoft::Web::WebView2::Win32::{
        ICoreWebView2BrowserExtension, ICoreWebView2Profile7, ICoreWebView2_13,
    },
    ProfileAddBrowserExtensionCompletedHandler,
};
use windows::{
    core::{Interface, BOOL, HSTRING},
    Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK},
};

#[derive(Clone, Copy)]
pub(super) enum State {
    Loading,
    Enabled,
    Disabled,
    Unavailable,
}

fn preference_path() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(|base| PathBuf::from(base).join("UltraMusic").join("blocker.txt"))
}

pub(super) fn load_preference() -> bool {
    let saved = (|| {
        let mut value = String::new();
        fs::File::open(preference_path()?)
            .ok()?
            .take(16)
            .read_to_string(&mut value)
            .ok()?;
        Some(value)
    })();
    saved.as_deref() != Some("off\n")
}

fn save_preference(enabled: bool) -> std::io::Result<()> {
    let path =
        preference_path().ok_or_else(|| std::io::Error::other("LOCALAPPDATA indisponible"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, if enabled { "on\n" } else { "off\n" })?;
    fs::rename(temporary, path)
}

/// Use one ordinary local-drive spelling even after an update's canonical-path restart.
fn package_path() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    let path = executable.parent()?.join("blocker");
    let mut components = path.components();
    let Component::Prefix(prefix) = components.next()? else {
        return None;
    };
    let (Prefix::Disk(drive) | Prefix::VerbatimDisk(drive)) = prefix.kind() else {
        return None;
    };
    let mut plain = OsString::from(format!("{}:", char::from(drive)));
    plain.push(components.as_path());
    let path = PathBuf::from(plain);
    path.join("manifest.json").is_file().then_some(path)
}

impl App {
    fn wants_blocking(&self) -> bool {
        self.blocker_enabled.get() && self.service.get() != Service::Apple
    }

    pub(super) fn blocker_status(&self) -> &'static str {
        match self.blocker_state.get() {
            State::Loading => "Initialisation du module…",
            State::Unavailable => "Indisponible · vérifier le module ou WebView2",
            State::Enabled => "Module uBO Lite activé",
            State::Disabled if self.service.get() == Service::Apple => {
                "Non utilisé sur Apple Music"
            }
            State::Disabled => "Module désactivé",
        }
    }

    pub(super) fn toggle_blocker(&self) {
        let enabled = !self.blocker_enabled.get();
        if let Err(error) = save_preference(enabled) {
            dialog(
                Some(self.hwnd),
                &format!("Impossible de mémoriser le réglage du bloqueur.\n\n{error}"),
                MB_OK | MB_ICONERROR,
            );
            return;
        }
        self.blocker_enabled.set(enabled);
        self.refresh_ui();
        // An in-flight callback reads the latest preference before navigation.
        if !self.blocker_busy.get() && self.controller().is_some() {
            self.begin_blocker(self.generation.get(), true);
        }
    }

    pub(super) fn begin_blocker(&self, generation: usize, reload: bool) {
        self.blocker_busy.set(true);
        self.blocker_state.set(State::Loading);
        self.refresh_ui();
        let extension = self.blocker.borrow().clone();
        if let Some(extension) = extension {
            self.apply_blocker(extension, generation, reload);
            return;
        }
        let result = (|| -> windows::core::Result<()> {
            let controller = self
                .controller()
                .ok_or_else(windows::core::Error::from_thread)?;
            let path = package_path().ok_or_else(windows::core::Error::from_thread)?;
            // SAFETY: interfaces and the callback are owned by the UI STA; no borrow spans calls.
            unsafe {
                let web: ICoreWebView2_13 = controller.CoreWebView2()?.cast()?;
                let profile: ICoreWebView2Profile7 = web.Profile()?.cast()?;
                profile.AddBrowserExtension(
                    &HSTRING::from(path.as_os_str()),
                    &ProfileAddBrowserExtensionCompletedHandler::create(Box::new(
                        move |hr, extension| {
                            if let Some(app) = current(generation) {
                                if hr.is_ok() {
                                    if let Some(extension) = extension {
                                        *app.blocker.borrow_mut() = Some(extension.clone());
                                        app.apply_blocker(extension, generation, reload);
                                        return Ok(());
                                    }
                                }
                                app.finish_blocker(State::Unavailable, reload);
                            }
                            Ok(())
                        },
                    )),
                )
            }
        })();
        if result.is_err() && self.generation.get() == generation {
            self.finish_blocker(State::Unavailable, reload);
        }
    }

    fn apply_blocker(
        &self,
        extension: ICoreWebView2BrowserExtension,
        generation: usize,
        reload: bool,
    ) {
        let enabled = self.wants_blocking();
        let completed_extension = extension.clone();
        // SAFETY: Enable is called on its owning STA. Generation checks reject stale completions.
        let result = unsafe {
            extension.Enable(
                enabled,
                &BrowserExtensionEnableCompletedHandler::create(Box::new(move |hr| {
                    if let Some(app) = current(generation) {
                        if hr.is_err() {
                            app.finish_blocker(State::Unavailable, reload);
                            return Ok(());
                        }
                        if app.wants_blocking() != enabled {
                            app.apply_blocker(completed_extension.clone(), generation, reload);
                            return Ok(());
                        }
                        let mut actual = BOOL(0);
                        let state = if completed_extension.IsEnabled(&mut actual).is_err()
                            || actual.as_bool() != enabled
                        {
                            State::Unavailable
                        } else if enabled {
                            State::Enabled
                        } else {
                            State::Disabled
                        };
                        app.finish_blocker(state, reload);
                    }
                    Ok(())
                })),
            )
        };
        if result.is_err() && self.generation.get() == generation {
            self.finish_blocker(State::Unavailable, reload);
        }
    }

    fn finish_blocker(&self, state: State, reload: bool) {
        self.blocker_busy.set(false);
        self.blocker_state.set(state);
        self.refresh_ui();
        if let Some(controller) = self.controller() {
            // SAFETY: completion runs on the current UI STA, without retained borrows.
            let result = unsafe {
                controller.CoreWebView2().and_then(|web| {
                    if reload {
                        web.Reload()
                    } else {
                        web.Navigate(&HSTRING::from(self.service.get().home()))
                    }
                })
            };
            if result.is_err() {
                self.report_failure();
            }
        }
    }
}
