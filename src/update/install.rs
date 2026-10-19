//! Verified setup launcher. A temporary helper waits for the player to exit.
use super::{verify, Download};
use std::{
    ffi::{OsStr, OsString},
    fs,
    os::windows::process::CommandExt,
    path::{Component, Path, PathBuf, Prefix},
    process::Command,
};
use windows::Win32::{
    Foundation::{CloseHandle, WAIT_OBJECT_0},
    System::Threading::{OpenProcess, WaitForSingleObject, CREATE_NO_WINDOW, PROCESS_SYNCHRONIZE},
};

pub(crate) fn launch(download: Download, arguments: &[&str]) -> Result<(), String> {
    let target = std::env::current_exe()
        .and_then(fs::canonicalize)
        .map_err(|e| e.to_string())?;
    verify(&download.path, &download.digest)?;
    let directory = download
        .path
        .parent()
        .ok_or("Dossier de mise à jour absent.")?;
    // Reject unsupported paths while the original player is still available.
    installer_path(&target)?;
    installer_path(&fs::canonicalize(directory).map_err(|e| e.to_string())?)?;
    let helper = directory.join("installer-helper.exe");
    fs::copy(&target, &helper).map_err(|_| "Impossible de préparer la mise à jour.")?;
    if cfg!(target_env = "gnu") {
        let loader = target
            .parent()
            .ok_or("Dossier du lecteur absent.")?
            .join("WebView2Loader.dll");
        fs::copy(loader, directory.join("WebView2Loader.dll")).map_err(|e| e.to_string())?;
    }
    Command::new(helper)
        .arg("--apply-update")
        .arg(std::process::id().to_string())
        .arg(target)
        .arg(download.path)
        .arg(download.digest)
        .args(arguments)
        .creation_flags(CREATE_NO_WINDOW.0)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Runs before COM and the named instance mutex are initialized.
pub(crate) fn run_helper() -> Option<Result<(), String>> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(OsStr::new("--apply-update")) {
        return None;
    }
    Some(apply(args.collect()))
}

fn apply(args: Vec<OsString>) -> Result<(), String> {
    if args.len() < 4 {
        return Err("Arguments de mise à jour incomplets.".into());
    }
    let parent: u32 = args[0]
        .to_str()
        .ok_or("PID invalide.")?
        .parse()
        .map_err(|_| "PID invalide.")?;
    if parent == std::process::id() {
        return Err("Processus de remplacement invalide.".into());
    }
    let target = fs::canonicalize(PathBuf::from(&args[1])).map_err(|e| e.to_string())?;
    let source = fs::canonicalize(PathBuf::from(&args[2])).map_err(|e| e.to_string())?;
    let helper = std::env::current_exe()
        .and_then(fs::canonicalize)
        .map_err(|e| e.to_string())?;
    if !target.is_absolute()
        || target
            .extension()
            .is_none_or(|ext| !ext.eq_ignore_ascii_case("exe"))
        || source.parent() != helper.parent()
        || source.file_name() != Some(OsStr::new("UltraMusic-Setup.exe"))
        || target == source
        || target == helper
    {
        return Err("Chemins de mise à jour invalides.".into());
    }
    let digest = args[3].to_str().ok_or("Empreinte invalide.")?;
    let restart = &args[4..];
    if restart.iter().any(|arg| {
        !matches!(
            arg.to_str(),
            Some("--apple" | "--youtube" | "--youtube-video" | "--keep-rendering")
        )
    }) {
        return Err("Options de redémarrage invalides.".into());
    }
    verify(&source, digest)?;
    wait_for_parent(parent)?;
    let directory = target.parent().ok_or("Dossier de destination invalide.")?;
    let backup = source
        .parent()
        .ok_or("Dossier de sauvegarde absent.")?
        .join("UltraMusic.previous.exe");
    fs::copy(&target, &backup)
        .map_err(|_| "Impossible de sauvegarder le lecteur avant la mise à jour.")?;
    let mut destination = OsString::from("/DIR=");
    destination.push(installer_path(directory)?);
    let log = source
        .parent()
        .ok_or("Dossier du journal absent.")?
        .join("installer.log");
    let log = installer_path(&log)?;
    let mut log_argument = OsString::from("/LOG=");
    log_argument.push(&log);
    let result = Command::new(&source)
        // Keep Setup's progress and error dialogs visible; suppressing them made
        // installation failures indistinguishable from a silent rollback.
        .args(["/SILENT", "/NORESTART"])
        .arg(destination)
        .arg(log_argument)
        .creation_flags(CREATE_NO_WINDOW.0)
        .status();
    let failure = match &result {
        Ok(status) if !status.success() => {
            format!(
                "L’installeur a renvoyé le code {}.",
                status.code().unwrap_or(-1)
            )
        }
        Err(error) => format!("L’installeur n’a pas pu démarrer : {error}"),
        _ if !target.is_file() => "Le fichier du lecteur est absent après l’installation.".into(),
        _ => String::new(),
    };
    let succeeded = failure.is_empty();
    if !succeeded {
        fs::copy(&backup, &target)
            .map_err(|_| format!("Restaurez le lecteur depuis {}", backup.display()))?;
        let relaunched = Command::new(&target).args(restart).spawn();
        let restart_status = if relaunched.is_ok() {
            "L’ancien lecteur a été restauré et relancé."
        } else {
            "L’ancien lecteur a été restauré, mais son redémarrage a échoué."
        };
        return Err(format!(
            "{failure}\n\n{restart_status}\nJournal : {}",
            log.display()
        ));
    }
    if Command::new(&target).args(restart).spawn().is_err() {
        fs::copy(&backup, &target)
            .map_err(|_| format!("Restaurez le lecteur depuis {}", backup.display()))?;
        let _ = Command::new(&target).args(restart).spawn();
        return Err("Le redémarrage a échoué. L’ancien lecteur a été restauré.".into());
    }
    let _ = fs::remove_file(source);
    Ok(())
}

/// Inno Setup needs ordinary drive paths, not Rust's canonical `\\?\` form.
fn installer_path(path: &Path) -> Result<PathBuf, String> {
    let mut components = path.components();
    if let Some(Component::Prefix(prefix)) = components.next() {
        if let Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) = prefix.kind() {
            let mut plain = OsString::from(format!("{}:", char::from(drive)));
            plain.push(components.as_path());
            return Ok(PathBuf::from(plain));
        }
    }
    Err("La mise à jour nécessite une installation sur un disque local.".into())
}

fn wait_for_parent(pid: u32) -> Result<(), String> {
    // SAFETY: this owns the process handle, waits without polling, and closes it.
    unsafe {
        match OpenProcess(PROCESS_SYNCHRONIZE, false, pid) {
            Ok(handle) => {
                let result = WaitForSingleObject(handle, 30_000);
                let _ = CloseHandle(handle);
                if result != WAIT_OBJECT_0 {
                    return Err(
                        "Le lecteur ne s’est pas fermé à temps. Mise à jour annulée.".into(),
                    );
                }
            }
            Err(error) if error.code().0 == 0x80070057_u32 as i32 => {}
            Err(_) => return Err("Impossible d’attendre la fermeture du lecteur.".into()),
        }
    }
    Ok(())
}
