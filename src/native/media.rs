//! On-demand media actions and application-scoped Windows audio sessions.
use super::{current, dialog, App, MEDIA_FAILED};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, io::Read, mem::size_of};
use webview2_com::{
    CallDevToolsProtocolMethodCompletedHandler, IsDocumentPlayingAudioChangedEventHandler,
    Microsoft::Web::WebView2::Win32::{ICoreWebView2, ICoreWebView2_8},
};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub(super) struct Preferences {
    pub(super) volume: f32,
    pub(super) muted: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            volume: 1.0,
            muted: false,
        }
    }
}
impl Preferences {
    fn path() -> Option<std::path::PathBuf> {
        std::env::var_os("LOCALAPPDATA").map(|root| {
            std::path::PathBuf::from(root)
                .join("UltraMusic")
                .join("audio-preferences.json")
        })
    }
    pub(super) fn load() -> Self {
        let value = (|| {
            let path = Self::path()?;
            let mut bytes = Vec::new();
            std::fs::File::open(path)
                .ok()?
                .take(4097)
                .read_to_end(&mut bytes)
                .ok()?;
            if bytes.len() > 4096 {
                return None;
            }
            let mut preferences: Self = serde_json::from_slice(&bytes).ok()?;
            if !preferences.volume.is_finite() {
                return None;
            }
            preferences.volume = preferences.volume.clamp(0.0, 1.0);
            Some(preferences)
        })();
        value.unwrap_or_default()
    }
    fn save(&self) -> std::io::Result<()> {
        let path =
            Self::path().ok_or_else(|| std::io::Error::other("LOCALAPPDATA indisponible"))?;
        let bytes = serde_json::to_vec(self)?;
        let temporary = path.with_extension("tmp");
        std::fs::write(&temporary, bytes)?;
        std::fs::rename(temporary, path)
    }
}
use windows::{
    core::{w, Interface, Result, BOOL, HSTRING},
    Win32::{
        Foundation::{CloseHandle, LPARAM, WPARAM},
        Media::Audio::{
            eRender, IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator,
            ISimpleAudioVolume, MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
        },
        System::{
            Com::{CoCreateInstance, CLSCTX_ALL},
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
                TH32CS_SNAPPROCESS,
            },
        },
        UI::WindowsAndMessaging::{PostMessageW, MB_ICONERROR, MB_OK},
    },
};

impl App {
    pub(super) fn attach_media(&self, web: &ICoreWebView2, generation: usize) -> Result<()> {
        let audio: ICoreWebView2_8 = web.cast()?;
        // SAFETY: this interface is owned by the UI STA; the callback captures only a number.
        unsafe {
            audio.SetIsMuted(self.muted.get())?;
            let mut token = 0;
            audio.add_IsDocumentPlayingAudioChanged(
                &IsDocumentPlayingAudioChangedEventHandler::create(Box::new(move |sender, _| {
                    if let (Some(app), Some(web)) = (current(generation), sender) {
                        let audio: ICoreWebView2_8 = web.cast()?;
                        let mut playing = BOOL(0);
                        audio.IsDocumentPlayingAudio(&mut playing)?;
                        // Stopping playback does not require enumerating Windows audio sessions.
                        if playing.as_bool() {
                            app.reset_volume_sessions();
                            let _ = app.apply_volume();
                        }
                    }
                    Ok(())
                })),
                &mut token,
            )?;
        }
        Ok(())
    }

    pub(super) fn play_pause(&self) {
        let Some(controller) = self.controller() else {
            return;
        };
        let generation = self.generation.get();
        // A user-triggered one-shot command, not a permanent injected script.
        let script = "(async()=>{const roots=[document],media=[];for(let i=0;i<roots.length;i++){const r=roots[i];media.push(...r.querySelectorAll('audio,video'));for(const n of r.querySelectorAll('*'))if(n.shadowRoot)roots.push(n.shadowRoot);for(const f of r.querySelectorAll('iframe'))try{if(f.contentDocument)roots.push(f.contentDocument)}catch{}}if(!media.length)return false;const playing=media.filter(x=>!x.paused&&!x.ended);if(playing.length){playing.forEach(x=>x.pause());return true}try{await(media.find(x=>x.readyState>=2)||media[0]).play();return true}catch{return false}})()";
        let parameters = HSTRING::from(serde_json::json!({
            "expression": script, "awaitPromise": true, "userGesture": true, "returnByValue": true
        }).to_string());
        // SAFETY: all COM access stays on the UI thread. Errors are displayed later.
        unsafe {
            let result = controller.CoreWebView2().and_then(|web| {
                web.CallDevToolsProtocolMethod(
                    w!("Runtime.evaluate"),
                    &parameters,
                    &CallDevToolsProtocolMethodCompletedHandler::create(Box::new(
                        move |hr, result| {
                            let succeeded = serde_json::from_str::<serde_json::Value>(&result)
                                .ok()
                                .is_some_and(|value| value["result"]["value"] == true);
                            if hr.is_err() || !succeeded {
                                if let Some(app) = current(generation) {
                                    let _ = PostMessageW(
                                        Some(app.hwnd),
                                        MEDIA_FAILED,
                                        WPARAM(generation),
                                        LPARAM(0),
                                    );
                                }
                            }
                            Ok(())
                        },
                    )),
                )
            });
            if result.is_err() {
                self.media_error();
            }
        }
    }

