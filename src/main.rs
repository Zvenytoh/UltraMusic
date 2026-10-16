#![windows_subsystem = "windows"]
#![deny(unsafe_op_in_unsafe_fn)]
//! UltraMusic's executable entry point. Native state lives exclusively on the UI thread.

mod config;
mod native;
mod update;

fn main() -> std::process::ExitCode {
    let result = update::install::run_helper().unwrap_or_else(native::run);
    if let Err(error) = result {
        native::show_error(&error);
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}
