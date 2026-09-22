use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    pub supports_init: bool,
}

#[derive(Debug)]
pub enum ProbeError {
    NotFound,
    ExecFailed(std::io::Error),
    NonUtf8Output,
}

pub fn parse_help_output(help_text: &str) -> Capabilities {
    Capabilities {
        supports_init: help_text.contains("-i <init>"),
    }
}

pub fn probe(checkout: &Path) -> Result<Capabilities, ProbeError> {
    let script = checkout.join("mkiso.sh");
    if !script.is_file() {
        return Err(ProbeError::NotFound);
    }

    let output = Command::new(&script)
        .arg("-h")
        .current_dir(checkout)
        .output()
        .map_err(ProbeError::ExecFailed)?;

    let combined = format!(
        "{}{}",
        String::from_utf8(output.stdout).map_err(|_| ProbeError::NonUtf8Output)?,
        String::from_utf8(output.stderr).map_err(|_| ProbeError::NonUtf8Output)?
    );

    Ok(parse_help_output(&combined))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOID_MKLIVE_HELP: &str = "Usage: mkiso.sh [options ...] [-- mklive options ...]\n\nOPTIONS\n -a <arch>     Set architecture (or platform) in the image\n -i <init>     Set init system (default: runit)\n -b <variant>  One of base, xfce, kde, gnome\n -h            Show this help and exit\n";

    const VOID_MKLIVE_HELP: &str = "Usage: mkiso.sh [options ...] [-- mklive options ...]\n\nOPTIONS\n -a <arch>     Set architecture (or platform) in the image\n -b <variant>  One of base, xfce, kde, gnome\n -h            Show this help and exit\n";

    #[test]
    fn detects_init_support_on_noid_mklive() {
        assert!(parse_help_output(NOID_MKLIVE_HELP).supports_init);
    }

    #[test]
    fn detects_no_init_support_on_upstream_void_mklive() {
        assert!(!parse_help_output(VOID_MKLIVE_HELP).supports_init);
    }

    #[test]
    fn probe_errors_when_mkiso_missing() {
        let dir = std::env::temp_dir().join(format!("vessel-test-empty-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let result = probe(&dir);
        std::fs::remove_dir_all(&dir).ok();
        assert!(matches!(result, Err(ProbeError::NotFound)));
    }
}
