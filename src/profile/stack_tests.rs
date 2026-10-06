use super::*;
use crate::profile::RepoPreset;

#[test]
fn runit_gnu_is_standard() {
    assert_eq!(support(Init::Runit, Userland::Gnu), Support::Standard);
}

#[test]
fn select_sets_stack_and_adds_repos() {
    let mut p = Profile::new_default();
    select(&mut p, Init::Dinit, Userland::Chimerautils);
    assert_eq!((p.init, p.userland), (Init::Dinit, Userland::Chimerautils));
    assert!(p.repos.presets.contains(&RepoPreset::Voidlab));
}

#[test]
fn old_init_ids_load_as_dinit() {
    for id in ["dinit", "dinit-chimera", "dinit-noid"] {
        #[derive(serde::Deserialize)]
        struct W {
            init: Init,
        }
        let w: W = toml::from_str(&format!("init = \"{id}\"")).unwrap();
        assert_eq!(w.init, Init::Dinit);
    }
}

#[test]
fn old_noid_preset_is_ignored_and_unknown_ones_error() {
    let ok = format!(
        "{}\n[repos]\npresets = [\"voidlab\", \"noid\"]\n",
        include_str!("../../examples/dinit-base.toml")
            .split("[repos]")
            .next()
            .unwrap()
    );
    let p: Profile = toml::from_str(&ok).unwrap();
    assert_eq!(p.repos.presets, vec![RepoPreset::Voidlab]);

    let bad = ok.replace("\"noid\"", "\"bogus\"");
    assert!(toml::from_str::<Profile>(&bad).is_err());
}
