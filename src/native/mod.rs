//! Windows host: STA lifetime, application state, and the blocking message loop.
//! Child modules never hold a RefCell borrow across a Win32/COM call.

mod blocker;
mod chrome;
mod commands;
mod frame;
mod instance;
mod media;
mod service;
mod settings;
mod settings_layout;
mod theme;
mod theme_draw;
mod tray;
mod ui;
mod updates;
mod webview;
mod window;

use window::window_proc;

use crate::config::{Config, Service};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller;
use windows::{
    core::{w, HSTRING, PCWSTR},
    Win32::{
        Foundation::{HINSTANCE, HWND},
        System::{
            Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED},
            LibraryLoader::GetModuleHandleW,
        },
        UI::{
            HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
            WindowsAndMessaging::{
                CreateWindowExW, DispatchMessageW, GetMessageW, LoadCursorW, LoadIconW,
                MessageBoxW, RegisterClassW, RegisterWindowMessageW, ShowWindow, TranslateMessage,
                CW_USEDEFAULT, HICON, IDC_ARROW, MB_ICONERROR, MB_OK, MESSAGEBOX_RESULT,
                MESSAGEBOX_STYLE, MSG, SW_SHOW, WM_APP, WNDCLASSW, WS_CLIPCHILDREN,
                WS_OVERLAPPEDWINDOW,
            },
        },
    },
};

const CLASS: PCWSTR = w!("UltraMusic.Native.1");
const TRAY: u32 = WM_APP + 1;
const RESTORE: u32 = WM_APP + 2;
const FAILED: u32 = WM_APP + 3;
const EXTERNAL: u32 = WM_APP + 4;
const UPDATE_READY: u32 = WM_APP + 5;
const MEDIA_FAILED: u32 = WM_APP + 6;
const HEADER_ACTION: u32 = WM_APP + 7;
const BLOCKER_FAILED: u32 = WM_APP + 8;

