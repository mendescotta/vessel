use super::*;
use crate::profile::validate::assume_valid;
use crate::profile::{Bootloader as B, Desktop, DisplayManager as Dm, Init, Profile, RepoPreset, Userland};
use std::path::PathBuf;

const VOIDLAB: &str = "/voidlab/repo";

fn gen(p: Profile) -> String {
    generate_with(&assume_valid(p), VOIDLAB)
}

fn golden_profiles() -> Vec<(&'static str, Profile)> {
    let runit = Profile::new_default();

    let mut chimera = Profile::new_default();
    chimera.name = "void-gnome-dinit".into();
    chimera.init = Init::DinitChimera;
    chimera.userland = Userland::Chimerautils;
    chimera.bootloaders = vec![B::Limine];
    chimera.desktops = vec![Desktop::Gnome];
    chimera.display_manager = Dm::Lightdm;
    chimera.repos.presets = vec![RepoPreset::Voidlab];
    chimera.packages.extra = vec!["firefox".into()];
    chimera.packages.exclude = vec!["nano".into()];

    let mut noid = Profile::new_default();
    noid.name = "noid-xfce".into();
    noid.init = Init::DinitNoid;
    noid.desktops = vec![Desktop::Xfce];
    noid.display_manager = Dm::Lightdm;
    noid.repos.presets = vec![RepoPreset::Noid, RepoPreset::Nonfree];
    noid.services.enable = vec!["sshd".into()];

    let mut dynamod = Profile::new_default();
    dynamod.name = "dynamod-cosmic".into();
    dynamod.init = Init::Dynamod;
    dynamod.bootloaders = vec![B::Refind, B::Limine];
    dynamod.desktops = vec![Desktop::Cosmic];
    dynamod.display_manager = Dm::CosmicGreeter;
    dynamod.repos.presets = vec![RepoPreset::Voidlab];
    dynamod.overlay_dir = Some("/somewhere/overlay".into());
    dynamod.post_rootfs_hook = Some("/somewhere/hook.sh".into());

    vec![
        ("runit-grub-console", runit),
        ("dinit-chimera-limine-gnome", chimera),
        ("dinit-noid-grub-xfce", noid),
        ("dynamod-refind-limine-cosmic", dynamod),
    ]
}

fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden").join(format!("{name}.sh"))
}

#[test]
fn golden_matrix() {
    for (name, p) in golden_profiles() {
        let script = gen(p);
        let path = golden_path(name);
        if std::env::var_os("VESSEL_BLESS").is_some() {
            std::fs::write(&path, &script).unwrap();
            continue;
        }
        let want = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing golden {}; run with VESSEL_BLESS=1", path.display()));
        assert!(script == want, "{name} differs from golden; rerun with VESSEL_BLESS=1 and review the diff");
    }
}

fn bash_n(script: &str) -> Result<(), String> {
    let f = std::env::temp_dir().join(format!("vessel-gen-{}-{}.sh", std::process::id(), script.len()));
    std::fs::write(&f, script).unwrap();
    let out = std::process::Command::new("bash").arg("-n").arg(&f).output().unwrap();
    let sc = std::process::Command::new("shellcheck").args(["-s", "bash", "-S", "error"]).arg(&f).output();
    std::fs::remove_file(&f).ok();
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into());
    }
    if let Ok(sc) = sc {
        if !sc.status.success() {
            return Err(String::from_utf8_lossy(&sc.stdout).into());
        }
    }
    Ok(())
}

#[test]
fn every_generated_script_parses() {
    for (name, p) in golden_profiles() {
        if let Err(e) = bash_n(&gen(p)) {
            panic!("{name}: {e}");
        }
    }
}

#[test]
fn hostile_name_is_quoted() {
    let mut p = Profile::new_default();
    p.name = "it's $HOME".into();
    let s = gen(p);
    assert!(s.contains(r"ISO_NAME='it'\''s $HOME'"), "{}", &s[..600]);
    bash_n(&s).unwrap();
}

#[test]
fn wipe_happens_after_unmount() {
    let s = gen(Profile::new_default());
    let rootfs_stage = s.find("# --- 3.").unwrap();
    let unmount = rootfs_stage + s[rootfs_stage..].find("umount_chroot\n").unwrap();
    let wipe = s.find("rm -rf \"$WORK\"").unwrap();
    assert!(unmount < wipe);
}

