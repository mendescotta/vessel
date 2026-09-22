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
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut counter = 0;
        loop {
            let path = std::env::temp_dir().join(
                format!("vessel-fake-xbps-query-{}-{}", nanos, counter)
            );
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut file) => {
                    writeln!(file, "#!/bin/sh\n[ \"$2\" = \"firefox\" ] && exit 0\nexit 1").unwrap();
                    file.sync_all().unwrap();
                    drop(file);
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    let mut perms = std::fs::metadata(&path).unwrap().permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(&path, perms).unwrap();
                    return path;
                }
                Err(_) => {
                    counter += 1;
                    if counter > 100 {
                        panic!("Could not create temp file after 100 attempts");
                    }
                }
            }
        }
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
