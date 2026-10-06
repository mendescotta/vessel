use crate::profile::Init;

pub fn base_packages(i: Init) -> &'static [&'static str] {
    match i {
        Init::Runit => &["base-system"],
        Init::Dinit => &["base-system-dinit", "dinit-void"],
    }
}

pub fn initramfs_packages(_: Init) -> &'static [&'static str] {
    &["dracut", "binutils", "xz", "device-mapper", "dhclient", "dracut-network", "openresolv"]
}

pub fn default_services(i: Init) -> &'static [&'static str] {
    match i {
        Init::Runit | Init::Dinit => &["agetty-tty1", "agetty-tty2", "dhcpcd"],
    }
}

pub fn cmdline(p: &crate::profile::Profile, label: &str) -> String {
    let l = &p.live;
    let mut c = format!(
        "root=live:CDLABEL={label} ro init=/sbin/init rd.luks=0 rd.md=0 rd.dm=0 \
         rd.overlay=1 loglevel=4 vconsole.unicode=1 \
         vconsole.keymap={} locale.LANG={} live.user={}",
        l.keymap, l.locale, l.user
    );
    if p.init == Init::Dinit {
        c.push_str(" live.init_system=dinit");
    }
    c
}

pub fn service_functions(i: Init) -> &'static str {
    match i {
        Init::Runit => r#"enable_service() {
	if [ -d "$ROOTFS/etc/sv/$1" ]; then
		ln -sfn "/etc/sv/$1" "$ROOTFS/etc/runit/runsvdir/default/$1"
	else
		warn "service $1 is not in the image (/etc/sv/$1); skipped"
	fi
}
disable_service() {
	rm -f "$ROOTFS/etc/runit/runsvdir/default/$1"
}
"#,
        Init::Dinit => r#"enable_service() {
	local d
	mkdir -p "$ROOTFS/etc/dinit.d/boot.d"
	for d in /etc/dinit.d /usr/lib/dinit.d; do
		if [ -e "$ROOTFS$d/$1" ]; then
			ln -sfn "$d/$1" "$ROOTFS/etc/dinit.d/boot.d/$1"
			return 0
		fi
	done
	warn "service $1 is not in the image (/etc/dinit.d or /usr/lib/dinit.d); skipped"
}
disable_service() {
	rm -f "$ROOTFS/etc/dinit.d/boot.d/$1"
}
"#,
    }
}

pub fn initramfs_stage(_: Init) -> &'static str {
    r#"info "Building live initramfs (dracut + vmklive)"
rm -rf "$ROOTFS/usr/lib/dracut/modules.d/01vmklive"
mkdir -p "$ROOTFS/usr/lib/dracut/modules.d/01vmklive"
cp "$HERE/vmklive/"*.sh "$HERE/vmklive/"*.rules "$ROOTFS/usr/lib/dracut/modules.d/01vmklive/"
chroot "$ROOTFS" env -i PATH=/usr/bin:/usr/sbin \
	dracut -N --zstd --add-drivers ahci --force-add vmklive --omit systemd /boot/initrd "$KVER"
mv "$ROOTFS/boot/initrd" "$ISODIR/boot/initrd"
rm -rf "$ROOTFS/usr/lib/dracut/modules.d/01vmklive"
"#
}

pub fn squashfs_stage(_: Init) -> &'static str {
    r#"info "Packing rootfs (dmsquash-live layout)"
rm -rf "$WORK/squash"
mkdir -p "$WORK/squash/LiveOS" "$ISODIR/LiveOS"
size_mb=$(du --apparent-size -sm "$ROOTFS" | cut -f1)
truncate -s "$((size_mb * 2 + 256))M" "$WORK/squash/LiveOS/rootfs.img"
mkfs.ext3 -q -F -m1 -d "$ROOTFS" "$WORK/squash/LiveOS/rootfs.img"
mksquashfs "$WORK/squash" "$ISODIR/LiveOS/squashfs.img" -comp zstd -noappend
rm -rf "$WORK/squash"
"#
}
