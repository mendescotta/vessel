use super::*;
use crate::profile::validate::validate;
use crate::profile::{Init, Profile, RepoPreset};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

fn tmp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vessel-out-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn mode(p: &Path) -> u32 {
    std::fs::metadata(p).unwrap().permissions().mode() & 0o777
}

fn valid(p: &Profile) -> crate::profile::validate::ValidProfile {
    let (issues, v) = validate(p);
    v.unwrap_or_else(|| panic!("{issues:?}"))
}

#[test]
fn writes_script_profile_and_assets() {
    let dir = tmp("basic");
    let script = save_output_with(&valid(&Profile::new_default()), &dir, "/vl").unwrap();
    assert_eq!(script, dir.join("build.sh"));
    assert_eq!(mode(&script), 0o755);
    assert!(std::fs::read_to_string(&script).unwrap().starts_with("#!/bin/bash"));
    assert!(dir.join("profile.toml").is_file());
    assert!(dir.join("vmklive/module-setup.sh").is_file());
    assert_eq!(mode(&dir.join("vmklive/module-setup.sh")), 0o755);
    assert_eq!(mode(&dir.join("vmklive/COPYING")), 0o644);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn resave_removes_stale_assets_and_switching_init_drops_vmklive() {
    let dir = tmp("stale");
    let mut p = Profile::new_default();
    save_output_with(&valid(&p), &dir, "/vl").unwrap();
    std::fs::write(dir.join("vmklive/stale.sh"), "x").unwrap();
    save_output_with(&valid(&p), &dir, "/vl").unwrap();
    assert!(!dir.join("vmklive/stale.sh").exists());

    p.init = Init::Dynamod;
    p.repos.presets = vec![RepoPreset::Voidlab];
    save_output_with(&valid(&p), &dir, "/vl").unwrap();
    assert!(!dir.join("vmklive").exists());
    assert_eq!(mode(&dir.join("dynamod-initramfs.sh")), 0o755);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn copies_overlay_and_hook_and_rewrites_profile_paths() {
    let src = tmp("src");
    std::fs::create_dir_all(src.join("ov/etc/skel")).unwrap();
    std::fs::write(src.join("ov/etc/skel/.bashrc"), "hi").unwrap();
    std::fs::write(src.join("hook.sh"), "echo hook").unwrap();
    let mut p = Profile::new_default();
    p.overlay_dir = Some(src.join("ov"));
    p.post_rootfs_hook = Some(src.join("hook.sh"));

    let dir = tmp("dest");
    std::fs::create_dir_all(dir.join("overlay/old")).unwrap();
    save_output_with(&valid(&p), &dir, "/vl").unwrap();
    assert_eq!(std::fs::read_to_string(dir.join("overlay/etc/skel/.bashrc")).unwrap(), "hi");
    assert!(!dir.join("overlay/old").exists());
    assert_eq!(mode(&dir.join("hooks/post_rootfs.sh")), 0o755);

    let saved = Profile::load(&dir.join("profile.toml")).unwrap();
    assert_eq!(saved.overlay_dir, Some(dir.join("overlay")));
    assert_eq!(saved.post_rootfs_hook, Some(dir.join("hooks/post_rootfs.sh")));
    assert!(validate(&saved).1.is_some());
    std::fs::remove_dir_all(&dir).ok();
    std::fs::remove_dir_all(&src).ok();
}

#[test]
fn resave_with_overlay_already_in_folder_keeps_it() {
    let dir = tmp("self");
    std::fs::create_dir_all(dir.join("overlay")).unwrap();
    let mut p = Profile::new_default();
    p.overlay_dir = Some(dir.join("overlay"));
    std::fs::write(dir.join("overlay/keep"), "k").unwrap();
    save_output_with(&valid(&p), &dir, "/vl").unwrap();
    assert_eq!(std::fs::read_to_string(dir.join("overlay/keep")).unwrap(), "k");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn leaves_work_and_out_alone() {
    let dir = tmp("keep");
    std::fs::create_dir_all(dir.join("out")).unwrap();
    std::fs::write(dir.join("out/old.iso"), "iso").unwrap();
    save_output_with(&valid(&Profile::new_default()), &dir, "/vl").unwrap();
    assert!(dir.join("out/old.iso").exists());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn resave_does_not_change_a_script_that_is_running() {
    use std::io::Read;
    let dir = tmp("running");
    let script = save_output_with(&valid(&Profile::new_default()), &dir, "/vl").unwrap();
    let before = std::fs::read_to_string(&script).unwrap();
    let mut running = std::fs::File::open(&script).unwrap();

    let mut p = Profile::new_default();
    p.name = "renamed-while-building".into();
    save_output_with(&valid(&p), &dir, "/vl").unwrap();

    let mut seen = String::new();
    running.read_to_string(&mut seen).unwrap();
    assert!(seen == before, "the open build.sh changed under the running reader");
    assert!(std::fs::read_to_string(&script).unwrap().contains("renamed-while-building"));
    assert_eq!(mode(&script), 0o755);
}

#[test]
fn overlay_containing_the_output_folder_is_refused() {
    let src = tmp("overlay-parent");
    std::fs::write(src.join("motd"), "hi").unwrap();
    let dir = src.join("out");
    let mut p = Profile::new_default();
    p.overlay_dir = Some(src.clone());
    let err = save_output_with(&valid(&p), &dir, "/vl").unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput, "{err}");
    assert!(!dir.join("overlay").exists());
}
