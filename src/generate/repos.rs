use crate::profile::{Profile, RepoPreset};

pub const OFFICIAL: &str = "https://repo-default.voidlinux.org/current";

/// Where the `voidlab` repository preset points: `VESSEL_VOIDLAB_REPO` (a local repository directory, or a
/// URL), or empty when unset. There is deliberately no default: the build installs with `-y`, which imports a
/// remote repository's signing key without asking, so a remote repository must be chosen explicitly.
pub fn voidlab_repo_path() -> String {
    voidlab_repo_path_from(std::env::var("VESSEL_VOIDLAB_REPO").ok())
}

pub fn voidlab_repo_path_from(env: Option<String>) -> String {
    env.unwrap_or_default()
}

/// The `voidlab` preset needs a repository to be named; refuse to generate without one.
pub fn require_voidlab_repo(p: &Profile, path: &str) -> Result<(), String> {
    if p.repos.presets.contains(&RepoPreset::Voidlab) && path.is_empty() {
        return Err("the voidlab repository preset needs VESSEL_VOIDLAB_REPO: the repo/ directory of a local voidlab \
             checkout, or a repository URL whose signing key you trust (the build imports unknown keys without asking)"
            .to_string());
    }
    Ok(())
}

/// The official repository for an architecture: musl lives under `/musl`.
pub fn official(arch: &str) -> String {
    if arch.ends_with("-musl") {
        format!("{OFFICIAL}/musl")
    } else {
        OFFICIAL.to_string()
    }
}

pub fn preset_urls(
    preset: RepoPreset,
    all: &[RepoPreset],
    voidlab: &str,
    arch: &str,
) -> Vec<String> {
    let official = official(arch);
    match preset {
        RepoPreset::Voidlab if voidlab.is_empty() => Vec::new(),
        RepoPreset::Voidlab => vec![voidlab.to_string()],
        RepoPreset::Nonfree => vec![format!("{official}/nonfree")],
        RepoPreset::Multilib => {
            let mut v = vec![format!("{official}/multilib")];
            if all.contains(&RepoPreset::Nonfree) {
                v.push(format!("{official}/multilib/nonfree"));
            }
            v
        }
    }
}

pub fn repo_list_with(p: &Profile, voidlab: &str) -> Vec<String> {
    let mut out = p.repos.custom.clone();
    for preset in &p.repos.presets {
        out.extend(preset_urls(*preset, &p.repos.presets, voidlab, &p.arch));
    }
    out.push(official(&p.arch));
    super::dedup(out)
}