#[test]
fn exclude_and_init_ignores_become_ignorepkg() {
    let mut p = Profile::new_default();
    p.init = Init::Dynamod;
    p.repos.presets = vec![RepoPreset::Voidlab];
    p.packages.exclude = vec!["nano".into()];
    let s = gen(p);
    assert!(s.contains("ignorepkg=runit-void"));
    assert!(s.contains("ignorepkg=nano"));
}

#[test]
fn no_ignore_file_when_nothing_ignored() {
    assert!(!gen(Profile::new_default()).contains("00-vessel.conf"));
}

#[test]
fn dynamod_install_uses_force_flag_and_its_initramfs() {
    let mut p = Profile::new_default();
    p.init = Init::Dynamod;
    p.repos.presets = vec![RepoPreset::Voidlab];
    let s = gen(p);
    assert!(s.contains("xbps-install -S -y -I"));
    assert!(s.contains("dynamod-initramfs.sh"));
    assert!(!s.contains("dracut -N"));
    assert!(s.contains("CMDLINE_RAM=''"));
}

#[test]
fn refind_uefi_grub_bios_has_both_eltorito_entries() {
    let mut p = Profile::new_default();
    p.bootloaders = vec![B::Refind, B::Grub];
    let s = gen(p);
    let x = &s[s.find("xorriso -as mkisofs").unwrap()..];
    let bios = x.find("-b boot/grub/i386-pc/eltorito.img").unwrap();
    let alt = x.find("-eltorito-alt-boot").unwrap();
    let uefi = x.find("-e boot/refind/efiboot.img").unwrap();
    assert!(bios < alt && alt < uefi);
    // rEFInd copies the kernel into its image, so it must run after the kernel is staged.
    assert!(s.find("rEFInd (UEFI)").unwrap() > s.find("\"$ISODIR/boot/vmlinuz\"").unwrap());
}

#[test]
fn uefi_only_has_no_bios_args() {
    let mut p = Profile::new_default();
    p.uefi_only = true;
    let s = gen(p);
    assert!(!s.contains("-b boot/"));
    assert!(!s.contains("-eltorito-alt-boot"));
    assert!(s.contains("-e boot/grub/efiboot.img"));
}

#[test]
fn unused_bootloader_is_not_staged() {
    let mut p = Profile::new_default();
    p.bootloaders = vec![B::Limine, B::Refind];
    let s = gen(p);
    assert!(!s.contains("refind_x64.efi"));
}

#[test]
fn overlay_and_hook_use_fixed_output_names() {
    let (_, p) = golden_profiles().pop().unwrap();
    let s = gen(p);
    assert!(s.contains("\"$HERE/overlay/.\""));
    assert!(s.contains("\"$HERE/hooks/post_rootfs.sh\""));
    assert!(!s.contains("/somewhere/"));
}

#[test]
fn repos_and_packages_are_quoted_words() {
    let mut p = Profile::new_default();
    p.repos.presets = vec![RepoPreset::Voidlab];
    let s = gen(p);
    assert!(s.contains("--repository='/voidlab/repo'"));
    assert!(s.contains("'base-system'"));
}

#[test]
fn wipe_is_guarded_against_live_mounts_and_bad_work() {
    let s = gen(Profile::new_default());
    let wipe = s.find("rm -rf \"$WORK\"").unwrap();
    let guard = s.find("/proc/mounts").expect("mount guard");
    let root_guard = s.find("refusing to use WORK").expect("WORK=/ guard");
    assert!(guard < wipe && root_guard < wipe);
}

#[test]
fn work_guard_rejects_root_at_runtime() {
    let s = gen(Profile::new_default());
    let start = s.find("case \"$WORK\"").unwrap();
    let end = start + s[start..].find("esac").unwrap() + 4;
    let snippet = format!("die() {{ exit 7; }}\nWORK=/\n{}\nexit 0", &s[start..end]);
    let code = std::process::Command::new("bash").arg("-c").arg(&snippet).status().unwrap().code();
    assert_eq!(code, Some(7));
}
