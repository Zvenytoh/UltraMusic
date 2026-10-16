#![windows_subsystem = "windows"]
#![deny(unsafe_op_in_unsafe_fn)]
//! UltraMusic's executable entry point. Native state lives exclusively on the UI thread.

mod config;
mod native;

fn main() -> std::process::ExitCode {
    if let Err(error) = native::run() {
        native::show_error(&error);
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}
