use super::*;
use crate::profile::{Bootloader as B, Desktop, DisplayManager as Dm, Init, Profile, RepoPreset, Userland};

fn errors(p: &Profile) -> Vec<Issue> {
    validate(p).0.into_iter().filter(|i| i.severity == Severity::Error).collect()
}

fn warnings(p: &Profile) -> Vec<Issue> {
    validate(p).0.into_iter().filter(|i| i.severity == Severity::Warning).collect()
}

fn has_error(p: &Profile, field: &str) -> bool {
    errors(p).iter().any(|i| i.field == field)
}

#[test]
fn default_profile_is_valid() {
    let (issues, valid) = validate(&Profile::new_default());
    assert!(issues.is_empty(), "{issues:?}");
    assert!(valid.is_some());
}

#[test]
fn errors_withhold_valid_profile() {
    let mut p = Profile::new_default();
    p.bootloaders.clear();
    assert!(validate(&p).1.is_none());
}

#[test]
fn empty_bootloaders_error() {
    let mut p = Profile::new_default();
    p.bootloaders.clear();
    assert!(has_error(&p, "bootloaders"));
}

#[test]
fn duplicate_bootloader_error() {
    let mut p = Profile::new_default();
    p.bootloaders = vec![B::Grub, B::Grub];
    assert!(has_error(&p, "bootloaders"));
}

#[test]
fn refind_only_needs_bios_unless_uefi_only() {
    let mut p = Profile::new_default();
    p.bootloaders = vec![B::Refind];
    assert!(errors(&p).iter().any(|i| i.message.contains("BIOS")));
    p.uefi_only = true;
    assert!(errors(&p).is_empty(), "{:?}", errors(&p));
}

#[test]
fn refind_then_grub_splits_firmware_without_warning() {
    let mut p = Profile::new_default();
    p.bootloaders = vec![B::Refind, B::Grub];
    let fw = firmware_owners(&p);
    assert_eq!(fw.uefi, Some(B::Refind));
    assert_eq!(fw.bios, Some(B::Grub));
    assert!(warnings(&p).is_empty(), "{:?}", warnings(&p));
}

#[test]
fn limine_then_refind_warns_refind_unused() {
    let mut p = Profile::new_default();
    p.bootloaders = vec![B::Limine, B::Refind];
    assert!(errors(&p).is_empty());
    assert!(warnings(&p).iter().any(|i| i.field == "bootloaders" && i.message.contains("rEFInd")));
}

#[test]
fn uefi_only_drops_bios_owner() {
    let mut p = Profile::new_default();
    p.uefi_only = true;
    assert_eq!(firmware_owners(&p).bios, None);
    assert_eq!(firmware_owners(&p).uefi, Some(B::Grub));
}

#[test]
fn required_repos() {
    let mut p = Profile::new_default();
    p.init = Init::DinitChimera;
    assert!(has_error(&p, "repos"));
    p.repos.presets.push(RepoPreset::Voidlab);
    assert!(!has_error(&p, "repos"));

    p.init = Init::DinitNoid;
    assert_eq!(required_presets(&p), vec![RepoPreset::Noid]);
    p.init = Init::Runit;
    p.userland = Userland::Chimerautils;
    assert_eq!(required_presets(&p), vec![RepoPreset::Voidlab]);
    p.init = Init::Dynamod;
    assert_eq!(required_presets(&p), vec![RepoPreset::Voidlab]);
}

#[test]
fn display_manager_rules() {
    let mut p = Profile::new_default();
    p.display_manager = Dm::Lightdm;
    assert!(has_error(&p, "display_manager"), "DM without desktop");

    p.desktops = vec![Desktop::Xfce];
    p.display_manager = Dm::Gdm;
    assert!(has_error(&p, "display_manager"), "gdm without gnome");
    p.display_manager = Dm::CosmicGreeter;
    assert!(has_error(&p, "display_manager"), "cosmic-greeter without cosmic");

    p.display_manager = Dm::None;
    assert!(!has_error(&p, "display_manager"));
    assert!(warnings(&p).iter().any(|i| i.field == "display_manager"));

    p.desktops = vec![Desktop::Gnome];
    p.display_manager = Dm::Gdm;
    assert!(!has_error(&p, "display_manager"));
}

#[test]
fn live_identity_rules() {
    for bad in ["", "-x", "x-", "has space", &"a".repeat(64)] {
        let mut p = Profile::new_default();
        p.live.hostname = bad.to_string();
        assert!(has_error(&p, "live"), "hostname {bad:?}");
    }
    for bad in ["", "Root", "9x", "a b", "x;rm"] {
        let mut p = Profile::new_default();
        p.live.user = bad.to_string();
        assert!(has_error(&p, "live"), "user {bad:?}");
    }
    let mut p = Profile::new_default();
    p.live.locale = "en US".into();
    assert!(has_error(&p, "live"));
    let mut p = Profile::new_default();
    p.live.hostname = "my-box-01".into();
    p.live.user = "_gui-1".into();
    assert!(!has_error(&p, "live"));
}

#[test]
fn empty_name_error() {
    let mut p = Profile::new_default();
    p.name = "  ".into();
    assert!(has_error(&p, "name"));
}

#[test]
fn missing_overlay_and_hook_error() {
    let mut p = Profile::new_default();
    p.overlay_dir = Some("/nonexistent/vessel/overlay".into());
    p.post_rootfs_hook = Some("/nonexistent/vessel/hook.sh".into());
    assert!(has_error(&p, "overlay_dir"));
    assert!(has_error(&p, "post_rootfs_hook"));
    p.overlay_dir = Some(std::env::temp_dir());
    assert!(!has_error(&p, "overlay_dir"));
}

#[test]
fn non_x86_64_arch_error() {
    let mut p = Profile::new_default();
    p.arch = "aarch64".into();
    assert!(has_error(&p, "arch"));
}

#[test]
fn list_entries_must_be_single_tokens() {
    let mut p = Profile::new_default();
    p.packages.extra = vec!["fire fox".into()];
    assert!(has_error(&p, "packages"));
    let mut p = Profile::new_default();
    p.services.enable = vec!["".into()];
    assert!(has_error(&p, "services"));
    let mut p = Profile::new_default();
    p.repos.custom = vec!["https://x y".into()];
    assert!(has_error(&p, "repos"));
    let mut p = Profile::new_default();
    p.kernel = "linux lts".into();
    assert!(has_error(&p, "kernel"));
}

#[test]
fn names_starting_with_dash_are_rejected_as_option_injection() {
    let mut p = Profile::new_default();
    p.packages.extra = vec!["--rootdir=/".into()];
    assert!(has_error(&p, "packages"));
    let mut p = Profile::new_default();
    p.packages.exclude = vec!["-x".into()];
    assert!(has_error(&p, "packages"));
    let mut p = Profile::new_default();
    p.services.enable = vec!["-rf".into()];
    assert!(has_error(&p, "services"));
    let mut p = Profile::new_default();
    p.kernel = "-r/".into();
    assert!(has_error(&p, "kernel"));
    let mut p = Profile::new_default();
    p.repos.custom = vec!["-C/etc".into()];
    assert!(has_error(&p, "repos"));
}

#[test]
fn examples_validate() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    for name in ["dinit-chimera-base.toml", "dynamod-base.toml"] {
        let p = Profile::load(&dir.join(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
        let (issues, valid) = validate(&p);
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert!(errors.is_empty(), "{name}: {errors:?}");
        assert!(valid.is_some(), "{name}");
    }
}
