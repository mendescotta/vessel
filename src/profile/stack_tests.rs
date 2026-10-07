use super::*;
use crate::profile::RepoPreset;

#[test]
fn runit_gnu_is_standard() {
    assert_eq!(support(Init::Runit, Userland::Gnu), Support::Standard);
}

#[test]
fn select_sets_stack_and_adds_repos() {
    let mut p = Profile::new_default();
    select(&mut p, Init::Dinit, Userland::Gnu);
    assert_eq!((p.init, p.userland), (Init::Dinit, Userland::Gnu));
    assert!(p.repos.presets.contains(&RepoPreset::Voidlab));
}

#[test]
fn old_init_ids_load_as_dinit() {
    for id in ["dinit", "dinit-chimera"] {
        #[derive(serde::Deserialize)]
        struct W {
            init: Init,
        }
        let w: W = toml::from_str(&format!("init = \"{id}\"")).unwrap();
        assert_eq!(w.init, Init::Dinit);
    }
}

#[test]
fn unknown_repo_preset_errors() {
    let toml = format!(
        "{}\n[repos]\npresets = [\"bogus\"]\n",
        include_str!("../../examples/dinit-base.toml")
            .split("[repos]")
            .next()
            .unwrap()
    );
    assert!(toml::from_str::<Profile>(&toml).is_err());
}
