use crate::profile::Userland;

pub fn packages(u: Userland) -> &'static [&'static str] {
    match u {
        Userland::Gnu => &[],
        // Co-installs with GNU coreutils under /usr/lib/chimerautils and is put
        // first in PATH by its own profile.d script; nothing to ignore.
        Userland::Chimerautils => &["chimerautils"],
    }
}
