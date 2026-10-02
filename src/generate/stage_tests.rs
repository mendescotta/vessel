use super::*;
use crate::profile::{Bootloader as B, Init, Profile};

#[test]
fn sh_quote_escapes_single_quotes_and_keeps_dollar_literal() {
    assert_eq!(shell::sh_quote("it's"), r"'it'\''s'");
    assert_eq!(shell::sh_quote("$HOME"), "'$HOME'");
    assert_eq!(shell::sh_quote(""), "''");
}

#[test]
fn sh_quote_round_trips_through_bash() {
    let nasty = "a'b\"c $(x) `y` \\ ;";
    let out = std::process::Command::new("bash")
        .arg("-c")
        .arg(format!("printf %s {}", shell::sh_quote(nasty)))
        .output()
        .unwrap();
    assert_eq!(String::from_utf8(out.stdout).unwrap(), nasty);
}

#[test]
fn sh_words_quotes_each() {
    assert_eq!(shell::sh_words(&["a".into(), "b c".into()]), "'a' 'b c'");
}

#[test]
fn iso_label_sanitises_and_truncates() {
    assert_eq!(shell::iso_label("my distro-1"), "MY_DISTRO_1");
    assert_eq!(shell::iso_label(&"x".repeat(40)).len(), 32);
    assert_eq!(shell::iso_label(""), "VOID_LIVE");
}

#[test]
fn cmdline_per_init() {
    let mut p = Profile::new_default();
    let runit = init::cmdline(&p, "LBL");
    assert!(runit.contains("root=live:CDLABEL=LBL"), "{runit}");
    assert!(runit.contains("live.user=anon"));
    assert!(runit.contains("vconsole.keymap=us"));
    assert!(!runit.contains("live.init_system"));

    p.init = Init::Dinit;
    assert!(init::cmdline(&p, "LBL").contains("live.init_system=dinit"));
}

#[test]
fn service_functions_use_init_layout() {
    assert!(init::service_functions(Init::Runit).contains("/etc/runit/runsvdir/default"));
    assert!(init::service_functions(Init::Dinit).contains("/etc/dinit.d/boot.d"));
    assert!(init::service_functions(Init::Dinit).contains("/usr/lib/dinit.d"));
}

#[test]
fn initramfs_and_squashfs_per_init() {
    assert!(init::initramfs_stage(Init::Runit).contains("dracut"));
    assert!(init::squashfs_stage(Init::Runit).contains("LiveOS/rootfs.img"));
}

#[test]
fn host_tools_per_role() {
    let grub_bios = bootloader::host_tools(B::Grub, true, false);
    assert!(grub_bios.contains(&("grub-mkimage", "grub")));
    assert!(!grub_bios.iter().any(|(c, _)| *c == "grub-mkstandalone"));
    let grub_uefi = bootloader::host_tools(B::Grub, false, true);
    assert!(grub_uefi.contains(&("grub-mkstandalone", "grub-x86_64-efi")));
    assert!(grub_uefi.contains(&("mcopy", "mtools")));
    assert!(bootloader::host_tools(B::Refind, false, true).contains(&("mkfs.vfat", "dosfstools")));
    assert!(bootloader::host_tools(B::Limine, true, true).contains(&("limine", "limine")));
}

#[test]
fn xorriso_args() {
    assert!(bootloader::xorriso_bios_args(B::Limine).contains("limine-bios-cd.bin"));
    assert!(bootloader::xorriso_bios_args(B::Grub).contains("eltorito.img"));
    assert!(bootloader::xorriso_uefi_args(B::Refind).contains("boot/refind/efiboot.img"));
    assert!(bootloader::xorriso_uefi_args(B::Limine).contains("limine-uefi-cd.bin"));
    assert!(bootloader::post_iso(B::Limine, true).contains("bios-install"));
    assert!(bootloader::post_iso(B::Limine, false).is_empty());
    assert!(bootloader::post_iso(B::Grub, true).is_empty());
}

#[test]
fn stage_text_only_for_owned_roles() {
    let limine_uefi = bootloader::stage(B::Limine, false, true);
    assert!(limine_uefi.contains("BOOTX64.EFI"));
    assert!(!limine_uefi.contains("limine-bios.sys"));
    let grub_bios = bootloader::stage(B::Grub, true, false);
    assert!(grub_bios.contains("i386-pc-eltorito"));
    assert!(!grub_bios.contains("grub-mkstandalone"));
    assert!(bootloader::stage(B::Refind, false, true).contains("refind_x64.efi"));
}
