use crate::profile::Userland;

pub fn packages(u: Userland) -> &'static [&'static str] {
    match u {
        Userland::Gnu => &[],
        Userland::Chimerautils => &["chimerautils"],
    }
}
