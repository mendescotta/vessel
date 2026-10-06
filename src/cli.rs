use crate::output::save_output_with;
use crate::profile::validate::validate;
use crate::profile::Profile;
use std::path::{Path, PathBuf};

pub fn generate(profile: &Path, out: &Path, voidlab_repo: &str) -> Result<PathBuf, String> {
    let p = Profile::load(profile).map_err(|e| format!("{}: {e}", profile.display()))?;
    let (issues, valid) = validate(&p);
    let valid =
        valid.ok_or_else(|| format!("{}: invalid profile: {issues:?}", profile.display()))?;
    save_output_with(&valid, out, voidlab_repo).map_err(|e| format!("{}: {e}", out.display()))
}

pub fn run(args: &[String]) -> Option<i32> {
    let pos = args.iter().position(|a| a == "--generate")?;
    let profile = args.get(pos + 1).map(PathBuf::from);
    let out = args
        .iter()
        .position(|a| a == "--out")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);
    let (Some(profile), Some(out)) = (profile, out) else {
        eprintln!("usage: vessel --generate <profile.toml> --out <dir>");
        return Some(2);
    };
    match generate(&profile, &out, &crate::generate::repos::voidlab_repo_path()) {
        Ok(script) => {
            println!("{}", script.display());
            Some(0)
        }
        Err(e) => {
            eprintln!("vessel: {e}");
            Some(1)
        }
    }
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
