//! Release discovery and verified downloads, independent of the music WebView.
mod http;
pub(crate) mod install;

use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const REPOSITORY: &str = "https://github.com/Zvenytoh/UltraMusic";
const MAX_BINARY: u64 = 128 * 1024 * 1024;

#[derive(Clone)]
pub(crate) struct Release {
    pub(crate) version: Version,
    url: String,
    size: u64,
    digest: String,
}
pub(crate) struct Download {
    pub(crate) path: PathBuf,
    pub(crate) digest: String,
}
#[derive(Deserialize)]
struct PublishedRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}
#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    size: u64,
    digest: Option<String>,
}

/// Checks published stable releases with a verified setup asset.
pub(crate) fn check() -> Result<Option<Release>, String> {
    let response =
        http::get("https://api.github.com/repos/Zvenytoh/UltraMusic/releases?per_page=30")?;
    let mut bytes = Vec::new();
    let mut chunk = [0; 16_384];
    loop {
        let read = response.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        if bytes.len() + read > 2 * 1024 * 1024 {
            return Err("Réponse de mise à jour trop volumineuse.".into());
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
    let releases: Vec<PublishedRelease> =
        serde_json::from_slice(&bytes).map_err(|_| "La réponse de GitHub n’est pas valide.")?;
    let installed = Version::parse(env!("CARGO_PKG_VERSION")).map_err(|e| e.to_string())?;
    let mut latest: Option<Release> = None;
    for release in releases.into_iter().filter(|r| !r.draft && !r.prerelease) {
        let tag = release.tag_name;
        let Ok(version) = Version::parse(tag.strip_prefix('v').unwrap_or(&tag)) else {
            continue;
        };
        if version <= installed || latest.as_ref().is_some_and(|r| r.version >= version) {
            continue;
        }
        for asset in release.assets {
            let expected = format!("{REPOSITORY}/releases/download/{tag}/UltraMusic-Setup.exe");
            let Some(digest) = asset
                .digest
                .and_then(|d| d.strip_prefix("sha256:").map(str::to_owned))
            else {
                continue;
            };
            if asset.name == "UltraMusic-Setup.exe"
                && asset.browser_download_url == expected
                && asset.size > 0
                && asset.size <= MAX_BINARY
                && valid_digest(&digest)
            {
                latest = Some(Release {
                    version: version.clone(),
                    url: expected,
                    size: asset.size,
                    digest,
                });
                break;
            }
        }
    }
    Ok(latest)
}

fn valid_digest(digest: &str) -> bool {
    digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit())
}

pub(crate) fn download(release: Release) -> Result<Download, String> {
    let root = std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA indisponible.")?;
    let directory = PathBuf::from(root)
        .join("UltraMusic")
        .join("Updates")
        .join(unique_id()?);
    fs::create_dir_all(&directory).map_err(|_| "Impossible de préparer le téléchargement.")?;
    let path = directory.join("UltraMusic-Setup.exe");
    let result = (|| {
        let response = http::get(&release.url)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();
        let mut size = 0_u64;
        let mut chunk = [0_u8; 65_536];
        loop {
            let read = response.read(&mut chunk)?;
            if read == 0 {
                break;
            }
            size += read as u64;
            if size > release.size || size > MAX_BINARY {
                return Err("La taille du téléchargement est invalide.".into());
            }
            file.write_all(&chunk[..read]).map_err(|e| e.to_string())?;
            hasher.update(&chunk[..read]);
        }
        file.sync_all().map_err(|e| e.to_string())?;
        let digest = format!("{:x}", hasher.finalize());
        if size != release.size || !digest.eq_ignore_ascii_case(&release.digest) {
            return Err(
                "L’intégrité de la mise à jour n’a pas pu être vérifiée. Installation annulée."
                    .into(),
            );
        }
        verify_executable(&path)?;
        Ok(Download {
            path: path.clone(),
            digest,
        })
    })();
    if result.is_err() {
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&directory);
    }
    result
}

pub(super) fn unique_id() -> Result<String, String> {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    Ok(format!("{}-{time}", std::process::id()))
}

pub(super) fn verify(path: &Path, expected: &str) -> Result<(), String> {
    if !valid_digest(expected) {
        return Err("Empreinte de mise à jour invalide.".into());
    }
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_BINARY {
        return Err("Fichier de mise à jour trop volumineux.".into());
    }
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 65_536];
    loop {
        let size = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if size == 0 {
            break;
        }
        hasher.update(&buffer[..size]);
    }
    if !format!("{:x}", hasher.finalize()).eq_ignore_ascii_case(expected) {
        return Err("Empreinte de mise à jour incorrecte.".into());
    }
    verify_executable(path)
}

fn verify_executable(path: &Path) -> Result<(), String> {
    let mut signature = [0; 2];
    fs::File::open(path)
        .and_then(|mut f| f.read_exact(&mut signature))
        .map_err(|e| e.to_string())?;
    if signature != *b"MZ" {
        return Err("Le téléchargement n’est pas un exécutable Windows.".into());
    }
    Ok(())
}
