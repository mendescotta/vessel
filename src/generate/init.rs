use crate::profile::Init;

pub fn base_packages(i: Init) -> &'static [&'static str] {
    match i {
        Init::Runit => &["base-system"],
        Init::DinitChimera => &["base-system-dinit", "dinit-chimera", "dinit-void"],
        Init::DinitNoid => &["noid-base-system"],
        // kmod/util-linux/zstd/binutils feed dynamod-initramfs.sh (modprobe -D, blkid, .ko.zst).
        Init::Dynamod => &["base-system", "dynamod", "zstd", "kmod", "util-linux"],
    }
}

/// Packages xbps must never install for this init.
pub fn ignore_packages(i: Init) -> &'static [&'static str] {
    match i {
        Init::Dynamod => &["runit-void"],
        _ => &[],
    }
}

pub fn uses_dracut(i: Init) -> bool {
    !matches!(i, Init::Dynamod)
}

/// Needed in the rootfs to build the live initramfs (as void-mklive installs them).
pub fn initramfs_packages(i: Init) -> &'static [&'static str] {
    if uses_dracut(i) {
        &["dracut", "binutils", "xz", "device-mapper", "dhclient", "dracut-network", "openresolv"]
    } else {
        &[]
    }
}

pub fn default_services(i: Init) -> &'static [&'static str] {
    match i {
        Init::Runit | Init::DinitChimera | Init::DinitNoid => &["agetty-tty1", "agetty-tty2"],
        // dynamod ships its own enabled set in /etc/dynamod/services.
        Init::Dynamod => &[],
    }
}
