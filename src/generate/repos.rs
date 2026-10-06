use crate::profile::{Profile, RepoPreset};

pub const OFFICIAL: &str = "https://repo-default.voidlinux.org/current";

/// The public voidlab binary repository, used when `VESSEL_VOIDLAB_REPO` is not set.
pub const VOIDLAB_RELEASE_REPO: &str =
    "https://github.com/mendescotta/voidlab/releases/download/repo";

/// Where the `voidlab` repository preset points: `VESSEL_VOIDLAB_REPO` (a path or URL) or the public repository.
pub fn voidlab_repo_path() -> String {
    voidlab_repo_path_from(std::env::var("VESSEL_VOIDLAB_REPO").ok())
}

pub fn voidlab_repo_path_from(env: Option<String>) -> String {
    env.filter(|s| !s.is_empty())
        .unwrap_or_else(|| VOIDLAB_RELEASE_REPO.to_string())
}

pub fn preset_urls(preset: RepoPreset, all: &[RepoPreset], voidlab: &str) -> Vec<String> {
    match preset {
        RepoPreset::Voidlab => vec![voidlab.to_string()],
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

pub fn repo_list_with(p: &Profile, voidlab: &str) -> Vec<String> {
    let mut out = p.repos.custom.clone();
    for preset in &p.repos.presets {
        out.extend(preset_urls(*preset, &p.repos.presets, voidlab));
    }
    out.push(OFFICIAL.to_string());
    super::dedup(out)
}
