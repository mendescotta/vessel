use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageCheck {
    Exists,
    NotFound,
}

pub fn check_with(command: &str, name: &str) -> std::io::Result<PackageCheck> {
    let status = Command::new(command)
        .arg("-R")
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    Ok(if status.success() { PackageCheck::Exists } else { PackageCheck::NotFound })
}

pub fn package_exists(name: &str) -> std::io::Result<PackageCheck> {
    check_with("xbps-query", name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    fn fake_xbps_query() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("vessel-fake-xbps-query-{}", std::process::id()));
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "#!/bin/sh\n[ \"$2\" = \"firefox\" ] && exit 0\nexit 1").unwrap();
        drop(file);
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms).unwrap();
        path
    }

    #[test]
    fn reports_exists_for_known_package() {
        let script = fake_xbps_query();
        let result = check_with(script.to_str().unwrap(), "firefox").unwrap();
        std::fs::remove_file(&script).ok();
        assert_eq!(result, PackageCheck::Exists);
    }

    #[test]
    fn reports_not_found_for_unknown_package() {
        let script = fake_xbps_query();
        let result = check_with(script.to_str().unwrap(), "not-a-real-package").unwrap();
        std::fs::remove_file(&script).ok();
        assert_eq!(result, PackageCheck::NotFound);
    }
}
