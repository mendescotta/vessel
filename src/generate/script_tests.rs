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
    chimera.init = Init::Dinit;
    chimera.userland = Userland::Chimerautils;
    chimera.bootloaders = vec![B::Limine];
    chimera.desktops = vec![Desktop::Gnome];
    chimera.display_manager = Dm::Lightdm;
    chimera.repos.presets = vec![RepoPreset::Voidlab];
    chimera.packages.extra = vec!["firefox".into()];
    chimera.packages.exclude = vec!["nano".into()];

    let mut gnu = Profile::new_default();
    gnu.name = "dinit-xfce".into();
    gnu.init = Init::Dinit;
    gnu.desktops = vec![Desktop::Xfce];
    gnu.display_manager = Dm::Lightdm;
    gnu.repos.presets = vec![RepoPreset::Voidlab, RepoPreset::Nonfree];
    gnu.services.enable = vec!["sshd".into()];
    gnu.overlay_dir = Some("/somewhere/overlay".into());
    gnu.post_rootfs_hook = Some("/somewhere/hook.sh".into());

    vec![
        ("runit-grub-console", runit),
        ("dinit-bsdutils-limine-gnome", chimera),
        ("dinit-gnu-grub-xfce", gnu),
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
fn exclude_becomes_ignorepkg() {
    let mut p = Profile::new_default();
    p.packages.exclude = vec!["nano".into()];
    let s = gen(p);
    assert!(s.contains("ignorepkg=nano"));
}

#[test]
fn no_ignore_file_when_nothing_ignored() {
    assert!(!gen(Profile::new_default()).contains("00-vessel.conf"));
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

fn between<'a>(s: &'a str, from: &str, to: &str) -> &'a str {
    let start = s.find(from).unwrap_or_else(|| panic!("no {from:?}"));
    let end = start + s[start..].find(to).unwrap_or_else(|| panic!("no {to:?}"));
    &s[start..end]
}

fn run_bash(snippet: &str, cwd: &std::path::Path) -> (Option<i32>, String) {
    let out = std::process::Command::new("bash").arg("-c").arg(snippet).current_dir(cwd).output().unwrap();
    (out.status.code(), String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vessel-work-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn relative_work_is_made_absolute() {
    let s = gen(Profile::new_default());
    let header = between(&s, "WORK=", "ROOTFS=");
    let cwd = scratch("rel");
    let (_, work) = run_bash(&format!("HERE=/h\nWORK=build/\n{header}echo \"$WORK\""), &cwd);
    assert_eq!(work, format!("{}/build", cwd.display()));
}

#[test]
fn wipe_refuses_a_folder_vessel_did_not_create() {
    let s = gen(Profile::new_default());
    let guard = between(&s, "case \"$WORK\"", "rm -rf");
    let run = |work: &std::path::Path| {
        let snippet = format!("die() {{ exit 7; }}\numount_chroot() {{ :; }}\nHERE=/h\nWORK='{}'\n{guard}exit 0", work.display());
        run_bash(&snippet, &std::env::temp_dir()).0
    };
    let foreign = scratch("foreign");
    std::fs::write(foreign.join("important"), "x").unwrap();
    assert_eq!(run(&foreign), Some(7), "non-empty folder without marker");
    std::fs::write(foreign.join(".vessel-work"), "").unwrap();
    assert_eq!(run(&foreign), Some(0), "marked scratch folder");
    assert_eq!(run(&scratch("empty")), Some(0), "empty folder");
    assert_eq!(run(&std::env::temp_dir().join("vessel-no-such-dir")), Some(0), "missing folder");
    assert!(s.contains("touch \"$WORK/.vessel-work\""));
}

#[test]
fn mount_guard_catches_a_mount_at_work_itself() {
    let s = gen(Profile::new_default());
    let guard = between(&s, "case \"$WORK\"", "rm -rf");
    let dir = scratch("mnt");
    let mounts = dir.join("mounts");
    let work = dir.join("w");
    let run = |line: String| {
        std::fs::write(&mounts, line).unwrap();
        let snippet = format!(
            "die() {{ exit 7; }}\numount_chroot() {{ :; }}\nHERE=/h\nWORK='{}'\n{}exit 0",
            work.display(),
            guard.replace("/proc/mounts", &mounts.display().to_string())
        );
        run_bash(&snippet, &dir).0
    };
    assert_eq!(run(format!("tmpfs {} tmpfs rw 0 0\n", work.display())), Some(7));
    assert_eq!(run(format!("proc {}/rootfs/proc proc rw 0 0\n", work.display())), Some(7));
    assert_eq!(run(format!("tmpfs {}-other tmpfs rw 0 0\n", work.display())), Some(0));
}

#[test]
fn free_space_is_checked_where_work_will_live() {
    let s = gen(Profile::new_default());
    let check = between(&s, "free_dir=", "free_gb=");
    let (_, dir) = run_bash(&format!("WORK=/nonexistent-vessel/a/b\n{check}echo \"$free_dir\""), &std::env::temp_dir());
    assert_eq!(dir, "/");
    assert!(s.contains("df --output=avail -BG \"$free_dir\""));
}

#[test]
fn no_bootloader_builds_a_plain_iso() {
    let mut p = Profile::new_default();
    p.bootloaders.clear();
    let s = gen(p);
    assert!(s.contains("#   bootloaders: none"));
    assert!(s.contains("xorriso -as mkisofs"));
    assert!(!s.contains("-eltorito"));
    assert!(!s.contains("-isohybrid-gpt-basdat"));
}

#[test]
fn generated_scripts_reference_no_noid() {
    for (name, p) in golden_profiles() {
        let script = gen(p);
        assert!(
            !script.to_lowercase().contains("noid"),
            "{name} generated script mentions noid"
        );
    }
}

#[test]
fn only_the_official_void_key_is_copied_into_the_rootfs() {
    for (name, p) in golden_profiles() {
        let script = gen(p);
        assert!(
            !script.contains("cp -a /var/db/xbps/keys/."),
            "{name} copies every host key into the image"
        );
        assert!(
            !script.contains("<string>Void Linux</string>"),
            "{name} selects keys by their self-declared signer name"
        );
        assert!(
            script.contains("60:ae:0c:d6:f0:95:17:80:bc:93:46:7a:89:af:a3:2d"),
            "{name} does not pin the official Void key fingerprint"
        );
        assert!(
            script.contains("official Void signing key"),
            "{name} does not fail when the official key is missing"
        );
    }
}

#[test]
fn package_cache_is_emptied_before_the_rootfs_is_packed() {
    for (name, p) in golden_profiles() {
        let script = gen(p);
        let clean = script
            .find("$ROOTFS/var/cache/xbps")
            .unwrap_or_else(|| panic!("{name} never cleans the xbps package cache"));
        let pack = script.find("mkfs.ext3 -q").unwrap_or_else(|| panic!("{name} has no image step"));
        assert!(clean < pack, "{name} cleans the xbps cache after packing the rootfs");
    }
}
