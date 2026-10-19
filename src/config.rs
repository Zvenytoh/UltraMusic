//! Command-line configuration and pure navigation policy.

use url::Url;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Service {
    Youtube,
    Apple,
    YoutubeVideo,
}

impl Service {
    fn preference_path() -> Option<std::path::PathBuf> {
        std::env::var_os("LOCALAPPDATA").map(|root| {
            std::path::PathBuf::from(root)
                .join("UltraMusic")
                .join("service.txt")
        })
    }

    /// Missing or invalid preferences fall back to YouTube Music.
    fn load() -> Self {
        use std::io::Read;
        let saved = (|| {
            let mut value = String::new();
            std::fs::File::open(Self::preference_path()?)
                .ok()?
                .take(32)
                .read_to_string(&mut value)
                .ok()?;
            Some(value)
        })();
        match saved.as_deref() {
            Some("apple\n") => Self::Apple,
            Some("youtube-video\n") => Self::YoutubeVideo,
            _ => Self::Youtube,
        }
    }

    /// Commit a complete preference file before replacing the previous value.
    pub fn save(self) -> std::io::Result<()> {
        let path = Self::preference_path()
            .ok_or_else(|| std::io::Error::other("LOCALAPPDATA indisponible"))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let temporary = path.with_extension("tmp");
        std::fs::write(
            &temporary,
            match self {
                Self::Apple => "apple\n",
                Self::Youtube => "youtube\n",
                Self::YoutubeVideo => "youtube-video\n",
            },
        )?;
        std::fs::rename(temporary, path)
    }

    pub fn home(self) -> &'static str {
        match self {
            Self::Youtube => "https://music.youtube.com/",
            Self::Apple => "https://music.apple.com/",
            Self::YoutubeVideo => "https://www.youtube.com/",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Youtube => "YouTube Music",
            Self::Apple => "Apple Music",
            Self::YoutubeVideo => "YouTube",
        }
    }

    pub fn argument(self) -> &'static str {
        match self {
            Self::Youtube => "--youtube",
            Self::Apple => "--apple",
            Self::YoutubeVideo => "--youtube-video",
        }
    }

    pub fn allows(self, input: &str) -> bool {
        let Ok(url) = Url::parse(input) else {
            return false;
        };
        if url.scheme() != "https"
            || url.port_or_known_default() != Some(443)
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return false;
        }
        let hosts: &[&str] = match self {
            Self::Youtube => &[
                "music.youtube.com",
                "accounts.google.com",
                "consent.youtube.com",
                "consent.google.com",
                "accounts.youtube.com",
            ],
            Self::Apple => &[
                "music.apple.com",
                "idmsa.apple.com",
                "appleid.apple.com",
                "account.apple.com",
            ],
            Self::YoutubeVideo => &[
                "www.youtube.com",
                "youtube.com",
                "accounts.google.com",
                "consent.youtube.com",
                "consent.google.com",
                "accounts.youtube.com",
            ],
        };
        url.host_str().is_some_and(|host| hosts.contains(&host))
    }
}

pub fn external_url(input: &str) -> bool {
    Url::parse(input).is_ok_and(|u| {
        u.scheme() == "https"
            && u.host_str().is_some()
            && u.username().is_empty()
            && u.password().is_none()
    })
}

#[derive(Clone)]
pub struct Config {
    pub service: Service,
    pub keep_rendering: bool,
}
impl Config {
    pub fn parse(args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut result = Self {
            service: Service::load(),
            keep_rendering: false,
        };
        for arg in args {
            match arg.as_str() {
                "--youtube" => result.service = Service::Youtube,
                "--apple" => result.service = Service::Apple,
                "--youtube-video" => result.service = Service::YoutubeVideo,
                "--keep-rendering" => result.keep_rendering = true,
                _ => {
                    return Err(
                        "Usage : ultramusic.exe [--youtube | --apple | --youtube-video] [--keep-rendering]".into(),
                    )
                }
            }
        }
        Ok(result)
    }
}
