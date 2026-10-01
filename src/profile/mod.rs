mod enums;
pub mod validate;

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub use enums::{Bootloader, Desktop, DisplayManager, Init, RepoPreset, Userland};

pub const PROFILE_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub version: u32,
    pub name: String,
    #[serde(default = "default_arch")]
    pub arch: String,
    pub init: Init,
    pub userland: Userland,
    pub bootloaders: Vec<Bootloader>,
    #[serde(default)]
    pub uefi_only: bool,
    #[serde(default)]
    pub desktops: Vec<Desktop>,
    #[serde(default = "default_dm")]
    pub display_manager: DisplayManager,
    #[serde(default = "default_kernel")]
    pub kernel: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlay_dir: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_rootfs_hook: Option<PathBuf>,
    #[serde(default)]
    pub repos: Repos,
    #[serde(default)]
    pub packages: Packages,
    #[serde(default)]
    pub services: Services,
    #[serde(default)]
    pub live: Live,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Repos {
    #[serde(default)]
    pub presets: Vec<RepoPreset>,
    #[serde(default)]
    pub custom: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Packages {
    #[serde(default)]
    pub extra: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Services {
    #[serde(default)]
    pub enable: Vec<String>,
    #[serde(default)]
    pub disable: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Live {
    pub hostname: String,
    pub locale: String,
    pub keymap: String,
    pub timezone: String,
    pub user: String,
}

impl Default for Live {
    fn default() -> Self {
        Self {
            hostname: "void-live".into(),
            locale: "en_US.UTF-8".into(),
            keymap: "us".into(),
            timezone: "UTC".into(),
            user: "anon".into(),
        }
    }
}

fn default_arch() -> String {
    "x86_64".into()
}

fn default_dm() -> DisplayManager {
    DisplayManager::None
}

fn default_kernel() -> String {
    "linux".into()
}

#[derive(Debug)]
pub enum ProfileError {
    Io(std::io::Error),
    Parse(String),
    Version(u32),
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "could not read profile: {e}"),
            Self::Parse(m) => write!(f, "invalid profile: {m}"),
            Self::Version(v) => {
                write!(f, "unsupported profile version {v} (this vessel reads version {PROFILE_VERSION})")
            }
        }
    }
}

impl Profile {
    pub fn new_default() -> Self {
        Self {
            version: PROFILE_VERSION,
            name: "void-live".into(),
            arch: default_arch(),
            init: Init::Runit,
            userland: Userland::Gnu,
            bootloaders: vec![Bootloader::Grub],
            uefi_only: false,
            desktops: Vec::new(),
            display_manager: DisplayManager::None,
            kernel: default_kernel(),
            overlay_dir: None,
            post_rootfs_hook: None,
            repos: Repos::default(),
            packages: Packages::default(),
            services: Services::default(),
            live: Live::default(),
        }
    }

    pub fn from_toml(s: &str) -> Result<Self, ProfileError> {
        let value: toml::Table = s.parse().map_err(|e: toml::de::Error| ProfileError::Parse(e.to_string()))?;
        match value.get("version") {
            None => return Err(ProfileError::Parse("missing `version`".into())),
            Some(toml::Value::Integer(v)) if *v == i64::from(PROFILE_VERSION) => {}
            Some(toml::Value::Integer(v)) => return Err(ProfileError::Version(u32::try_from(*v).unwrap_or(u32::MAX))),
            Some(_) => return Err(ProfileError::Parse("`version` must be an integer".into())),
        }
        toml::from_str(s).map_err(|e| ProfileError::Parse(e.to_string()))
    }

    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).expect("profile always serialises")
    }

    pub fn load(path: &Path) -> Result<Self, ProfileError> {
        let text = std::fs::read_to_string(path).map_err(ProfileError::Io)?;
        let mut profile = Self::from_toml(&text)?;
        let base = path.parent().unwrap_or(Path::new("."));
        for p in [&mut profile.overlay_dir, &mut profile.post_rootfs_hook].into_iter().flatten() {
            if p.is_relative() {
                *p = base.join(&*p);
            }
        }
        Ok(profile)
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, self.to_toml())
    }
}

#[cfg(test)]
mod tests;
