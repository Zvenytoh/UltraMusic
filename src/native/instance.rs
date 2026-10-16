//! Session-local single-instance guard and bounded startup activation.
use super::{CLASS, RESTORE};
use std::time::{Duration, Instant};
use windows::{
    core::w,
    Win32::{
        Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, LPARAM, WPARAM},
        System::Threading::CreateMutexW,
        UI::WindowsAndMessaging::{FindWindowW, PostMessageW},
    },
};

/// Keeps the named kernel object alive until the application has shut down.
pub(super) struct InstanceGuard(HANDLE);

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        // SAFETY: this guard owns the handle returned by CreateMutexW exactly once.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// Returns `None` after activating an existing instance, or an owned first-instance guard.
pub(super) fn acquire() -> Result<Option<InstanceGuard>, String> {
    // SAFETY: the name is a static, null-terminated UTF-16 string; no borrowed pointers escape.
    let (guard, exists) = unsafe {
        let handle = CreateMutexW(None, false, w!("Local\\UltraMusic.Native.1"))
            .map_err(|error| error.to_string())?;
        let exists = GetLastError() == ERROR_ALREADY_EXISTS;
        (InstanceGuard(handle), exists)
    };
    if !exists {
        return Ok(Some(guard));
    }

    // Only the short-lived second process waits. The music host has no polling loop.
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        // SAFETY: FindWindowW provides a window handle; PostMessageW sends no pointers.
        unsafe {
            if let Ok(hwnd) = FindWindowW(CLASS, None) {
                if PostMessageW(Some(hwnd), RESTORE, WPARAM(0), LPARAM(0)).is_ok() {
                    return Ok(None);
                }
            }
        }
        if Instant::now() >= deadline {
            return Err("Une instance UltraMusic existe, mais sa fenêtre reste indisponible. Réessayez dans quelques instants.".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
