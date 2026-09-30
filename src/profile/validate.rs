use std::collections::HashSet;

use super::{Bootloader, Desktop, DisplayManager, Init, Profile, RepoPreset, Userland};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub severity: Severity,
    pub field: &'static str,
    pub message: String,
}

/// A profile that passed `validate` with no errors. Only `validate` builds one,
/// which is what lets `generate` be infallible.
#[derive(Debug, Clone)]
pub struct ValidProfile(Profile);

impl ValidProfile {
    pub fn get(&self) -> &Profile {
        &self.0
    }
}

#[cfg(test)]
pub fn assume_valid(p: Profile) -> ValidProfile {
    ValidProfile(p)
}

/// Which bootloader boots the ISO on each firmware path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Firmware {
    pub bios: Option<Bootloader>,
    pub uefi: Option<Bootloader>,
}

/// The first bootloader in list order providing a path owns it.
pub fn firmware_owners(p: &Profile) -> Firmware {
    let bios = if p.uefi_only { None } else { p.bootloaders.iter().copied().find(|b| b.bios()) };
    let uefi = p.bootloaders.iter().copied().find(|b| b.uefi());
    Firmware { bios, uefi }
}

/// Repo presets the chosen init/userland packages come from.
pub fn required_presets(p: &Profile) -> Vec<RepoPreset> {
    let mut out = Vec::new();
    let mut need = |r| {
        if !out.contains(&r) {
            out.push(r);
        }
    };
    match p.init {
        Init::Runit => {}
        Init::DinitChimera | Init::Dynamod => need(RepoPreset::Voidlab),
        Init::DinitNoid => need(RepoPreset::Noid),
    }
    match p.userland {
        Userland::Gnu => {}
        Userland::Chimerautils => need(RepoPreset::Voidlab),
    }
    out
}

pub fn validate(p: &Profile) -> (Vec<Issue>, Option<ValidProfile>) {
    let mut issues = Vec::new();
    let mut error = |field, message: String| issues.push(Issue { severity: Severity::Error, field, message });

    if p.name.trim().is_empty() {
        error("name", "give the profile a name".into());
    }
    if p.arch != "x86_64" {
        error("arch", format!("architecture {:?} is not supported yet (only x86_64)", p.arch));
    }
    if !is_token(&p.kernel) {
        error("kernel", format!("kernel package {:?} must be a single package name", p.kernel));
    }

    // Bootloaders and firmware coverage.
    if p.bootloaders.is_empty() {
        error("bootloaders", "choose at least one bootloader".into());
    }
    let mut seen = HashSet::new();
    for b in &p.bootloaders {
        if !seen.insert(b) {
            error("bootloaders", format!("{} is listed twice", b.label()));
        }
    }
    let fw = firmware_owners(p);
    if !p.bootloaders.is_empty() {
        if fw.uefi.is_none() {
            error("bootloaders", "no bootloader covers UEFI".into());
        }
        if !p.uefi_only && fw.bios.is_none() {
            error(
                "bootloaders",
                "no bootloader covers BIOS (add GRUB or Limine, or make the ISO UEFI-only)".into(),
            );
        }
    }

    // Repositories.
    for preset in required_presets(p) {
        if !p.repos.presets.contains(&preset) {
            let who = if preset == RepoPreset::Noid || p.init != Init::Runit {
                p.init.label()
            } else {
                p.userland.label()
            };
            error("repos", format!("{who} needs the {} repository", preset.id()));
        }
    }
    for url in &p.repos.custom {
        if !is_token(url) {
            error("repos", format!("repository {url:?} must be a single URL or path"));
        }
    }

    // Desktops and display manager.
    let dm = p.display_manager;
    if dm != DisplayManager::None && p.desktops.is_empty() {
        error("display_manager", format!("{} needs at least one desktop", dm.label()));
    }
    if dm == DisplayManager::Gdm && !p.desktops.contains(&Desktop::Gnome) {
        error("display_manager", "GDM requires the GNOME desktop".into());
    }
    if dm == DisplayManager::CosmicGreeter && !p.desktops.contains(&Desktop::Cosmic) {
        error("display_manager", "COSMIC Greeter requires the COSMIC desktop".into());
    }

    // Packages and services.
    for (field, list) in [
        ("packages", &p.packages.extra),
        ("packages", &p.packages.exclude),
        ("services", &p.services.enable),
        ("services", &p.services.disable),
    ] {
        for item in list {
            if !is_token(item) {
                error(field, format!("{item:?} must be a single name without spaces"));
            }
        }
    }
    let required = crate::generate::required_packages(p);
    for pkg in &p.packages.exclude {
        if required.contains(pkg) {
            error("packages", format!("{pkg} can't be excluded: the chosen options need it"));
        }
    }

    // Live identity.
    if !valid_hostname(&p.live.hostname) {
        error("live", format!("hostname {:?} must be 1-63 letters, digits or '-', not starting or ending with '-'", p.live.hostname));
    }
    if !valid_user(&p.live.user) {
        error("live", format!("user name {:?} must match [a-z_][a-z0-9_-]*", p.live.user));
    }
    for (what, v) in [("locale", &p.live.locale), ("keymap", &p.live.keymap), ("timezone", &p.live.timezone)] {
        if !is_token(v) {
            error("live", format!("{what} {v:?} must be a single value without spaces"));
        }
    }

    // Files.
    if let Some(dir) = &p.overlay_dir {
        if !dir.is_dir() {
            error("overlay_dir", format!("overlay folder {} does not exist", dir.display()));
        }
    }
    if let Some(hook) = &p.post_rootfs_hook {
        if !hook.is_file() {
            error("post_rootfs_hook", format!("hook script {} does not exist", hook.display()));
        }
    }

    // Warnings.
    let mut warn = |field, message: String| issues.push(Issue { severity: Severity::Warning, field, message });
    for b in &p.bootloaders {
        if Some(*b) != fw.bios && Some(*b) != fw.uefi {
            let owner = fw.uefi.or(fw.bios).map(|o| o.label()).unwrap_or("another bootloader");
            warn("bootloaders", format!("{} is unused: {owner} already covers its firmware", b.label()));
        }
    }
    if !p.desktops.is_empty() && dm == DisplayManager::None {
        warn("display_manager", "desktops are selected but no display manager: you'll log in on the console".into());
    }

    let valid = issues.iter().all(|i| i.severity != Severity::Error).then(|| ValidProfile(p.clone()));
    (issues, valid)
}

fn is_token(s: &str) -> bool {
    !s.is_empty() && !s.chars().any(char::is_whitespace)
}

fn valid_hostname(s: &str) -> bool {
    (1..=63).contains(&s.len())
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
}

fn valid_user(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase() || c == '_')
        && s.len() <= 32
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

#[cfg(test)]
#[path = "validate_tests.rs"]
mod tests;
