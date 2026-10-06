# vessel

Compose a Void Linux live ISO from a TOML profile. vessel generates a standalone `build.sh`
(no vessel or void-mklive needed to run it); the GTK4 app is an editor, with a profile also
usable from the command line.

```
vessel --generate profile.toml --out <dir>
cd <dir> && sudo ./build.sh        # ISO lands in out/<name>-<YYYYMMDD>.iso
```

## Profile

```toml
version = 1
name = "dinit-base"
init = "dinit"                  # runit | dinit
userland = "chimerautils"       # gnu | chimerautils
bootloaders = ["grub"]          # grub | limine | refind (refind: UEFI only)
desktops = []                   # gnome cosmic cinnamon xfce budgie kde; empty = console
display_manager = "none"        # none lightdm sddm gdm cosmic-greeter

[repos]
presets = ["voidlab"]           # voidlab nonfree multilib
custom = []

[packages]
extra = []
exclude = []

[services]
enable = []
disable = []
```

Optional `overlay_dir` (copied over the rootfs) and `post_rootfs_hook` (run in the rootfs chroot) are
relative to the profile. Examples are in `examples/`. The `voidlab` preset is the
[voidlab](https://github.com/mendescotta/voidlab) binary repository; point it elsewhere with
`VESSEL_VOIDLAB_REPO` (a path or URL). `WORK` and `OUT` override the script's scratch and output folders.

## Build

```
cargo build --release && cargo test
```

x86_64 glibc only. No musl, disk images or package builds.

## References

- [void-mklive](https://github.com/void-linux/void-mklive): the Void live-ISO tooling the generated script follows; the `vmklive` dracut module in `assets/vmklive/` comes from it (license in `assets/vmklive/COPYING`).
- [dinit-chimera](https://github.com/chimera-linux/dinit-chimera) and [Chimera Linux](https://chimera-linux.org/): the dinit service suite and chimerautils userland the `dinit` and `chimerautils` options build on.
- [Void Linux](https://voidlinux.org/) and [xbps](https://github.com/void-linux/xbps).

Licensed under GPL-3.0-or-later.
