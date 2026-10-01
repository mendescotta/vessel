use crate::profile::Bootloader;

pub fn host_tools(b: Bootloader, bios: bool, uefi: bool) -> Vec<(&'static str, &'static str)> {
    let mut t = Vec::new();
    match b {
        Bootloader::Grub => {
            if bios {
                t.push(("grub-mkimage", "grub"));
            }
            if uefi {
                t.push(("grub-mkstandalone", "grub-x86_64-efi"));
            }
        }
        Bootloader::Limine => t.push(("limine", "limine")),
        Bootloader::Refind => {}
    }
    if uefi && matches!(b, Bootloader::Grub | Bootloader::Refind) {
        t.push(("mkfs.vfat", "dosfstools"));
        t.push(("mcopy", "mtools"));
    }
    t
}

pub fn host_files(b: Bootloader, bios: bool, uefi: bool) -> Vec<(&'static str, &'static str)> {
    let mut f = Vec::new();
    match b {
        Bootloader::Grub if bios => f.push(("/usr/lib/grub/i386-pc/boot_hybrid.img", "grub")),
        Bootloader::Limine if bios => f.push(("/usr/share/limine/limine-bios-cd.bin", "limine")),
        _ => {}
    }
    match b {
        Bootloader::Limine if uefi => f.push(("/usr/share/limine/BOOTX64.EFI", "limine")),
        Bootloader::Refind if uefi => f.push(("/usr/share/refind/refind_x64.efi", "refind")),
        _ => {}
    }
    f
}

pub fn stage(b: Bootloader, bios: bool, uefi: bool) -> String {
    let mut s = String::new();
    match b {
        Bootloader::Grub => {
            s.push_str(&format!("info \"GRUB ({})\"\n", roles(bios, uefi)));
            s.push_str(GRUB_CFG);
            if bios {
                s.push_str(GRUB_BIOS);
            }
            if uefi {
                s.push_str(GRUB_UEFI);
            }
        }
        Bootloader::Limine => {
            s.push_str(&format!("info \"Limine ({})\"\n", roles(bios, uefi)));
            s.push_str("mkdir -p \"$ISODIR/boot/limine\"\n");
            if bios {
                s.push_str(
                    "cp /usr/share/limine/limine-bios-cd.bin /usr/share/limine/limine-bios.sys \"$ISODIR/boot/limine/\"\n",
                );
            }
            if uefi {
                s.push_str(LIMINE_UEFI);
            }
            s.push_str(LIMINE_CONF);
        }
        Bootloader::Refind => s.push_str(REFIND_UEFI),
    }
    s
}

fn roles(bios: bool, uefi: bool) -> &'static str {
    match (bios, uefi) {
        (true, true) => "BIOS + UEFI",
        (true, false) => "BIOS",
        _ => "UEFI",
    }
}

const GRUB_CFG: &str = r#"mkdir -p "$ISODIR/boot/grub"
{
	echo "set timeout=5"
	echo "set default=0"
	echo "search --no-floppy --label --set=root $LABEL"
	echo "menuentry \"$MENU_TITLE\" {"
	echo "	linux /boot/vmlinuz $CMDLINE"
	echo "	initrd /boot/initrd"
	echo "}"
	if [ -n "$CMDLINE_RAM" ]; then
		echo "menuentry \"$MENU_TITLE (copy to RAM)\" {"
		echo "	linux /boot/vmlinuz $CMDLINE_RAM"
		echo "	initrd /boot/initrd"
		echo "}"
	fi
} > "$ISODIR/boot/grub/grub.cfg"
"#;

