use adw::prelude::*;

use super::{issues_group, page, switch_row, toggle_in};
use crate::profile::validate::firmware_owners;
use crate::profile::{Bootloader, Profile};
use crate::ui::state::{self, Shared};

pub fn build(s: &Shared) -> gtk::Widget {
    let page = page(
        "Boot",
        "Pick one or more bootloaders. The first one in the list that supports a firmware path boots it, so rEFInd (UEFI only) needs GRUB or Limine for BIOS machines.",
    );
    page.add(&issues_group(s, &["bootloaders"]));

    let loaders = adw::PreferencesGroup::builder().title("Bootloaders").build();
    for b in Bootloader::ALL.iter().copied() {
        let roles = match (b.bios(), b.uefi()) {
            (true, true) => "BIOS and UEFI",
            _ => "UEFI only",
        };
        loaders.add(&switch_row(
            s,
            b.label(),
            Some(roles),
            move |p| p.bootloaders.contains(&b),
            move |p, on| toggle_in(&mut p.bootloaders, b, on),
        ));
    }
    page.add(&loaders);

    let fw = adw::PreferencesGroup::builder().title("Firmware").build();
    fw.add(&switch_row(
        s,
        "UEFI-only ISO",
        Some("Skip BIOS boot entirely"),
        |p| p.uefi_only,
        |p, on| p.uefi_only = on,
    ));
    let coverage = adw::ActionRow::builder().title("Coverage").build();
    coverage.set_subtitle(&coverage_text(&s.borrow().profile));
    {
        let coverage = coverage.clone();
        state::on_change(s, move |p, _, _| coverage.set_subtitle(&coverage_text(p)));
    }
    fw.add(&coverage);
    page.add(&fw);

    page.upcast()
}

fn coverage_text(p: &Profile) -> String {
    let fw = firmware_owners(p);
    let name = |b: Option<Bootloader>| b.map(|b| b.label()).unwrap_or("none");
    let bios = if p.uefi_only { "skipped".to_string() } else { name(fw.bios).to_string() };
    format!("BIOS: {bios} · UEFI: {}", name(fw.uefi))
}