thread_local! { static APP: RefCell<Option<Rc<App>>> = const { RefCell::new(None) }; }
struct App {
    hwnd: HWND,
    icon: HICON,
    toolbar: Cell<HWND>,
    ui: RefCell<Option<ui::Resources>>,
    appearance: Cell<theme::Choice>,
    palette: RefCell<theme::Palette>,
    settings_font: RefCell<Option<ui::Font>>,
    settings_heading_font: RefCell<Option<ui::Font>>,
    settings: Cell<HWND>,
    settings_scroll: Cell<i32>,
    settings_content: Cell<i32>,
    settings_wheel: Cell<i32>,
    settings_positions: RefCell<Vec<(usize, windows::Win32::Foundation::RECT)>>,
    blocker_enabled: Cell<bool>,
    blocker_busy: Cell<bool>,
    blocker_state: Cell<blocker::State>,
    blocker: RefCell<
        Option<webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2BrowserExtension>,
    >,
    service: Cell<Service>,
    profile: HSTRING,
    controller: RefCell<Option<ICoreWebView2Controller>>,
    generation: Cell<usize>,
    hidden: Cell<bool>,
    keep_rendering: Cell<bool>,
    tray: Cell<bool>,
    taskbar_created: u32,
    external: RefCell<Option<String>>,
    failed: Cell<bool>,
    update_phase: Cell<updates::Phase>,
    volume_gesture: Cell<bool>,
    volume_sessions_valid: Cell<bool>,
    volume_sessions: RefCell<Vec<windows::Win32::Media::Audio::ISimpleAudioVolume>>,
    audio_dirty: Cell<bool>,
    volume_failed: Cell<bool>,
    volume: Cell<f32>,
    muted: Cell<bool>,
    update_manual: Cell<bool>,
    update_receiver: RefCell<Option<std::sync::mpsc::Receiver<updates::Event>>>,
}
fn app() -> Option<Rc<App>> {
    APP.with(|a| a.borrow().clone())
}
fn current(generation: usize) -> Option<Rc<App>> {
    app().filter(|a| a.generation.get() == generation)
}
fn dialog(hwnd: Option<HWND>, message: &str, flags: MESSAGEBOX_STYLE) -> MESSAGEBOX_RESULT {
    // SAFETY: strings live through the synchronous call; hwnd belongs to this UI thread.
    unsafe { MessageBoxW(hwnd, &HSTRING::from(message), w!("UltraMusic"), flags) }
}
pub(crate) fn show_error(message: &str) {
    dialog(None, message, MB_OK | MB_ICONERROR);
}
/// Balances a successful STA initialization after all COM owners are dropped.
struct Com;
impl Drop for Com {
    fn drop(&mut self) {
        // SAFETY: constructed only after CoInitializeEx succeeds on this same thread.
        unsafe {
            CoUninitialize();
        }
    }
}
pub(crate) fn run() -> std::result::Result<(), String> {
    let config = Config::parse(std::env::args().skip(1))?;
    // SAFETY: Win32 and COM are confined to this STA. The class strings are static,
    // all input structures outlive their calls, and COM owners are dropped before Com.
    unsafe {
        let Some(_instance) = instance::acquire()? else {
            return Ok(());
        };
        let base = std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA indisponible")?;
        let profile = std::path::PathBuf::from(base)
            .join("UltraMusic")
            .join("WebView2");
        std::fs::create_dir_all(&profile).map_err(|e| e.to_string())?;
        let audio_preferences = media::Preferences::load();
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .map_err(|e| e.to_string())?;
        let _com = Com;
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let instance = HINSTANCE(GetModuleHandleW(None).map_err(|e| e.to_string())?.0);
        // The named resource is emitted by build.rs. LoadIconW returns a shared icon
        // owned by the module, so it must not be passed to DestroyIcon.
        let icon = LoadIconW(Some(instance), w!("ULTRAMUSIC_ICON")).map_err(|e| e.to_string())?;
        let class = WNDCLASSW {
            hInstance: instance,
            lpszClassName: CLASS,
            lpfnWndProc: Some(window_proc),
            hCursor: LoadCursorW(None, IDC_ARROW).map_err(|e| e.to_string())?,
            hIcon: icon,
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 {
            return Err(windows::core::Error::from_thread().to_string());
        }
        let hwnd = CreateWindowExW(
            Default::default(),
            CLASS,
            w!("UltraMusic"),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1100,
            760,
            None,
            None,
            Some(instance),
            None,
        )
        .map_err(|e| e.to_string())?;
        let appearance = theme::Choice::load();
        let palette = theme::Palette::new(appearance).map_err(|e| e.to_string())?;
        let state = Rc::new(App {
            hwnd,
            icon,
            toolbar: Cell::new(HWND::default()),
            ui: RefCell::new(None),
            appearance: Cell::new(appearance),
            palette: RefCell::new(palette),
            settings_font: RefCell::new(None),
            settings_heading_font: RefCell::new(None),
            settings: Cell::new(HWND::default()),
            settings_scroll: Cell::new(0),
            settings_content: Cell::new(0),
            settings_wheel: Cell::new(0),
            settings_positions: RefCell::new(Vec::new()),
            blocker_enabled: Cell::new(blocker::load_preference()),
            blocker_busy: Cell::new(false),
            blocker_state: Cell::new(blocker::State::Loading),
            blocker: RefCell::new(None),
            keep_rendering: Cell::new(config.keep_rendering),
            service: Cell::new(config.service),
            profile: HSTRING::from(profile.as_os_str()),
            controller: RefCell::new(None),
            generation: Cell::new(0),
            hidden: Cell::new(false),
            tray: Cell::new(false),
            taskbar_created: RegisterWindowMessageW(w!("TaskbarCreated")),
            external: RefCell::new(None),
            failed: Cell::new(false),
            update_phase: Cell::new(updates::Phase::Idle),
            volume_gesture: Cell::new(false),
            volume_sessions_valid: Cell::new(false),
            volume_sessions: RefCell::new(Vec::new()),
            audio_dirty: Cell::new(false),
            volume_failed: Cell::new(false),
            volume: Cell::new(audio_preferences.volume),
            muted: Cell::new(audio_preferences.muted),
            update_manual: Cell::new(false),
            update_receiver: RefCell::new(None),
        });
        APP.with(|a| *a.borrow_mut() = Some(state.clone()));
        if let Err(error) = state.create_chrome() {
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd);
            APP.with(|a| a.borrow_mut().take());
            return Err(error.to_string());
        }
        theme::caption(hwnd, state.colors());
        state.configure_frame();
        state.add_tray();
        let _ = ShowWindow(hwnd, SW_SHOW);
        if state.start().is_err() {
            state.report_failure();
        }
        state.check_updates(false);
        let mut msg = MSG::default();
        let outcome = loop {
            match GetMessageW(&mut msg, None, 0, 0).0 {
                -1 => break Err(windows::core::Error::from_thread().to_string()),
                0 => break Ok(()),
                _ => {
                    if state.keyboard(&msg) {
                        continue;
                    }
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        };
        state.close_webview();
        state.remove_tray();
        APP.with(|a| a.borrow_mut().take());
        drop(state);

        outcome
    }
}
