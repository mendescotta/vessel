use std::collections::BTreeMap;
use std::process::Command;

use crate::generate::{enabled_services, repos, required_packages};
use crate::profile::validate::required_presets;
use crate::profile::{Bootloader, Desktop, DisplayManager, Init, Profile, RepoPreset, Userland};

pub trait HostProbe {
    fn read(&self, path: &str) -> Option<String>;
    fn list(&self, path: &str) -> Option<Vec<String>>;
    fn exists(&self, path: &str) -> bool;
    fn run(&self, cmd: &str, args: &[&str]) -> Option<String>;
}

pub struct RealHost;

impl HostProbe for RealHost {
    fn read(&self, path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    fn list(&self, path: &str) -> Option<Vec<String>> {
        let mut names: Vec<String> = std::fs::read_dir(path)
            .ok()?
            .filter_map(|e| e.ok()?.file_name().into_string().ok())
            .collect();
        names.sort();
        Some(names)
    }

    fn exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    fn run(&self, cmd: &str, args: &[&str]) -> Option<String> {
        let out = Command::new(cmd).args(args).output().ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

pub fn snapshot(host: &dyn HostProbe) -> (Profile, Vec<String>) {
    snapshot_with(host, &repos::voidlab_repo_path())
}

pub fn snapshot_with(host: &dyn HostProbe, voidlab_repo: &str) -> (Profile, Vec<String>) {
    let mut warnings = Vec::new();
    let mut p = Profile::new_default();

    let installed: Vec<String> = match host.run("xbps-query", &["-l"]) {
        Some(out) => out
            .lines()
            .filter_map(|l| l.split_whitespace().nth(1))
            .map(pkgname)
            .collect(),
        None => {
            warnings.push("couldn't list installed packages (xbps-query -l failed)".into());
            Vec::new()
        }
    };
    let has = |name: &str| installed.iter().any(|p| p == name);

    match host.read("/etc/hostname").map(|h| h.trim().to_string()) {
        Some(h) if !h.is_empty() => {
            p.live.hostname = h.clone();
            p.name = format!("{h}-snapshot");
        }
        _ => {
            warnings.push("couldn't read /etc/hostname; using the default name".into());
            p.name = "host-snapshot".into();
        }
    }

    let enabled: Vec<String> = if host.exists("/etc/dinit.d/boot.d") {
        p.init = Init::Dinit;
        host.list("/etc/dinit.d/boot.d").unwrap_or_default()
    } else {
        p.init = Init::Runit;
        host.list("/var/service").unwrap_or_else(|| {
            warnings.push("couldn't read /var/service; no services carried over".into());
            Vec::new()
        })
    };

    if has("chimerautils") {
        p.userland = Userland::Chimerautils;
    }

    const DESKTOP_METAS: &[(&str, Desktop)] = &[
        ("gnome", Desktop::Gnome),
        ("cosmic-desktop", Desktop::Cosmic),
        ("cinnamon", Desktop::Cinnamon),
        ("xfce4", Desktop::Xfce),
        ("budgie-desktop", Desktop::Budgie),
        ("kde-plasma", Desktop::Kde),
    ];
    p.desktops = DESKTOP_METAS
        .iter()
        .filter(|(pkg, _)| has(pkg))
        .map(|(_, d)| *d)
        .collect();

    const DMS: &[(&str, DisplayManager)] = &[
        ("lightdm", DisplayManager::Lightdm),
        ("sddm", DisplayManager::Sddm),
        ("gdm", DisplayManager::Gdm),
        ("cosmic-greeter", DisplayManager::CosmicGreeter),
    ];
    if let Some((_, dm)) = DMS.iter().find(|(svc, _)| enabled.iter().any(|e| e == svc)) {
        if p.desktops.is_empty() {
            warnings.push(format!(
                "{} is enabled but no known desktop is installed; leaving it out",
                dm.label()
            ));
        } else {
            p.display_manager = *dm;
        }
    }

    p.bootloaders = vec![if host.exists("/boot/grub") {
        Bootloader::Grub
    } else {
        Bootloader::Limine
    }];

    read_repos(host, voidlab_repo, &mut p, &mut warnings);
    for preset in required_presets(&p) {
        if !p.repos.presets.contains(&preset) {
            p.repos.presets.push(preset);
        }
    }

    match host.run("xbps-query", &["-m"]) {
        Some(out) => {
            let required = required_packages(&p);
            p.packages.extra = out
                .lines()
                .map(pkgname)
                .filter(|n| !n.is_empty() && !required.contains(n))
                .collect();
        }
        None => {
            warnings.push("couldn't list manually installed packages (xbps-query -m failed)".into())
        }
    }

    let defaults = enabled_services(&p);
    p.services.enable = enabled
        .into_iter()
        .filter(|s| !defaults.contains(s))
        .collect();

    (p, warnings)
}

fn read_repos(
    host: &dyn HostProbe,
    voidlab_repo: &str,
    p: &mut Profile,
    warnings: &mut Vec<String>,
) {
    let mut files: BTreeMap<String, String> = BTreeMap::new();
    let mut any_dir = false;
    for dir in ["/usr/share/xbps.d", "/etc/xbps.d"] {
        if let Some(names) = host.list(dir) {
            any_dir = true;
            for name in names.into_iter().filter(|n| n.ends_with(".conf")) {
                files.insert(name.clone(), format!("{dir}/{name}"));
            }
        }
    }
    if !any_dir {
        warnings.push(
            "couldn't read /etc/xbps.d or /usr/share/xbps.d; using the official repository only"
                .into(),
        );
    }
    for path in files.values() {
        let Some(text) = host.read(path) else {
            continue;
        };
        for line in text.lines().map(str::trim).filter(|l| !l.starts_with('#')) {
            let Some(url) = line.strip_prefix("repository=") else {
                continue;
            };
            let url = url.trim().trim_end_matches('/');
            match classify(url, voidlab_repo) {
                Some(Some(preset)) => {
                    if !p.repos.presets.contains(&preset) {
                        p.repos.presets.push(preset);
                    }
                }
                Some(None) => {}
                None => {
                    if !p.repos.custom.iter().any(|c| c == url) {
                        p.repos.custom.push(url.to_string());
                    }
                }
            }
        }
    }
}

fn classify(url: &str, voidlab_repo: &str) -> Option<Option<RepoPreset>> {
    if url == voidlab_repo.trim_end_matches('/') || url.ends_with("/Projects/voidlab/voidlab/repo")
    {
        return Some(Some(RepoPreset::Voidlab));
    }
    if url.ends_with("/current") {
        Some(None)
    } else if url.ends_with("/current/nonfree") {
        Some(Some(RepoPreset::Nonfree))
    } else if url.ends_with("/current/multilib") || url.ends_with("/current/multilib/nonfree") {
        Some(Some(RepoPreset::Multilib))
    } else {
        None
    }
}

pub fn pkgname(pkgver: &str) -> String {
    let pkgver = pkgver.trim();
    match pkgver.rsplit_once('-') {
        Some((name, ver)) if ver.contains('_') => name.to_string(),
        _ => pkgver.to_string(),
    }
}

#[cfg(test)]
#[path = "snapshot_tests.rs"]
mod tests;
