# vessel

A GTK4/libadwaita app that composes a Void Linux live ISO from a profile
(init system, userland, bootloaders, desktops, display manager, repositories,
packages) and generates a standalone `build.sh`. The script needs neither
vessel nor a void-mklive checkout to run. vessel can also run it for you
through pkexec, streaming the log.

Profiles can be built from scratch, or seeded from the running system
(Start page → snapshot) and edited from there.

## Axes

| Field | Values |
|---|---|
| `init` | `runit`, `dinit-chimera`, `dinit-noid`, `dynamod` |
| `userland` | `gnu`, `chimerautils` |
| `bootloaders` | any of `grub`, `limine`, `refind` (refind is UEFI only) |
| `desktops` | any of `gnome`, `cosmic`, `cinnamon`, `xfce`, `budgie`, `kde`; empty = console |
| `display_manager` | `none`, `lightdm`, `sddm`, `gdm`, `cosmic-greeter` |
| `repos.presets` | `voidlab`, `nonfree`, `multilib`, `noid` |

Some bootloader must cover UEFI, and BIOS too unless `uefi_only = true`.
Validation runs on every edit; the Review page lists errors and warnings and
previews the generated script.

## Profile format

```toml
version = 1
name = "dinit-chimera-base"
arch = "x86_64"
init = "dinit-chimera"
userland = "chimerautils"
bootloaders = ["grub"]
desktops = []
display_manager = "none"

[repos]
presets = ["voidlab"]
custom = []              # extra repository URLs or paths

[packages]
extra = []
exclude = []

[services]
enable = []
disable = []

[live]
hostname = "void-live"
user = "anon"
```

Everything except `version`, `name`, `init`, `userland` and `bootloaders` has
a default. Optional `overlay_dir` (copied over the rootfs) and
`post_rootfs_hook` (run chrooted in the rootfs) are relative to the profile
file. Working examples are in `examples/`.

## Output folder

Save writes to `~/vessel/<name>` (or a folder you pick):

- `profile.toml`: the profile, reloadable in vessel
- `build.sh`: the generated script
- `vmklive/`: the dracut live module (vendored from noid-mklive; license in `vmklive/COPYING`)
- `dynamod-initramfs.sh`, overlay and hook, when the profile uses them

Saving again over the same folder replaces these files cleanly.

## Running

```
cd ~/vessel/<name>
sudo ./build.sh            # ISO lands in out/<name>-<YYYYMMDD>.iso
```

`WORK` and `OUT` override the scratch (`work/`) and output (`out/`) folders.
The script only wipes a scratch folder it created itself (it leaves a
`.vessel-work` marker), so a non-empty `WORK` without the marker is refused.
The script checks for its host tools first and names the package for each
missing one (xbps, dracut, squashfs-tools, xorriso, grub/limine/refind,
mtools, dosfstools, e2fsprogs).

"Build now" in the app runs the same thing through
`pkexec /usr/libexec/vessel/run-build <folder>`. Install
`packaging/vessel-run-build` there and `packaging/org.voidlinux.vessel.policy`
into `/usr/share/polkit-1/actions/`.

## Building vessel

```
cargo build --release
cargo test
```

## Limitations

- x86_64 glibc only; the `arch` field exists for later.
- No musl, no disk images, no package builds (build packages with voidlab).
- No headless CLI; generate scripts from the app.
- Boot tested so far: the generated scripts are checked by golden tests and
  `bash -n`. ISO boots in QEMU are not recorded yet.