    pub(super) fn toggle_mute(&self) {
        let Some(controller) = self.controller() else {
            return;
        };
        // SAFETY: the WebView2 interface is accessed only on its owning STA.
        let result = unsafe {
            controller.CoreWebView2().and_then(|web| {
                let audio: ICoreWebView2_8 = web.cast()?;
                audio.SetIsMuted(!self.muted.get())
            })
        };
        if result.is_ok() {
            self.muted.set(!self.muted.get());
            self.audio_dirty.set(true);
            self.commit_audio_preferences();
        } else {
            self.media_error();
        }
    }

    pub(super) fn set_volume(&self, value: f32) {
        self.set_live_volume(value);
        self.finish_volume_gesture();
    }

    pub(super) fn begin_volume_gesture(&self) {
        if !self.volume_gesture.replace(true) {
            self.reset_volume_sessions();
            self.volume_failed.set(false);
        }
    }

    pub(super) fn set_live_volume(&self, value: f32) {
        let value = value.clamp(0.0, 1.0);
        if self.volume.get() == value {
            return;
        }
        self.volume.set(value);
        self.audio_dirty.set(true);
        if self.apply_volume().is_err() {
            self.volume_failed.set(true);
        }
    }

    pub(super) fn finish_volume_gesture(&self) {
        self.volume_gesture.set(false);
        self.reset_volume_sessions();
        self.commit_audio_preferences();
        if self.volume_failed.replace(false) {
            self.media_error();
        }
    }

    pub(super) fn reset_volume_sessions(&self) {
        self.volume_sessions_valid.set(false);
        let previous = self.volume_sessions.replace(Vec::new());
        drop(previous);
    }

    pub(super) fn apply_volume(&self) -> Result<()> {
        // Retry discovery once if a cached audio session becomes invalid during the gesture.
        for attempt in 0..2 {
            let sessions = if self.volume_gesture.get() && self.volume_sessions_valid.get() {
                self.volume_sessions.borrow().clone()
            } else {
                let sessions = self.find_volume_sessions()?;
                if self.volume_gesture.get() {
                    let previous = self.volume_sessions.replace(sessions.clone());
                    drop(previous);
                    self.volume_sessions_valid.set(true);
                }
                sessions
            };
            // SAFETY: cloned session interfaces stay on the UI STA; no borrow spans COM calls.
            let result = sessions.iter().try_for_each(|session| unsafe {
                session.SetMasterVolume(self.volume.get(), std::ptr::null())
            });
            match result {
                Ok(()) => return Ok(()),
                Err(_) if attempt == 0 && self.volume_gesture.get() => self.reset_volume_sessions(),
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn find_volume_sessions(&self) -> Result<Vec<ISimpleAudioVolume>> {
        let Some(controller) = self.controller() else {
            return Ok(Vec::new());
        };
        let mut matching = Vec::new();
        // SAFETY: COM is initialized on this thread. Each interface is released
        // locally; only sessions belonging to this WebView2 process tree are changed.
        unsafe {
            let web = controller.CoreWebView2()?;
            let mut browser = 0;
            web.BrowserProcessId(&mut browser)?;
            let processes = descendants(browser)?;
            let devices: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let endpoints = devices.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
            for endpoint in 0..endpoints.GetCount()? {
                let device = endpoints.Item(endpoint)?;
                let manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;
                let sessions = manager.GetSessionEnumerator()?;
                for i in 0..sessions.GetCount()? {
                    let session = sessions.GetSession(i)?;
                    let control: IAudioSessionControl2 = session.cast()?;
                    if processes.contains(&control.GetProcessId()?) {
                        let volume: ISimpleAudioVolume = session.cast()?;
                        matching.push(volume);
                    }
                }
            }
        }
        Ok(matching)
    }

    fn commit_audio_preferences(&self) {
        if !self.audio_dirty.get() {
            return;
        }
        let result = Preferences {
            volume: self.volume.get(),
            muted: self.muted.get(),
        }
        .save();
        match result {
            Ok(()) => self.audio_dirty.set(false),
            Err(error) => {
                dialog(Some(self.hwnd), &format!("Le réglage audio actuel est appliqué, mais n’a pas pu être mémorisé.\n\n{error}"), MB_OK | MB_ICONERROR);
            }
        }
    }

    pub(super) fn media_error(&self) {
        dialog(Some(self.hwnd), "La commande audio n’est pas disponible pour le lecteur actuel. Ouvrez le service et démarrez une piste, puis réessayez.", MB_OK | MB_ICONERROR);
    }
}

fn descendants(root: u32) -> Result<HashSet<u32>> {
    let mut ids = HashSet::from([root]);
    let mut entries = Vec::new();
    // SAFETY: the snapshot is owned here and the structure size is initialized.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                entries.push((entry.th32ProcessID, entry.th32ParentProcessID));
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }
    loop {
        let before = ids.len();
        for (pid, parent) in &entries {
            if ids.contains(parent) {
                ids.insert(*pid);
            }
        }
        if ids.len() == before {
            break;
        }
    }
    Ok(ids)
}
