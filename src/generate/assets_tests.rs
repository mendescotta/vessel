use super::assets::assets_for;
use crate::profile::Init;

#[test]
fn dracut_inits_get_vmklive_with_license() {
    for init in [Init::Runit, Init::DinitChimera, Init::DinitNoid] {
        let paths: Vec<_> = assets_for(init).iter().map(|a| a.path).collect();
        assert!(paths.contains(&"vmklive/module-setup.sh"), "{paths:?}");
        assert!(paths.contains(&"vmklive/adduser.sh"));
        assert!(paths.contains(&"vmklive/COPYING"));
        assert!(!paths.contains(&"dynamod-initramfs.sh"));
    }
}

#[test]
fn dynamod_gets_only_its_initramfs_builder() {
    let a = assets_for(Init::Dynamod);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].path, "dynamod-initramfs.sh");
    assert!(a[0].executable);
}

#[test]
fn every_shell_asset_parses() {
    let dir = std::env::temp_dir().join(format!("vessel-assets-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for init in [Init::Runit, Init::Dynamod] {
        for a in assets_for(init).into_iter().filter(|a| a.path.ends_with(".sh")) {
            let f = dir.join(a.path.replace('/', "_"));
            std::fs::write(&f, a.contents).unwrap();
            let ok = std::process::Command::new("bash").arg("-n").arg(&f).status().unwrap().success();
            assert!(ok, "{} fails bash -n", a.path);
        }
    }
    std::fs::remove_dir_all(&dir).ok();
}
