use crate::profile::{Desktop, DisplayManager};

pub const GRAPHICAL_BASE: &[&str] = &[
    "dbus",
    "elogind",
    "polkit",
    "NetworkManager",
    "xorg-minimal",
    "xorg-fonts",
    "mesa-dri",
    "pipewire",
    "wireplumber",
    "xdg-user-dirs",
];

pub const GRAPHICAL_SERVICES: &[&str] = &["dbus", "elogind", "NetworkManager"];

pub fn desktop_packages(d: Desktop) -> &'static [&'static str] {
    match d {
        Desktop::Gnome => &["gnome"],
        Desktop::Cosmic => &["cosmic-desktop"],
        Desktop::Cinnamon => &["cinnamon"],
        Desktop::Xfce => &["xfce4"],
        Desktop::Budgie => &["budgie-desktop"],
        Desktop::Kde => &["kde-plasma"],
    }
}

pub fn dm_packages(dm: DisplayManager) -> &'static [&'static str] {
    match dm {
        DisplayManager::None => &[],
        DisplayManager::Lightdm => &["lightdm", "lightdm-gtk3-greeter"],
        DisplayManager::Sddm => &["sddm"],
        DisplayManager::Gdm => &["gdm"],
        DisplayManager::CosmicGreeter => &["cosmic-greeter"],
    }
}

pub fn dm_service(dm: DisplayManager) -> Option<&'static str> {
    match dm {
        DisplayManager::None => None,
        DisplayManager::Lightdm => Some("lightdm"),
        DisplayManager::Sddm => Some("sddm"),
        DisplayManager::Gdm => Some("gdm"),
        DisplayManager::CosmicGreeter => Some("cosmic-greeter"),
    }
}