const GRUB_BIOS: &str = r#"mkdir -p "$ISODIR/boot/grub/i386-pc"
cp /usr/lib/grub/i386-pc/*.mod /usr/lib/grub/i386-pc/*.lst "$ISODIR/boot/grub/i386-pc/"
grub-mkimage -O i386-pc-eltorito -p /boot/grub -o "$ISODIR/boot/grub/i386-pc/eltorito.img" \
	biosdisk iso9660 part_msdos part_gpt normal linux search search_label configfile echo
"#;

const GRUB_UEFI: &str = r#"rm -rf "$WORK/efi-grub"
mkdir -p "$WORK/efi-grub/EFI/BOOT"
grub-mkstandalone -O x86_64-efi -o "$WORK/efi-grub/EFI/BOOT/BOOTX64.EFI" \
	"boot/grub/grub.cfg=$ISODIR/boot/grub/grub.cfg"
make_efi_image "$ISODIR/boot/grub/efiboot.img" "$WORK/efi-grub"
"#;

const LIMINE_UEFI: &str = r#"cp /usr/share/limine/limine-uefi-cd.bin "$ISODIR/boot/limine/"
mkdir -p "$ISODIR/EFI/BOOT"
cp /usr/share/limine/BOOTX64.EFI "$ISODIR/EFI/BOOT/BOOTX64.EFI"
"#;

const LIMINE_CONF: &str = r#"{
	echo "timeout: 5"
	echo
	echo "/$MENU_TITLE"
	echo "    protocol: linux"
	echo "    path: boot():/boot/vmlinuz"
	echo "    cmdline: $CMDLINE"
	echo "    module_path: boot():/boot/initrd"
	if [ -n "$CMDLINE_RAM" ]; then
		echo
		echo "/$MENU_TITLE (copy to RAM)"
		echo "    protocol: linux"
		echo "    path: boot():/boot/vmlinuz"
		echo "    cmdline: $CMDLINE_RAM"
		echo "    module_path: boot():/boot/initrd"
	fi
} > "$ISODIR/boot/limine/limine.conf"
"#;

const REFIND_UEFI: &str = r#"info "rEFInd (UEFI)"
rm -rf "$WORK/efi-refind"
mkdir -p "$WORK/efi-refind/EFI/BOOT" "$WORK/efi-refind/boot" "$ISODIR/boot/refind"
cp /usr/share/refind/refind_x64.efi "$WORK/efi-refind/EFI/BOOT/BOOTX64.EFI"
cp "$ISODIR/boot/vmlinuz" "$ISODIR/boot/initrd" "$WORK/efi-refind/boot/"
{
	echo "timeout 5"
	echo "scanfor manual"
	echo "menuentry \"$MENU_TITLE\" {"
	echo "	loader /boot/vmlinuz"
	echo "	initrd /boot/initrd"
	echo "	options \"$CMDLINE\""
	echo "}"
} > "$WORK/efi-refind/EFI/BOOT/refind.conf"
make_efi_image "$ISODIR/boot/refind/efiboot.img" "$WORK/efi-refind"
"#;

pub fn xorriso_bios_args(b: Bootloader) -> &'static str {
    match b {
        Bootloader::Grub => {
            "-b boot/grub/i386-pc/eltorito.img -no-emul-boot -boot-load-size 4 -boot-info-table \
             --grub2-boot-info --grub2-mbr /usr/lib/grub/i386-pc/boot_hybrid.img"
        }
        Bootloader::Limine => "-b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table",
        Bootloader::Refind => unreachable!("rEFInd never owns BIOS boot"),
    }
}

pub fn xorriso_uefi_args(b: Bootloader) -> &'static str {
    match b {
        Bootloader::Grub => "-e boot/grub/efiboot.img",
        Bootloader::Limine => "-e boot/limine/limine-uefi-cd.bin",
        Bootloader::Refind => "-e boot/refind/efiboot.img",
    }
}

pub fn post_iso(b: Bootloader, bios: bool) -> &'static str {
    match b {
        Bootloader::Limine if bios => "limine bios-install \"$ISO\"\n",
        _ => "",
    }
}

pub const EFI_IMAGE_FN: &str = r#"make_efi_image() { # <image> <staging dir>
	local img="$1" dir="$2" size_kb
	size_kb=$(( $(du -sk "$dir" | cut -f1) * 11 / 10 + 2048 ))
	rm -f "$img"
	mkfs.vfat -C "$img" "$size_kb" >/dev/null
	mcopy -s -i "$img" "$dir"/* ::/
}
"#;
