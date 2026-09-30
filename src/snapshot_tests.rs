use super::*;
use crate::profile::{Bootloader, Desktop, DisplayManager, Init, RepoPreset, Userland};
use std::collections::HashMap;

#[derive(Default)]
struct FakeHost {
    files: HashMap<String, String>,
    dirs: HashMap<String, Vec<String>>,
    cmds: HashMap<String, String>,
}

impl FakeHost {
    fn file(mut self, p: &str, c: &str) -> Self {
        self.files.insert(p.into(), c.into());
        self
    }
    fn dir(mut self, p: &str, entries: &[&str]) -> Self {
        self.dirs.insert(p.into(), entries.iter().map(|s| s.to_string()).collect());
        self
    }
    fn cmd(mut self, key: &str, out: &str) -> Self {
        self.cmds.insert(key.into(), out.into());
        self
    }
}

impl HostProbe for FakeHost {
    fn read(&self, p: &str) -> Option<String> {
        self.files.get(p).cloned()
    }
    fn list(&self, p: &str) -> Option<Vec<String>> {
        self.dirs.get(p).cloned()
    }
    fn exists(&self, p: &str) -> bool {
        self.files.contains_key(p) || self.dirs.contains_key(p)
    }
    fn run(&self, cmd: &str, args: &[&str]) -> Option<String> {
        self.cmds.get(&format!("{cmd} {}", args.join(" "))).cloned()
    }
}

fn dinit_gnome_host() -> FakeHost {
    FakeHost::default()
        .file("/etc/hostname", "mybox\n")
        .dir("/etc/dinit.d/boot.d", &["agetty-tty1", "lightdm", "sshd", "dbus"])
        .dir("/etc/xbps.d", &["00-repository-main.conf", "20-voidlab.conf", "30-x.conf", "30-x.conf.old"])
        .file("/etc/xbps.d/00-repository-main.conf", "repository=https://repo-de.voidlinux.org/current\n")
        .file("/etc/xbps.d/20-voidlab.conf", "repository=/home/u/Projects/voidlab/voidlab/repo\n")
        .file("/etc/xbps.d/30-x.conf", "# c\nrepository=https://example.org/repo\n")
        .file("/etc/xbps.d/30-x.conf.old", "repository=https://old.example.org\n")
        .dir("/usr/share/xbps.d", &["00-repository-main.conf", "10-repository-nonfree.conf"])
        .file("/usr/share/xbps.d/00-repository-main.conf", "repository=https://repo-default.voidlinux.org/current\n")
        .file("/usr/share/xbps.d/10-repository-nonfree.conf", "repository=https://repo-default.voidlinux.org/current/nonfree\n")
        .cmd(
            "xbps-query -l",
            "ii base-system-dinit-0.1_1  x\nii chimerautils-15.1.1_3  x\nii gnome-51.0_1  x\nii lightdm-1.32.0_2  x\nii firefox-140.0_1  x\n",
        )
        .cmd("xbps-query -m", "base-system-dinit-0.1_1\ngnome-51.0_1\nlightdm-1.32.0_2\nfirefox-140.0_1\nlinux-6.18_1\n")
        .dir("/boot/grub", &[])
}

#[test]
fn seeds_dinit_chimera_gnome_host() {
    let (p, warnings) = snapshot_with(&dinit_gnome_host(), "/home/u/Projects/voidlab/voidlab/repo");
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(p.name, "mybox-snapshot");
    assert_eq!(p.live.hostname, "mybox");
    assert_eq!(p.init, Init::DinitChimera);
    assert_eq!(p.userland, Userland::Chimerautils);
    assert_eq!(p.desktops, vec![Desktop::Gnome]);
    assert_eq!(p.display_manager, DisplayManager::Lightdm);
    assert_eq!(p.bootloaders, vec![Bootloader::Grub]);
    assert!(p.repos.presets.contains(&RepoPreset::Voidlab));
    assert!(p.repos.presets.contains(&RepoPreset::Nonfree));
    assert_eq!(p.repos.custom, vec!["https://example.org/repo".to_string()]);
    // Already pulled in by the axes -> not repeated as extras.
    assert_eq!(p.packages.extra, vec!["firefox".to_string()]);
    assert_eq!(p.services.enable, vec!["sshd".to_string()]);
    assert!(crate::profile::validate::validate(&p).1.is_some());
}

#[test]
fn etc_overrides_same_named_usr_share_conf() {
    let host = FakeHost::default()
        .dir("/etc/xbps.d", &["10-repository-nonfree.conf"])
        .file("/etc/xbps.d/10-repository-nonfree.conf", "# disabled\n")
        .dir("/usr/share/xbps.d", &["10-repository-nonfree.conf"])
        .file("/usr/share/xbps.d/10-repository-nonfree.conf", "repository=https://repo-default.voidlinux.org/current/nonfree\n");
    let (p, _) = snapshot_with(&host, "/vl");
    assert!(!p.repos.presets.contains(&RepoPreset::Nonfree));
}

#[test]
fn detects_runit_and_dynamod() {
    let runit = FakeHost::default().dir("/var/service", &["sddm", "agetty-tty1"]).cmd("xbps-query -l", "ii kde-plasma-6.0_1 x\n");
    let (p, _) = snapshot_with(&runit, "/vl");
    assert_eq!(p.init, Init::Runit);
    assert_eq!(p.desktops, vec![Desktop::Kde]);
    assert_eq!(p.display_manager, DisplayManager::Sddm);

    let dynamod = FakeHost::default().dir("/etc/dynamod", &["services"]).dir("/etc/dynamod/services", &["dynamod-logind.toml", "sshd.toml"]);
    let (p, _) = snapshot_with(&dynamod, "/vl");
    assert_eq!(p.init, Init::Dynamod);
    assert!(p.repos.presets.contains(&RepoPreset::Voidlab), "required preset auto-added");
    assert!(p.services.enable.contains(&"sshd".to_string()));
    assert_eq!(p.bootloaders, vec![Bootloader::Limine]);
}

#[test]
fn noid_detected_by_base_package() {
    let host = FakeHost::default()
        .dir("/etc/dinit.d/boot.d", &[])
        .cmd("xbps-query -l", "ii noid-base-system-1_1 x\n");
    let (p, _) = snapshot_with(&host, "/vl");
    assert_eq!(p.init, Init::DinitNoid);
    assert!(p.repos.presets.contains(&RepoPreset::Noid));
}

#[test]
fn broken_host_still_produces_profile_with_warnings() {
    let (p, warnings) = snapshot_with(&FakeHost::default(), "/vl");
    assert_eq!(p.init, Init::Runit);
    assert!(!warnings.is_empty());
    assert!(crate::profile::validate::validate(&p).1.is_some());
}

#[test]
fn pkgname_strips_version() {
    assert_eq!(pkgname("xorg-fonts-7.7_1"), "xorg-fonts");
    assert_eq!(pkgname("nopkgver"), "nopkgver");
}

#[test]
#[ignore = "reads the real host; run with --ignored"]
fn real_host_smoke() {
    let (p, warnings) = snapshot(&RealHost);
    println!("{}\nwarnings: {warnings:?}\nextras: {}", p.to_toml(), p.packages.extra.len());
    let (issues, _) = crate::profile::validate::validate(&p);
    println!("issues: {issues:?}");
}
