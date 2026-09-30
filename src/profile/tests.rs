use super::*;

const MINIMAL: &str = r#"version = 1
name = "x"
init = "runit"
userland = "gnu"
bootloaders = ["grub"]
"#;

#[test]
fn minimal_profile_gets_defaults() {
    let p = Profile::from_toml(MINIMAL).unwrap();
    assert_eq!(p.display_manager, DisplayManager::None);
    assert_eq!(p.live.user, "anon");
    assert_eq!(p.live.hostname, "void-live");
    assert_eq!(p.arch, "x86_64");
    assert_eq!(p.kernel, "linux");
    assert!(p.desktops.is_empty());
}

#[test]
fn round_trips() {
    let mut p = Profile::new_default();
    p.init = Init::DinitChimera;
    p.desktops = vec![Desktop::Gnome];
    p.repos.presets = vec![RepoPreset::Voidlab];
    p.packages.extra = vec!["firefox".into()];
    assert_eq!(Profile::from_toml(&p.to_toml()).unwrap(), p);
}

#[test]
fn kebab_case_ids() {
    let p = Profile::from_toml(
        r#"version = 1
name = "x"
init = "dinit-chimera"
userland = "chimerautils"
bootloaders = ["limine", "refind"]
display_manager = "cosmic-greeter"
"#,
    )
    .unwrap();
    assert_eq!(p.init, Init::DinitChimera);
    assert_eq!(p.userland, Userland::Chimerautils);
    assert_eq!(p.bootloaders, vec![Bootloader::Limine, Bootloader::Refind]);
    assert_eq!(p.display_manager, DisplayManager::CosmicGreeter);
}

#[test]
fn unknown_enum_value_is_parse_error_naming_field() {
    let e = Profile::from_toml(&MINIMAL.replace("\"runit\"", "\"systemd\"")).unwrap_err();
    match e {
        ProfileError::Parse(m) => assert!(m.contains("init"), "{m}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn wrong_version_rejected() {
    let e = Profile::from_toml(&MINIMAL.replace("version = 1", "version = 2")).unwrap_err();
    assert!(matches!(e, ProfileError::Version(2)), "{e:?}");
}

#[test]
fn missing_version_is_parse_error() {
    let e = Profile::from_toml(&MINIMAL.replace("version = 1\n", "")).unwrap_err();
    assert!(matches!(e, ProfileError::Parse(_)), "{e:?}");
}

#[test]
fn load_resolves_relative_paths_against_profile_dir() {
    let dir = std::env::temp_dir().join(format!("vessel-profile-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("p.toml");
    std::fs::write(&path, format!("{MINIMAL}overlay_dir = \"ov\"\npost_rootfs_hook = \"/abs/hook.sh\"\n")).unwrap();
    let p = Profile::load(&path).unwrap();
    std::fs::remove_dir_all(&dir).ok();
    assert_eq!(p.overlay_dir, Some(dir.join("ov")));
    assert_eq!(p.post_rootfs_hook, Some(std::path::PathBuf::from("/abs/hook.sh")));
}

#[test]
fn bootloader_firmware_roles() {
    assert!(Bootloader::Grub.bios() && Bootloader::Grub.uefi());
    assert!(Bootloader::Limine.bios() && Bootloader::Limine.uefi());
    assert!(!Bootloader::Refind.bios() && Bootloader::Refind.uefi());
}
