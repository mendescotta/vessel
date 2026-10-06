use adw::prelude::*;

use super::{issues_group, list_group, page, path_row, switch_row, toggle_in};
use crate::profile::validate::required_presets;
use crate::profile::RepoPreset;
use crate::ui::state::{self, Shared};

pub fn build(s: &Shared) -> gtk::Widget {
    let page = page(
        "Repositories & packages",
        "Where packages come from, what goes on the ISO beyond the chosen components, and files to layer on top.",
    );
    page.add(&issues_group(
        s,
        &[
            "repos",
            "packages",
            "services",
            "overlay_dir",
            "post_rootfs_hook",
        ],
    ));

    let presets = adw::PreferencesGroup::builder()
        .title("Repositories")
        .description("The official Void repository is always used, after these.")
        .build();
    let fix = adw::ActionRow::builder()
        .title("The chosen init or userland needs more repositories")
        .build();
    let fix_btn = gtk::Button::builder()
        .label("Add required")
        .valign(gtk::Align::Center)
        .css_classes(["suggested-action"])
        .build();
    fix.add_suffix(&fix_btn);
    presets.add(&fix);
    {
        let s = s.clone();
        fix_btn.connect_clicked(move |_| {
            state::update(&s, |p| {
                for r in required_presets(p) {
                    toggle_in(&mut p.repos.presets, r, true);
                }
            })
        });
    }
    let refresh_fix = {
        let fix = fix.clone();
        move |p: &crate::profile::Profile| {
            fix.set_visible(
                required_presets(p)
                    .iter()
                    .any(|r| !p.repos.presets.contains(r)),
            )
        }
    };
    refresh_fix(&s.borrow().profile);
    state::on_change(s, move |p, _, _| refresh_fix(p));

    for r in RepoPreset::ALL.iter().copied() {
        presets.add(&switch_row(
            s,
            r.label(),
            None,
            move |p| p.repos.presets.contains(&r),
            move |p, on| toggle_in(&mut p.repos.presets, r, on),
        ));
    }
    page.add(&presets);

    page.add(&list_group(
        s,
        "Custom repositories",
        "URLs or local paths, used before the presets.",
        |p| &p.repos.custom,
        |p| &mut p.repos.custom,
        false,
    ));
    page.add(&list_group(
        s,
        "Extra packages",
        "Installed on top of what the chosen components pull in.",
        |p| &p.packages.extra,
        |p| &mut p.packages.extra,
        true,
    ));
    page.add(&list_group(
        s,
        "Excluded packages",
        "Never installed, even as dependencies (xbps ignorepkg).",
        |p| &p.packages.exclude,
        |p| &mut p.packages.exclude,
        false,
    ));
    page.add(&list_group(
        s,
        "Enable services",
        "In addition to the ones the chosen components enable.",
        |p| &p.services.enable,
        |p| &mut p.services.enable,
        false,
    ));
    page.add(&list_group(
        s,
        "Disable services",
        "Removed even if a component enables them.",
        |p| &p.services.disable,
        |p| &mut p.services.disable,
        false,
    ));

    let files = adw::PreferencesGroup::builder()
        .title("Customisation")
        .build();
    files.add(&path_row(
        s,
        "Overlay folder",
        true,
        |p| p.overlay_dir.clone(),
        |p, v| p.overlay_dir = v,
    ));
    files.add(&path_row(
        s,
        "Post-rootfs hook script",
        false,
        |p| p.post_rootfs_hook.clone(),
        |p, v| p.post_rootfs_hook = v,
    ));
    page.add(&files);

    page.upcast()
}
