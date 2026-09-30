pub mod desktop;
pub mod init;
pub mod repos;
pub mod userland;

use crate::profile::Profile;

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

/// Every package the chosen axes need, in install order, without duplicates.
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

/// What `xbps-install` is asked for: the required set plus the user's extras.
pub fn install_packages(p: &Profile) -> Vec<String> {
    let mut out = required_packages(p);
    out.extend(p.packages.extra.iter().cloned());
    dedup(out)
}

/// Written as `ignorepkg=` so xbps never installs them, even as dependencies.
pub fn ignored_packages(p: &Profile) -> Vec<String> {
    let mut out: Vec<String> = owned(init::ignore_packages(p.init)).collect();
    out.extend(p.packages.exclude.iter().cloned());
    dedup(out)
}

pub fn enabled_services(p: &Profile) -> Vec<String> {
    let mut out: Vec<String> = owned(init::default_services(p.init)).collect();
    if !p.desktops.is_empty() {
        out.extend(owned(desktop::GRAPHICAL_SERVICES));
    }
    out.extend(desktop::dm_service(p.display_manager).map(String::from));
    out.extend(p.services.enable.iter().cloned());
    dedup(out).into_iter().filter(|s| !p.services.disable.contains(s)).collect()
}

#[cfg(test)]
#[path = "resolve_tests.rs"]
mod resolve_tests;
