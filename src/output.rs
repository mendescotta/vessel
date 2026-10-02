use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::generate::assets::assets_for;
use crate::generate::script::{HOOK_PATH, OVERLAY_DIR};
use crate::generate::{generate_with, repos};
use crate::profile::validate::ValidProfile;

pub fn save_output(v: &ValidProfile, dir: &Path) -> io::Result<PathBuf> {
    save_output_with(v, dir, &repos::voidlab_repo_path())
}

pub fn save_output_with(v: &ValidProfile, dir: &Path, voidlab_repo: &str) -> io::Result<PathBuf> {
    let p = v.get();
    fs::create_dir_all(dir)?;
    if let Some(src) = &p.overlay_dir {
        if let (Ok(src), Ok(out)) = (src.canonicalize(), dir.canonicalize()) {
            if out.starts_with(&src) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("the overlay folder {} contains the output folder; pick an output folder outside it", src.display()),
                ));
            }
        }
    }

    remove_path(&dir.join("vmklive"))?;
    for asset in assets_for(p.init) {
        let path = dir.join(asset.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        write_mode(&path, asset.contents, if asset.executable { 0o755 } else { 0o644 })?;
    }

    let mut saved = p.clone();
    let overlay_dest = dir.join(OVERLAY_DIR);
    match &p.overlay_dir {
        Some(src) if !same_path(src, &overlay_dest) => {
            remove_path(&overlay_dest)?;
            copy_dir(src, &overlay_dest)?;
        }
        Some(_) => {}
        None => remove_path(&overlay_dest)?,
    }
    saved.overlay_dir = p.overlay_dir.as_ref().map(|_| PathBuf::from(OVERLAY_DIR));

    let hook_dest = dir.join(HOOK_PATH);
    match &p.post_rootfs_hook {
        Some(src) if !same_path(src, &hook_dest) => {
            fs::create_dir_all(hook_dest.parent().expect("hook path has a parent"))?;
            fs::copy(src, &hook_dest)?;
            fs::set_permissions(&hook_dest, fs::Permissions::from_mode(0o755))?;
        }
        Some(_) => {}
        None => remove_path(&hook_dest)?,
    }
    saved.post_rootfs_hook = p.post_rootfs_hook.as_ref().map(|_| PathBuf::from(HOOK_PATH));

    write_mode(&dir.join("profile.toml"), &saved.to_toml(), 0o644)?;
    let script = dir.join("build.sh");
    write_mode(&script, &generate_with(v, voidlab_repo), 0o755)?;
    Ok(script)
}

fn write_mode(path: &Path, contents: &str, mode: u32) -> io::Result<()> {
    let name = path.file_name().expect("output paths have a file name").to_string_lossy();
    let tmp = path.with_file_name(format!(".{name}.tmp"));
    fs::write(&tmp, contents)?;
    fs::set_permissions(&tmp, fs::Permissions::from_mode(mode))?;
    fs::rename(&tmp, path)
}

fn same_path(a: &Path, b: &Path) -> bool {
    matches!((a.canonicalize(), b.canonicalize()), (Ok(x), Ok(y)) if x == y)
}

fn remove_path(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(m) if m.is_dir() => fs::remove_dir_all(path),
        Ok(_) => fs::remove_file(path),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

fn copy_dir(src: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(dest)?;
    fs::set_permissions(dest, fs::metadata(src)?.permissions())?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            std::os::unix::fs::symlink(fs::read_link(&from)?, &to)?;
        } else if kind.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "output_tests.rs"]
mod tests;
