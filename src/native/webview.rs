//! WebView2 initialization, navigation policy, and generation-scoped callbacks.
use super::{current, App, EXTERNAL, FAILED};
use crate::config::external_url;
use webview2_com::{
    CoTaskMemPWSTR, CreateCoreWebView2ControllerCompletedHandler,
    CreateCoreWebView2EnvironmentCompletedHandler, DownloadStartingEventHandler,
    Microsoft::Web::WebView2::Win32::{
        CreateCoreWebView2EnvironmentWithOptions, ICoreWebView2Controller, ICoreWebView2_4,
        COREWEBVIEW2_PERMISSION_KIND_CAMERA, COREWEBVIEW2_PERMISSION_KIND_CLIPBOARD_READ,
        COREWEBVIEW2_PERMISSION_KIND_FILE_READ_WRITE, COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION,
        COREWEBVIEW2_PERMISSION_KIND_LOCAL_FONTS, COREWEBVIEW2_PERMISSION_KIND_MICROPHONE,
        COREWEBVIEW2_PERMISSION_KIND_MIDI_SYSTEM_EXCLUSIVE_MESSAGES,
        COREWEBVIEW2_PERMISSION_KIND_MULTIPLE_AUTOMATIC_DOWNLOADS,
        COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS, COREWEBVIEW2_PERMISSION_KIND_OTHER_SENSORS,
        COREWEBVIEW2_PERMISSION_KIND_WINDOW_MANAGEMENT, COREWEBVIEW2_PERMISSION_STATE_DENY,
    },
    NavigationStartingEventHandler, NewWindowRequestedEventHandler,
    PermissionRequestedEventHandler, ProcessFailedEventHandler,
};
use windows::{
    core::{Interface, Result as WinResult, BOOL, HSTRING, PWSTR},
    Win32::{
        Foundation::{LPARAM, WPARAM},
        UI::WindowsAndMessaging::PostMessageW,
    },
};
impl App {
    pub(super) fn controller(&self) -> Option<ICoreWebView2Controller> {
        self.controller.borrow().clone()
    }
    pub(super) fn close_webview(&self) {
        self.generation.set(self.generation.get().wrapping_add(1));
        self.external.borrow_mut().take();
        let controller = self.controller.borrow_mut().take();
        if let Some(c) = controller {
            // SAFETY: c is an owned STA reference and the RefCell borrow has ended.
            unsafe {
                let _ = c.Close();
            }
        }
    }
    pub(super) fn report_failure(&self) {
        // Defer dialogs until after the COM callback has returned.
        if !self.failed.replace(true) {
            // SAFETY: the message contains only a numeric generation, never a pointer.
            unsafe {
                let sent = PostMessageW(
                    Some(self.hwnd),
                    FAILED,
                    WPARAM(self.generation.get()),
                    LPARAM(0),
                );
                if sent.is_err() {
                    self.failed.set(false);
                }
            }
        }
    }
    pub(super) fn start(&self) -> WinResult<()> {
        self.close_webview();
        self.failed.set(false);
        let generation = self.generation.get();
        let handler =
            CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(move |hr, env| {
                let Some(a) = current(generation) else {
                    return Ok(());
                };
                if hr.is_err() {
                    a.report_failure();
                    return Ok(());
                }
                let Some(env) = env else {
                    a.report_failure();
                    return Ok(());
                };
                let handler = CreateCoreWebView2ControllerCompletedHandler::create(Box::new(
                    move |hr, controller| {
                        let Some(a) = current(generation) else {
                            if let Some(c) = controller {
                                // SAFETY: this stale controller was created on this same STA.
                                unsafe {
                                    let _ = c.Close();
                                }
                            }
                            return Ok(());
                        };
                        if hr.is_err() {
                            a.report_failure();
                            return Ok(());
                        }
                        if let Some(c) = controller {
                            *a.controller.borrow_mut() = Some(c.clone());
                            if a.configure(&c, generation).is_err() {
                                a.close_webview();
                                a.report_failure();
                            }
                        } else {
                            a.report_failure();
                        }
                        Ok(())
                    },
                ));
                // SAFETY: the current generation still owns hwnd; COM retains the callback.
                if unsafe { env.CreateCoreWebView2Controller(a.hwnd, &handler) }.is_err() {
                    a.report_failure();
                }
                Ok(())
            }));
        // SAFETY: profile is a valid UTF-16 string and WebView2 retains the callback.
        unsafe { CreateCoreWebView2EnvironmentWithOptions(None, &self.profile, None, &handler) }
    }
    fn configure(&self, controller: &ICoreWebView2Controller, generation: usize) -> WinResult<()> {
        // SAFETY: the controller and event arguments belong to this STA. Uri strings
        // returned by COM are immediately wrapped in CoTaskMemPWSTR for correct freeing.
        unsafe {
            self.resize();
            controller.SetIsVisible(!self.hidden.get() || self.keep_rendering.get())?;
            let web = controller.CoreWebView2()?;
            self.attach_media(&web, generation)?;
            let settings = web.Settings()?;
            settings.SetIsWebMessageEnabled(false)?;
            settings.SetAreHostObjectsAllowed(false)?;
            let mut token = 0;
            web.add_PermissionRequested(
                &PermissionRequestedEventHandler::create(Box::new(move |_, args| {
                    if let Some(args) = args {
                        let mut kind = Default::default();
                        args.PermissionKind(&mut kind)?;
                        if matches!(
                            kind,
                            COREWEBVIEW2_PERMISSION_KIND_CAMERA
                                | COREWEBVIEW2_PERMISSION_KIND_MICROPHONE
                                | COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION
                                | COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS
                                | COREWEBVIEW2_PERMISSION_KIND_CLIPBOARD_READ
                                | COREWEBVIEW2_PERMISSION_KIND_OTHER_SENSORS
                                | COREWEBVIEW2_PERMISSION_KIND_MIDI_SYSTEM_EXCLUSIVE_MESSAGES
                                | COREWEBVIEW2_PERMISSION_KIND_FILE_READ_WRITE
                                | COREWEBVIEW2_PERMISSION_KIND_LOCAL_FONTS
                                | COREWEBVIEW2_PERMISSION_KIND_WINDOW_MANAGEMENT
                                | COREWEBVIEW2_PERMISSION_KIND_MULTIPLE_AUTOMATIC_DOWNLOADS
                        ) {
                            args.SetState(COREWEBVIEW2_PERMISSION_STATE_DENY)?;
                        }
                    }
                    Ok(())
                })),
                &mut token,
            )?;
            // Streaming remains available; pages cannot save attachments through this host.
            let downloads: ICoreWebView2_4 = web.cast()?;
            downloads.add_DownloadStarting(
                &DownloadStartingEventHandler::create(Box::new(move |_, args| {
                    if let Some(args) = args {
                        args.SetCancel(true)?;
                        args.SetHandled(true)?;
                    }
                    Ok(())
                })),
                &mut token,
            )?;
            web.add_NavigationStarting(
                &NavigationStartingEventHandler::create(Box::new(move |_, args| {
                    if let (Some(a), Some(args)) = (current(generation), args) {
                        let mut uri = PWSTR::null();
                        args.Uri(&mut uri)?;
                        let uri = CoTaskMemPWSTR::from(uri).to_string();
                        if !a.service.get().allows(&uri) {
                            args.SetCancel(true)?;
                            let mut user = BOOL(0);
                            args.IsUserInitiated(&mut user)?;
                            if user.as_bool() {
                                a.queue_external(uri);
                            }
                        }
                    }
                    Ok(())
                })),
                &mut token,
            )?;
            web.add_NewWindowRequested(
                &NewWindowRequestedEventHandler::create(Box::new(move |sender, args| {
                    if let Some(args) = args {
                        args.SetHandled(true)?;
                        if let Some(a) = current(generation) {
                            let mut uri = PWSTR::null();
                            args.Uri(&mut uri)?;
                            let uri = CoTaskMemPWSTR::from(uri).to_string();
                            let mut user = BOOL(0);
                            args.IsUserInitiated(&mut user)?;
                            if user.as_bool() {
                                if a.service.get().allows(&uri) {
                                    if let Some(web) = sender {
                                        web.Navigate(&HSTRING::from(uri))?;
                                    }
                                } else {
                                    a.queue_external(uri);
                                }
                            }
                        }
                    }
                    Ok(())
                })),
                &mut token,
            )?;
            web.add_ProcessFailed(
                &ProcessFailedEventHandler::create(Box::new(move |_, _| {
                    if let Some(a) = current(generation) {
                        a.report_failure();
                    }
                    Ok(())
                })),
                &mut token,
            )?;
            web.Navigate(&HSTRING::from(self.service.get().home()))?;
        }
        Ok(())
    }
    fn queue_external(&self, uri: String) {
        if external_url(&uri) && self.external.borrow().is_none() {
            *self.external.borrow_mut() = Some(uri);
            // SAFETY: only the numeric generation crosses the message queue boundary.
            unsafe {
                let sent = PostMessageW(
                    Some(self.hwnd),
                    EXTERNAL,
                    WPARAM(self.generation.get()),
                    LPARAM(0),
                );
                if sent.is_err() {
                    self.external.borrow_mut().take();
                }
            }
        }
    }
}
