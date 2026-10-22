//! Bounded update retention. Never follow reparse points or delete an open operation.
use std::{
    fs::{self, File, OpenOptions},
    io,
    os::windows::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};
const SHARE_READ_WRITE: u32 = 0x3;
const SHARE_DELETE: u32 = 0x4;
const REPARSE_POINT: u32 = 0x400;
pub(super) struct Lease {
    _file: File,
}
impl Lease {
    pub(super) fn create(directory: &Path) -> io::Result<Self> {
        Ok(Self {
            _file: OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .share_mode(SHARE_READ_WRITE)
                .open(directory.join(".lease"))?,
        })
    }
    pub(super) fn open(directory: &Path) -> io::Result<Self> {
        Ok(Self {
            _file: OpenOptions::new()
                .read(true)
                .write(true)
                .share_mode(SHARE_READ_WRITE)
                .open(directory.join(".lease"))?,
        })
    }
}
fn ordinary(metadata: &fs::Metadata) -> bool {
    metadata.file_attributes() & REPARSE_POINT == 0
}
fn operation_name(name: &str) -> bool {
    name.split_once('-')
        .is_some_and(|(pid, time)| pid.parse::<u32>().is_ok() && time.parse::<u128>().is_ok())
}
/// Keep the two latest previous operations, plus every active/unknown operation.
pub(super) fn prune(root: &Path) -> io::Result<()> {
    if !ordinary(&fs::symlink_metadata(root)?) {
        return Ok(());
    }
    let root = fs::canonicalize(root)?;
    let mut directories = Vec::new();
    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.is_dir()
            && ordinary(&metadata)
            && entry.file_name().to_str().is_some_and(operation_name)
        {
            directories.push((metadata.modified()?, entry.path()));
        }
    }
    directories.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    for (_, directory) in directories.into_iter().skip(2) {
        let _ = remove_idle(&root, &directory);
    }
    Ok(())
}
fn remove_idle(root: &Path, directory: &Path) -> io::Result<()> {
    if directory.parent() != Some(root) || !ordinary(&fs::symlink_metadata(directory)?) {
        return Ok(());
    }
    let mut held = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if !metadata.is_file() || !ordinary(&metadata) {
            return Ok(());
        }
        if !matches!(
            entry.file_name().to_str(),
            Some(
                ".lease"
                    | "UltraMusic-Setup.exe"
                    | "installer-helper.exe"
                    | "UltraMusic.previous.exe"
                    | "WebView2Loader.dll"
                    | "installer.log"
            )
        ) {
            return Ok(());
        }
        // Deny other readers/writers, but allow deletion while our guard remains open.
        // Active download leases, helpers, Setup processes and open logs prevent this open.
        let guard = OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(SHARE_DELETE)
            .open(entry.path())?;
        held.push((entry.path(), guard));
    }
    for (file, _) in &held {
        fs::remove_file(file)?;
    }
    drop(held);
    fs::remove_dir(directory)
}
