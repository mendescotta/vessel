use super::*;
use std::path::PathBuf;

fn tmp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vessel-cli-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn generate_writes_script_for_an_example_profile() {
    let out = tmp("ok");
    let profile = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/dinit-base.toml");
    let script = generate(&profile, &out, "/vl").unwrap();
    assert_eq!(script, out.join("build.sh"));
    assert!(out.join("profile.toml").is_file());
    assert!(out.join("vmklive/module-setup.sh").is_file());
    std::fs::remove_dir_all(&out).ok();
}

#[test]
fn generate_reports_a_missing_profile() {
    let out = tmp("missing");
    let err = generate(&out.join("nope.toml"), &out, "/vl").unwrap_err();
    assert!(err.contains("nope.toml"), "{err}");
    std::fs::remove_dir_all(&out).ok();
}

#[test]
fn generate_refuses_an_invalid_profile() {
    let out = tmp("bad");
    let bad = out.join("bad.toml");
    std::fs::write(&bad, "version = 1\nname = \"x\"\ninit = \"dinit\"\nuserland = \"gnu\"\nbootloaders = []\n").unwrap();
    assert!(generate(&bad, &out.join("o"), "/vl").is_err());
    std::fs::remove_dir_all(&out).ok();
}
