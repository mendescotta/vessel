use adw::prelude::*;

use crate::ui::state::Shared;

pub fn build(_s: &Shared) -> gtk::Widget {
    super::page("desktop", "").upcast()
}
