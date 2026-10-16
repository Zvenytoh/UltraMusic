//! Command-line configuration and pure navigation policy.

use url::Url;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Service {
    Youtube,
    Apple,
}

impl Service {
    pub fn home(self) -> &'static str {
        match self {
            Self::Youtube => "https://music.youtube.com/",
            Self::Apple => "https://music.apple.com/",
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
            service: Service::Youtube,
            keep_rendering: false,
        };
        for arg in args {
            match arg.as_str() {
                "--youtube" => result.service = Service::Youtube,
                "--apple" => result.service = Service::Apple,
                "--keep-rendering" => result.keep_rendering = true,
                _ => {
                    return Err(
                        "Usage : ultramusic.exe [--youtube | --apple] [--keep-rendering]".into(),
                    )
                }
            }
        }
        Ok(result)
    }
}
