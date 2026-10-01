use crate::profile::{Profile, RepoPreset};

pub const OFFICIAL: &str = "https://repo-default.voidlinux.org/current";
pub const NOID: &str = "https://github.com/noid-linux/xbps-repo/releases/latest/download";

/// Where the voidlab overlay repo lives. Resolved when the script is generated,
/// because the build runs as root where `$HOME` points elsewhere.
pub fn voidlab_repo_path() -> String {
    voidlab_repo_path_from(std::env::var("VESSEL_VOIDLAB_REPO").ok(), std::env::var("HOME").ok())
}

pub fn voidlab_repo_path_from(env: Option<String>, home: Option<String>) -> String {
    env.filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("{}/Projects/voidlab/voidlab/repo", home.unwrap_or_default()))
}

pub fn preset_urls(preset: RepoPreset, all: &[RepoPreset], voidlab: &str) -> Vec<String> {
    match preset {
        RepoPreset::Voidlab => vec![voidlab.to_string()],
        RepoPreset::Noid => vec![NOID.to_string()],
        RepoPreset::Nonfree => vec![format!("{OFFICIAL}/nonfree")],
        RepoPreset::Multilib => {
            let mut v = vec![format!("{OFFICIAL}/multilib")];
            if all.contains(&RepoPreset::Nonfree) {
                v.push(format!("{OFFICIAL}/multilib/nonfree"));
            }
            v
        }
    }
}

/// Custom repos first, then presets in profile order, then the official repo.
pub fn repo_list_with(p: &Profile, voidlab: &str) -> Vec<String> {
    let mut out = p.repos.custom.clone();
    for preset in &p.repos.presets {
        out.extend(preset_urls(*preset, &p.repos.presets, voidlab));
    }
    out.push(OFFICIAL.to_string());
    super::dedup(out)
}
