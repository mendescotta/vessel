use serde::{Deserialize, Serialize};

macro_rules! axis {
    ($(#[$m:meta])* $name:ident { $($variant:ident => $id:literal, $label:literal;)+ }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum $name {
            $(#[serde(rename = $id)] $variant,)+
        }

        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant,)+];

            pub fn id(self) -> &'static str {
                match self { $(Self::$variant => $id,)+ }
            }

            pub fn label(self) -> &'static str {
                match self { $(Self::$variant => $label,)+ }
            }
        }
    };
}

axis!(Init {
    Runit => "runit", "runit";
    DinitChimera => "dinit-chimera", "dinit (Chimera service set)";
    DinitNoid => "dinit-noid", "dinit (noid)";
    Dynamod => "dynamod", "dynamod";
});

axis!(Userland {
    Gnu => "gnu", "GNU coreutils";
    Chimerautils => "chimerautils", "chimerautils (FreeBSD-derived)";
});

axis!(Bootloader {
    Grub => "grub", "GRUB";
    Limine => "limine", "Limine";
    Refind => "refind", "rEFInd";
});

axis!(Desktop {
    Gnome => "gnome", "GNOME";
    Cosmic => "cosmic", "COSMIC";
    Cinnamon => "cinnamon", "Cinnamon";
    Xfce => "xfce", "Xfce";
    Budgie => "budgie", "Budgie";
    Kde => "kde", "KDE Plasma";
});

axis!(DisplayManager {
    None => "none", "None (console login)";
    Lightdm => "lightdm", "LightDM";
    Sddm => "sddm", "SDDM";
    Gdm => "gdm", "GDM";
    CosmicGreeter => "cosmic-greeter", "COSMIC Greeter";
});

axis!(RepoPreset {
    Voidlab => "voidlab", "voidlab (local overlay repo)";
    Nonfree => "nonfree", "nonfree";
    Multilib => "multilib", "multilib";
    Noid => "noid", "noid";
});

impl Bootloader {
    pub fn bios(self) -> bool {
        match self {
            Self::Grub | Self::Limine => true,
            Self::Refind => false,
        }
    }

    pub fn uefi(self) -> bool {
        true
    }
}
