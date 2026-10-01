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
        // dhcpcd ships with base-system; a live image should come up online.
        Init::Runit | Init::DinitChimera | Init::DinitNoid => &["agetty-tty1", "agetty-tty2", "dhcpcd"],
        // dynamod ships its own enabled set in /etc/dynamod/services.
        Init::Dynamod => &[],
    }
}

/// Kernel command line for the live boot.
pub fn cmdline(p: &crate::profile::Profile, label: &str) -> String {
    let l = &p.live;
    match p.init {
        Init::Dynamod => format!(
            "rdinit=/sbin/dynamod-init dynamod.live=1 dynamod.media=LABEL={label} \
             dynamod.squashfs=/live/root.squashfs rootwait"
        ),
        Init::Runit | Init::DinitChimera | Init::DinitNoid => {
            let mut c = format!(
                "root=live:CDLABEL={label} ro init=/sbin/init rd.luks=0 rd.md=0 rd.dm=0 \
                 rd.overlay=1 loglevel=4 vconsole.unicode=1 \
                 vconsole.keymap={} locale.LANG={} live.user={}",
                l.keymap, l.locale, l.user
            );
            // The vendored vmklive module is noid-mklive's; it branches on this.
            if matches!(p.init, Init::DinitChimera | Init::DinitNoid) {
                c.push_str(" noid.init_system=dinit");
            }
            c
        }
    }
}

/// Bash definitions of `enable_service` / `disable_service` for this init's layout.
/// Missing services warn instead of failing the build.
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
        Init::DinitChimera | Init::DinitNoid => r#"enable_service() {
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
        Init::Dynamod => r#"enable_service() {
	local f="$ROOTFS/etc/dynamod/services/$1.toml"
	local avail="$ROOTFS/usr/share/dynamod/desktop-services/$1.toml"
	if [ -e "$f" ]; then
		return 0
	elif [ -e "$avail" ]; then
		cp "$avail" "$f"
	else
		warn "service $1 has no dynamod definition ($1.toml); skipped"
	fi
}
disable_service() {
	rm -f "$ROOTFS/etc/dynamod/services/$1.toml"
}
"#,
    }
}

/// Builds `$ISODIR/boot/initrd` from the rootfs for kernel `$KVER`.
pub fn initramfs_stage(i: Init) -> &'static str {
    if uses_dracut(i) {
        r#"info "Building live initramfs (dracut + vmklive)"
rm -rf "$ROOTFS/usr/lib/dracut/modules.d/01vmklive"
mkdir -p "$ROOTFS/usr/lib/dracut/modules.d/01vmklive"
cp "$HERE/vmklive/"*.sh "$HERE/vmklive/"*.rules "$ROOTFS/usr/lib/dracut/modules.d/01vmklive/"
chroot "$ROOTFS" env -i PATH=/usr/bin:/usr/sbin \
	dracut -N --zstd --add-drivers ahci --force-add vmklive --omit systemd /boot/initrd "$KVER"
mv "$ROOTFS/boot/initrd" "$ISODIR/boot/initrd"
rm -rf "$ROOTFS/usr/lib/dracut/modules.d/01vmklive"
"#
    } else {
        r#"info "Building dynamod initramfs"
bash "$HERE/dynamod-initramfs.sh" "$ROOTFS" "$KVER" "$ISODIR/boot/initrd"
"#
    }
}

/// Packs the rootfs where this init's live boot looks for it.
pub fn squashfs_stage(i: Init) -> &'static str {
    if uses_dracut(i) {
        // dmsquash-live wants LiveOS/rootfs.img inside the squashfs (as void-mklive does).
        // mkfs.ext3 -d fills the image without a loop mount.
        r#"info "Packing rootfs (dmsquash-live layout)"
rm -rf "$WORK/squash"
mkdir -p "$WORK/squash/LiveOS" "$ISODIR/LiveOS"
size_mb=$(du --apparent-size -sm "$ROOTFS" | cut -f1)
truncate -s "$((size_mb * 2 + 256))M" "$WORK/squash/LiveOS/rootfs.img"
mkfs.ext3 -q -F -m1 -d "$ROOTFS" "$WORK/squash/LiveOS/rootfs.img"
mksquashfs "$WORK/squash" "$ISODIR/LiveOS/squashfs.img" -comp zstd -noappend
rm -rf "$WORK/squash"
"#
    } else {
        r#"info "Packing rootfs (dynamod live layout)"
mkdir -p "$ISODIR/live"
mksquashfs "$ROOTFS" "$ISODIR/live/root.squashfs" -comp xz -noappend
"#
    }
}

/// dynamod-only rootfs setup, ported from void-dynamod-iso's build-rootfs.sh:
/// its mimic daemons need a permissive system bus policy, and there is no
/// vmklive to create the live user at boot.
pub fn dynamod_rootfs_extra() -> &'static str {
    r#"info "dynamod: permissive D-Bus system policy (Void's default blocks the mimic daemons)"
mkdir -p "$ROOTFS/etc/dbus-1"
cat > "$ROOTFS/etc/dbus-1/system.conf" <<'DBUSCONF'
<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
  "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <type>system</type>
  <listen>unix:path=/run/dbus/system_bus_socket</listen>
  <auth>EXTERNAL</auth>
  <policy context="default">
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
    <allow own="*"/>
    <allow send_type="method_call"/>
    <allow send_type="signal"/>
  </policy>
  <includedir>system.d</includedir>
  <includedir>/usr/share/dbus-1/system.d</includedir>
</busconfig>
DBUSCONF
info "dynamod: creating passwordless live user $LIVE_USER"
chroot "$ROOTFS" useradd -m -G wheel,audio,video,input -s /bin/bash "$LIVE_USER"
chroot "$ROOTFS" passwd -d "$LIVE_USER"
"#
}
