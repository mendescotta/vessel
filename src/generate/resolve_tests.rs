use super::*;
use crate::profile::{Desktop, DisplayManager as Dm, Init, Profile, RepoPreset};

fn count(v: &[String], s: &str) -> usize {
    v.iter().filter(|x| *x == s).count()
}

#[test]
fn repo_order_custom_presets_official() {
    let mut p = Profile::new_default();
    p.repos.custom = vec!["https://example.org/repo".into()];
    p.repos.presets = vec![RepoPreset::Voidlab, RepoPreset::Nonfree];
    assert_eq!(
        repos::repo_list_with(&p, "/vl/repo"),
        vec![
            "https://example.org/repo".to_string(),
            "/vl/repo".to_string(),
            format!("{}/nonfree", repos::OFFICIAL),
            repos::OFFICIAL.to_string(),
        ]
    );
}

#[test]
fn multilib_with_nonfree_adds_multilib_nonfree() {
    let mut p = Profile::new_default();
    p.repos.presets = vec![RepoPreset::Multilib, RepoPreset::Nonfree];
    let list = repos::repo_list_with(&p, "/vl/repo");
    assert!(list.contains(&format!("{}/multilib", repos::OFFICIAL)));
    assert!(list.contains(&format!("{}/multilib/nonfree", repos::OFFICIAL)));
}

#[test]
fn voidlab_preset_uses_given_path_and_dedups() {
    let mut p = Profile::new_default();
    p.repos.presets = vec![RepoPreset::Voidlab];
    p.repos.custom = vec!["/vl/repo".into()];
    assert_eq!(
        repos::repo_list_with(&p, "/vl/repo"),
        vec!["/vl/repo".to_string(), repos::OFFICIAL.to_string()]
    );
}

#[test]
fn voidlab_repo_has_no_default() {
    assert_eq!(repos::voidlab_repo_path_from(Some("/x".into())), "/x");
    assert_eq!(
        repos::voidlab_repo_path_from(None),
        "",
        "a remote repository must be chosen explicitly"
    );
}

#[test]
fn the_voidlab_preset_requires_a_repository() {
    let mut p = Profile::new_default();
    p.repos.presets = vec![RepoPreset::Voidlab];
    assert!(repos::require_voidlab_repo(&p, "")
        .unwrap_err()
        .contains("VESSEL_VOIDLAB_REPO"));
    assert!(repos::require_voidlab_repo(&p, "/vl/repo").is_ok());
    p.repos.presets.clear();
    assert!(
        repos::require_voidlab_repo(&p, "").is_ok(),
        "profiles without the preset need nothing"
    );
    assert_eq!(
        repos::repo_list_with(&p, ""),
        vec![repos::OFFICIAL.to_string()]
    );
}

#[test]
fn required_packages_dinit_gnome_lightdm() {
    let mut p = Profile::new_default();
    p.init = Init::Dinit;
    p.desktops = vec![Desktop::Gnome];
    p.display_manager = Dm::Lightdm;
    let pkgs = required_packages(&p);
    for want in [
        "linux",
        "base-system-dinit",
        "dinit-void",
        "dracut",
        "gnome",
        "lightdm",
        "lightdm-gtk3-greeter",
        "dbus",
    ] {
        assert_eq!(count(&pkgs, want), 1, "{want} in {pkgs:?}");
    }
    assert!(!pkgs.contains(&"base-system".to_string()));
}

#[test]
fn console_profile_has_no_graphical_base() {
    let pkgs = required_packages(&Profile::new_default());
    assert!(pkgs.contains(&"base-system".to_string()));
    assert!(!pkgs.contains(&"xorg-minimal".to_string()));
    assert!(!pkgs.contains(&"NetworkManager".to_string()));
}

#[test]
fn install_packages_appends_extra_once() {
    let mut p = Profile::new_default();
    p.packages.extra = vec!["firefox".into(), "linux".into()];
    let pkgs = install_packages(&p);
    assert_eq!(count(&pkgs, "linux"), 1);
    assert_eq!(pkgs.last().map(String::as_str), Some("firefox"));
}

#[test]
fn services_union_dedup_and_disable() {
    let mut p = Profile::new_default();
    p.desktops = vec![Desktop::Xfce, Desktop::Gnome];
    p.display_manager = Dm::Lightdm;
    p.services.enable = vec!["sshd".into(), "dbus".into()];
    p.services.disable = vec!["agetty-tty2".into()];
    let s = enabled_services(&p);
    assert_eq!(count(&s, "dbus"), 1);
    assert!(s.contains(&"lightdm".to_string()));
    assert!(s.contains(&"sshd".to_string()));
    assert!(s.contains(&"agetty-tty1".to_string()));
    assert!(!s.contains(&"agetty-tty2".to_string()));
}

#[test]
fn graphical_profiles_do_not_also_run_dhcpcd() {
    let mut p = Profile::new_default();
    assert!(
        enabled_services(&p).contains(&"dhcpcd".to_string()),
        "console profiles keep dhcpcd"
    );
    p.desktops = vec![Desktop::Xfce];
    let s = enabled_services(&p);
    assert!(s.contains(&"NetworkManager".to_string()));
    assert!(
        !s.contains(&"dhcpcd".to_string()),
        "NetworkManager already does DHCP: {s:?}"
    );
}

#[test]
fn ignored_packages_are_the_excludes() {
    let mut p = Profile::new_default();
    p.packages.exclude = vec!["nano".into()];
    assert_eq!(ignored_packages(&p), vec!["nano".to_string()]);
}

#[test]
fn validate_rejects_excluding_required_package() {
    let mut p = Profile::new_default();
    p.packages.exclude = vec!["base-system".into()];
    let (issues, valid) = crate::profile::validate::validate(&p);
    assert!(valid.is_none());
    assert!(issues
        .iter()
        .any(|i| i.field == "packages" && i.message.contains("base-system")));
}

#[test]
fn live_images_get_networking() {
    for init in [Init::Runit, Init::Dinit] {
        assert!(init::default_services(init).contains(&"dhcpcd"), "{init:?}");
    }
}

#[test]
fn live_cmdline_uses_current_overlay_option() {
    for init in [Init::Runit, Init::Dinit] {
        let mut p = Profile::new_default();
        p.init = init;
        let c = init::cmdline(&p, "L");
        assert!(c.contains(" rd.overlay=1 "), "{c}");
        assert!(!c.contains("rd.live.overlay"), "{c}");
    }
}
