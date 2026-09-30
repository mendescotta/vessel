#!/bin/bash
# dynamod-initramfs.sh <rootfs> <kernel version> <output initramfs>
#
# Builds a dynamod live initramfs: dynamod-init as rdinit, no busybox, no
# dracut. Ported from void-dynamod-iso's scripts/build-iso.sh (itself adapted
# from dynamod's tools/mkimage.sh). Written next to build.sh by vessel.
#
# dynamod-init mounts devtmpfs itself and only execs two helpers during live
# boot: /bin/modprobe (every module load) and blkid (LABEL= fallback when
# there are no udev by-label links). It finds the live media from the
# dynamod.media= / dynamod.squashfs= kernel parameters.
set -euo pipefail

ROOTFS="$1"
KVER="$2"
OUT="$3"

for tool in readelf cpio gzip modprobe depmod; do
	command -v "$tool" >/dev/null 2>&1 || { echo "!! dynamod-initramfs: missing host tool $tool" >&2; exit 1; }
done
[ -x "${ROOTFS}/sbin/dynamod-init" ] || { echo "!! dynamod-initramfs: no /sbin/dynamod-init in rootfs" >&2; exit 1; }
[ -x "${ROOTFS}/usr/bin/zstd" ] || { echo "!! dynamod-initramfs: no zstd in rootfs (needed for .ko.zst modules)" >&2; exit 1; }

INITRAMFS_DIR="$(mktemp -d)"
trap 'rm -rf "$INITRAMFS_DIR"' EXIT

mkdir -p "$INITRAMFS_DIR"/{sbin,bin,dev,proc,sys,newroot,run,usr/lib}
# usr-merged layout like the rootfs: /lib and /lib64 point at usr/lib, so the
# glibc loader path and /lib/modules both resolve.
ln -s usr/lib "${INITRAMFS_DIR}/lib"
ln -s usr/lib "${INITRAMFS_DIR}/lib64"
ln -s lib "${INITRAMFS_DIR}/usr/lib64"
cp "${ROOTFS}/sbin/dynamod-init" "${INITRAMFS_DIR}/sbin/dynamod-init"

needed_libs() { readelf -d "$1" | awk '/NEEDED/ { gsub(/[][]/, "", $5); print $5 }'; }
copy_lib_deps() {
	local lib
	for lib in $(needed_libs "$1"); do
		[ -e "${INITRAMFS_DIR}/usr/lib/${lib}" ] && continue
		cp -L "${ROOTFS}/usr/lib/${lib}" "${INITRAMFS_DIR}/usr/lib/${lib}"
		copy_lib_deps "${ROOTFS}/usr/lib/${lib}"
	done
}
copy_with_libs() { # <path in rootfs> <dest path in initramfs>
	install -Dm755 "${ROOTFS}$1" "${INITRAMFS_DIR}$2"
	copy_lib_deps "${ROOTFS}$1"
}
copy_with_libs /usr/bin/kmod /bin/kmod
ln -s kmod "${INITRAMFS_DIR}/bin/modprobe"
copy_with_libs /usr/bin/blkid /bin/blkid
cp -L "${ROOTFS}/usr/lib/ld-linux-x86-64.so.2" "${INITRAMFS_DIR}/usr/lib/"

# Void's kernel ships these as modules. The set is what dynamod-init's live
# code modprobes (media + fs), unioned with common real-hardware storage
# drivers. Each is resolved with modprobe -D one at a time: extra words after
# the first are module parameters, not more module names.
NEEDED_MODULES="scsi_mod ata_piix cdrom sr_mod virtio_scsi \
	loop squashfs isofs udf overlay \
	ahci nvme sd_mod virtio_blk virtio_pci usb_storage uas ehci_pci ehci_hcd xhci_pci xhci_hcd ext4"
MODULES_DST="${INITRAMFS_DIR}/lib/modules/${KVER}"
mkdir -p "$MODULES_DST"
MODULE_PATHS=""
for m in $NEEDED_MODULES; do
	out="$(modprobe -d "$ROOTFS" -S "$KVER" -D "$m" 2>&1)" || true
	case "$out" in
		*FATAL*) echo "!! dynamod-initramfs: cannot resolve kernel module $m: $out" >&2; exit 1 ;;
	esac
	# "builtin <name>" is already in vmlinuz; only "insmod <path>" names a file.
	MODULE_PATHS="${MODULE_PATHS}
$(printf '%s\n' "$out" | awk '$1 == "insmod" {print $2}')"
done
for src in $(printf '%s\n' "$MODULE_PATHS" | sed '/^$/d' | sort -u); do
	# host kmod may print <root>/lib/modules/... or <root>/usr/lib/modules/...
	rel="${src#*/modules/${KVER}/}"
	dst="${MODULES_DST}/${rel%.zst}"
	mkdir -p "$(dirname "$dst")"
	"${ROOTFS}/usr/bin/zstd" -qdf "$src" -o "$dst"
done
depmod -b "$INITRAMFS_DIR" "$KVER"

mkdir -p "$(dirname "$OUT")"
(cd "$INITRAMFS_DIR" && find . -print0 | cpio --null -o --format=newc 2>/dev/null | gzip -9) > "$OUT"
echo "==> dynamod initramfs: $OUT"
