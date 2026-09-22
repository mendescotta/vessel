use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Init {
    Runit,
    Dinit,
}

impl Init {
    pub fn as_flag(&self) -> &'static str {
        match self {
            Init::Runit => "runit",
            Init::Dinit => "dinit",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOptions {
    pub checkout: PathBuf,
    pub arch: String,
    pub init: Option<Init>,
    pub variant: String,
    pub kernel: Option<String>,
    pub packages: Vec<String>,
    pub output: PathBuf,
}

pub fn build_argv(opts: &BuildOptions) -> Vec<String> {
    let mut argv = vec!["-a".to_string(), opts.arch.clone()];

    if let Some(init) = opts.init {
        argv.push("-i".to_string());
        argv.push(init.as_flag().to_string());
    }

    argv.push("-b".to_string());
    argv.push(opts.variant.clone());
    argv.push("--".to_string());

    if let Some(kernel) = &opts.kernel {
        argv.push("-v".to_string());
        argv.push(kernel.clone());
    }

    if !opts.packages.is_empty() {
        argv.push("-p".to_string());
        argv.push(opts.packages.join(" "));
    }

    argv.push("-o".to_string());
    argv.push(opts.output.to_string_lossy().to_string());

    argv
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_opts() -> BuildOptions {
        BuildOptions {
            checkout: PathBuf::from("/tmp/checkout"),
            arch: "x86_64".to_string(),
            init: None,
            variant: "xfce".to_string(),
            kernel: None,
            packages: vec![],
            output: PathBuf::from("/tmp/out/void.iso"),
        }
    }

    #[test]
    fn omits_init_flag_when_none() {
        let argv = build_argv(&base_opts());
        assert_eq!(
            argv,
            vec!["-a", "x86_64", "-b", "xfce", "--", "-o", "/tmp/out/void.iso"]
        );
    }

    #[test]
    fn includes_init_flag_when_some() {
        let mut opts = base_opts();
        opts.init = Some(Init::Dinit);
        let argv = build_argv(&opts);
        assert_eq!(
            argv,
            vec!["-a", "x86_64", "-i", "dinit", "-b", "xfce", "--", "-o", "/tmp/out/void.iso"]
        );
    }

    #[test]
    fn includes_kernel_and_packages_after_double_dash() {
        let mut opts = base_opts();
        opts.kernel = Some("linux-lts".to_string());
        opts.packages = vec!["firefox".to_string(), "gimp".to_string()];
        let argv = build_argv(&opts);
        assert_eq!(
            argv,
            vec![
                "-a", "x86_64", "-b", "xfce", "--",
                "-v", "linux-lts", "-p", "firefox gimp", "-o", "/tmp/out/void.iso"
            ]
        );
    }
}
