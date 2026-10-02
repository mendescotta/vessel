use super::assets::assets_for;
use crate::profile::Init;

#[test]
fn dracut_inits_get_vmklive_with_license() {
    for init in [Init::Runit, Init::Dinit] {
        let paths: Vec<_> = assets_for(init).iter().map(|a| a.path).collect();
        assert!(paths.contains(&"vmklive/module-setup.sh"), "{paths:?}");
        assert!(paths.contains(&"vmklive/adduser.sh"));
        assert!(paths.contains(&"vmklive/COPYING"));
    }
}

#[test]
fn every_shell_asset_parses() {
    let dir = std::env::temp_dir().join(format!("vessel-assets-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for init in [Init::Runit] {
        for a in assets_for(init).into_iter().filter(|a| a.path.ends_with(".sh")) {
            let f = dir.join(a.path.replace('/', "_"));
            std::fs::write(&f, a.contents).unwrap();
            let ok = std::process::Command::new("bash").arg("-n").arg(&f).status().unwrap().success();
            assert!(ok, "{} fails bash -n", a.path);
        }
    }
    std::fs::remove_dir_all(&dir).ok();
}
