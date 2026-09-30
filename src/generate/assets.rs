use crate::profile::Init;

/// A file written next to `build.sh`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Asset {
    pub path: &'static str,
    pub contents: &'static str,
    pub executable: bool,
}

macro_rules! asset {
    ($path:literal, $exec:expr) => {
        Asset { path: $path, contents: include_str!(concat!("../../assets/", $path)), executable: $exec }
    };
}

/// void-mklive's live-boot dracut module, taken from noid-mklive (which adds
/// dinit support via `noid.init_system=`). BSD-2-Clause, see vmklive/COPYING.
const VMKLIVE: &[Asset] = &[
    asset!("vmklive/59-mtd.rules", false),
    asset!("vmklive/61-mtd.rules", false),
    asset!("vmklive/COPYING", false),
    asset!("vmklive/accessibility.sh", true),
    asset!("vmklive/adduser.sh", true),
    asset!("vmklive/display-manager-autologin.sh", true),
    asset!("vmklive/getty-serial.sh", true),
    asset!("vmklive/locale.sh", true),
    asset!("vmklive/module-setup.sh", true),
    asset!("vmklive/mtd.sh", true),
    asset!("vmklive/nomodeset.sh", true),
];

const DYNAMOD_INITRAMFS: Asset = asset!("dynamod-initramfs.sh", true);

pub fn assets_for(i: Init) -> Vec<Asset> {
    if super::init::uses_dracut(i) {
        VMKLIVE.to_vec()
    } else {
        vec![DYNAMOD_INITRAMFS]
    }
}
