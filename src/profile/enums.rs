use serde::{Deserialize, Serialize};

macro_rules! axis {
    ($(#[$m:meta])* $name:ident { $($variant:ident => $id:literal, $label:literal;)+ }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum $name {
            $(#[serde(rename = $id)] $variant,)+
        }

        impl $name {
            #[allow(dead_code)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Init {
    #[serde(rename = "runit")]
    Runit,
    #[serde(rename = "dinit")]
    Dinit,
}

impl Init {
    pub const ALL: &'static [Self] = &[Self::Runit, Self::Dinit];

    pub fn id(self) -> &'static str {
        match self {
            Self::Runit => "runit",
            Self::Dinit => "dinit",
        }
    }

    pub fn label(self) -> &'static str {
        self.id()
    }
}

axis!(Userland {
    Gnu => "gnu", "GNU coreutils";
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
