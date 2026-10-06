pub mod assets;
pub mod bootloader;
pub mod desktop;
pub mod init;
pub mod repos;
pub mod script;
pub mod shell;
pub mod userland;

use crate::profile::validate::ValidProfile;
use crate::profile::Profile;

pub use script::generate_with;

pub fn generate(v: &ValidProfile) -> String {
    generate_with(v, &repos::voidlab_repo_path())
}

pub(crate) fn dedup(items: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(items.len());
    for item in items {
        if !out.contains(&item) {
            out.push(item);
        }
    }
    out
}

fn owned<'a>(items: &'a [&'a str]) -> impl Iterator<Item = String> + 'a {
    items.iter().map(|s| s.to_string())
}

pub fn required_packages(p: &Profile) -> Vec<String> {
    let mut out = vec![p.kernel.clone()];
    out.extend(owned(init::base_packages(p.init)));
    out.extend(owned(init::initramfs_packages(p.init)));
    out.extend(owned(userland::packages(p.userland)));
    if !p.desktops.is_empty() {
        out.extend(owned(desktop::GRAPHICAL_BASE));
        for d in &p.desktops {
            out.extend(owned(desktop::desktop_packages(*d)));
        }
    }
    out.extend(owned(desktop::dm_packages(p.display_manager)));
    dedup(out)
}

pub fn install_packages(p: &Profile) -> Vec<String> {
    let mut out = required_packages(p);
    out.extend(p.packages.extra.iter().cloned());
    dedup(out)
}

pub fn ignored_packages(p: &Profile) -> Vec<String> {
    dedup(p.packages.exclude.clone())
}

pub fn enabled_services(p: &Profile) -> Vec<String> {
    let mut out: Vec<String> = owned(init::default_services(p.init)).collect();
    if !p.desktops.is_empty() {
        out.extend(owned(desktop::GRAPHICAL_SERVICES));
    }
    out.extend(desktop::dm_service(p.display_manager).map(String::from));
    out.extend(p.services.enable.iter().cloned());
    // NetworkManager (graphical profiles) runs its own DHCP; a second DHCP client on the
    // same interface makes it report "not connected" while the network works.
    let graphical = !p.desktops.is_empty();
    dedup(out)
        .into_iter()
        .filter(|s| !p.services.disable.contains(s) && !(graphical && s == "dhcpcd"))
        .collect()
}

#[cfg(test)]
#[path = "resolve_tests.rs"]
mod resolve_tests;

#[cfg(test)]
#[path = "stage_tests.rs"]
mod stage_tests;

#[cfg(test)]
#[path = "assets_tests.rs"]
mod assets_tests;

#[cfg(test)]
#[path = "script_tests.rs"]
mod script_tests;
