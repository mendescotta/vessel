use super::validate::required_presets;
use super::{Init, Profile, Userland};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Support {
    Standard,
    Supported,
    Experimental,
}

impl Support {
    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Supported => "Supported",
            Self::Experimental => "Experimental",
        }
    }
}

pub fn support(init: Init, userland: Userland) -> Support {
    match (init, userland) {
        (Init::Runit, Userland::Gnu) => Support::Standard,
        (Init::Runit, Userland::Chimerautils) => Support::Experimental,
        (Init::Dinit, _) => Support::Supported,
    }
}

pub fn select(p: &mut Profile, init: Init, userland: Userland) {
    p.init = init;
    p.userland = userland;
    for r in required_presets(p) {
        if !p.repos.presets.contains(&r) {
            p.repos.presets.push(r);
        }
    }
}

#[cfg(test)]
#[path = "stack_tests.rs"]
mod tests;
