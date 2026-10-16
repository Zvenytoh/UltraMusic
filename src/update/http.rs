//! Synchronous WinHTTP transport used only on short-lived update worker threads.
use std::{
    ffi::c_void,
    ptr::NonNull,
    time::{Duration, Instant},
};
use url::Url;
use windows::{
    core::{w, HSTRING, PCWSTR},
    Win32::Networking::WinHttp::*,
};

struct Handle(NonNull<c_void>);
impl Handle {
    fn new(raw: *mut c_void) -> Result<Self, String> {
        NonNull::new(raw)
            .map(Self)
            .ok_or_else(|| "Connexion HTTPS indisponible.".into())
    }
    fn raw(&self) -> *mut c_void {
        self.0.as_ptr()
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: each handle is owned once, and requests are dropped before parents.
        unsafe {
            let _ = WinHttpCloseHandle(self.raw());
        }
    }
}

pub(super) struct Response {
    request: Handle,
    _connection: Handle,
    _session: Handle,
    started: Instant,
}

fn transport_error(_: windows::core::Error) -> String {
    "Connexion à GitHub impossible. Vérifiez le réseau puis réessayez.".into()
}

fn allowed(url: &Url) -> bool {
    url.scheme() == "https"
        && url.port_or_known_default() == Some(443)
        && url.username().is_empty()
        && url.password().is_none()
        && matches!(
            url.host_str(),
            Some(
                "api.github.com"
                    | "github.com"
                    | "release-assets.githubusercontent.com"
                    | "objects.githubusercontent.com"
            )
        )
}

pub(super) fn get(address: &str) -> Result<Response, String> {
    let mut url = Url::parse(address).map_err(|_| "Adresse de mise à jour invalide.")?;
    for _ in 0..6 {
        if !allowed(&url) {
            return Err("La redirection de téléchargement n’est pas autorisée.".into());
        }
        // SAFETY: all strings and buffers live through the synchronous calls. HTTPS
        // uses normal certificate validation; redirect destinations are checked here.
        let response = unsafe {
            let agent = HSTRING::from(concat!("UltraMusic/", env!("CARGO_PKG_VERSION")));
            let session = Handle::new(WinHttpOpen(
                &agent,
                WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
                PCWSTR::null(),
                PCWSTR::null(),
                0,
            ))?;
            WinHttpSetTimeouts(session.raw(), 5_000, 10_000, 10_000, 15_000)
                .map_err(transport_error)?;
            let host = HSTRING::from(url.host_str().ok_or("Hôte manquant.")?);
            let connection = Handle::new(WinHttpConnect(session.raw(), &host, 443, 0))?;
            let mut path = url.path().to_owned();
            if let Some(query) = url.query() {
                path.push('?');
                path.push_str(query);
            }
            let path = HSTRING::from(path);
            let request = Handle::new(WinHttpOpenRequest(
                connection.raw(),
                w!("GET"),
                &path,
                PCWSTR::null(),
                PCWSTR::null(),
                std::ptr::null(),
                WINHTTP_FLAG_SECURE,
            ))?;
            WinHttpSetOption(
                Some(request.raw().cast_const()),
                WINHTTP_OPTION_REDIRECT_POLICY,
                Some(&WINHTTP_OPTION_REDIRECT_POLICY_NEVER.to_ne_bytes()),
            )
            .map_err(transport_error)?;
            let headers: Vec<u16> = "Accept: application/vnd.github+json\r\n"
                .encode_utf16()
                .collect();
            WinHttpSendRequest(request.raw(), Some(&headers), None, 0, 0, 0)
                .map_err(transport_error)?;
            WinHttpReceiveResponse(request.raw(), std::ptr::null_mut()).map_err(transport_error)?;
            Response {
                request,
                _connection: connection,
                _session: session,
                started: Instant::now(),
            }
        };
        let status = response.status()?;
        if status == 200 {
            return Ok(response);
        }
        if matches!(status, 301 | 302 | 303 | 307 | 308) {
            let destination = response.location()?;
            url = url
                .join(&destination)
                .map_err(|_| "Redirection invalide.")?;
        } else {
            return Err(format!(
                "GitHub est indisponible pour cette vérification (HTTP {status})."
            ));
        }
    }
    Err("Trop de redirections pour télécharger la mise à jour.".into())
}

impl Response {
    fn status(&self) -> Result<u32, String> {
        let mut status = 0_u32;
        let mut length = 4;
        // SAFETY: status is a writable u32, matching QUERY_FLAG_NUMBER and length.
        unsafe {
            WinHttpQueryHeaders(
                self.request.raw(),
                WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                PCWSTR::null(),
                Some((&mut status as *mut u32).cast()),
                &mut length,
                std::ptr::null_mut(),
            )
            .map_err(transport_error)?;
        }
        Ok(status)
    }
    fn location(&self) -> Result<String, String> {
        let mut buffer = vec![0_u16; 16_384];
        let mut length = (buffer.len() * 2) as u32;
        // SAFETY: the byte length describes the allocated UTF-16 buffer exactly.
        unsafe {
            WinHttpQueryHeaders(
                self.request.raw(),
                WINHTTP_QUERY_LOCATION,
                PCWSTR::null(),
                Some(buffer.as_mut_ptr().cast()),
                &mut length,
                std::ptr::null_mut(),
            )
            .map_err(transport_error)?;
        }
        let length = buffer.iter().position(|c| *c == 0).unwrap_or(buffer.len());
        String::from_utf16(&buffer[..length]).map_err(|_| "Adresse de redirection invalide.".into())
    }
    pub(super) fn read(&self, buffer: &mut [u8]) -> Result<usize, String> {
        if self.started.elapsed() > Duration::from_secs(300) {
            return Err("Le téléchargement a dépassé le délai autorisé.".into());
        }
        let mut read = 0;
        // SAFETY: the caller owns the buffer for the full synchronous read.
        unsafe {
            WinHttpReadData(
                self.request.raw(),
                buffer.as_mut_ptr().cast(),
                buffer.len() as u32,
                &mut read,
            )
            .map_err(transport_error)?;
        }
        Ok(read as usize)
    }
}
