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
userland = "gnu"
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
relative to the profile. Examples are in `examples/`. The `voidlab` preset needs `VESSEL_VOIDLAB_REPO`: the
`repo/` directory of a local [voidlab](https://github.com/mendescotta/voidlab) checkout, or a repository URL.
There is no default, because the build installs with `-y`, which imports a remote repository's signing key
without asking: only point it at a URL whose key you trust. `WORK` and `OUT` override the script's scratch and
output folders.

## Build

```
cargo build --release && cargo test
```

x86_64 glibc only. No musl, disk images or package builds.

## References

- [void-mklive](https://github.com/void-linux/void-mklive): the Void live-ISO tooling the generated script follows; the `vmklive` dracut module in `assets/vmklive/` comes from it (license in `assets/vmklive/COPYING`).
- [noid-linux/noid-mklive](https://github.com/noid-linux/noid-mklive): the dinit-aware fork of void-mklive; the dinit changes in `assets/vmklive/` are adapted from it.
- [dinit-chimera](https://github.com/chimera-linux/dinit-chimera) and [Chimera Linux](https://chimera-linux.org/): the dinit service suite the `dinit` option builds on.
- [Void Linux](https://voidlinux.org/) and [xbps](https://github.com/void-linux/xbps).

Licensed under GPL-3.0-or-later.
