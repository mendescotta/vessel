use adw::prelude::*;

use super::{issues_group, page, stack, text_row};
use crate::ui::state::Shared;

pub fn build(s: &Shared) -> gtk::Widget {
    let page = page("System", "The base system: which init and core userland it ships, and how the live session identifies itself.");
    page.add(&issues_group(s, &["name", "arch", "kernel", "live", "repos"]));
    page.add(&stack::group(s));

    let base = adw::PreferencesGroup::builder().title("Base").build();
    base.add(&text_row(s, "Profile name", |p| p.name.clone(), |p, v| p.name = v));
    base.add(&text_row(s, "Kernel package", |p| p.kernel.clone(), |p, v| p.kernel = v));
    page.add(&base);

    let live = adw::PreferencesGroup::builder()
        .title("Live session")
        .description("The live user has no password.")
        .build();
    live.add(&text_row(s, "Hostname", |p| p.live.hostname.clone(), |p, v| p.live.hostname = v));
    live.add(&text_row(s, "Live user", |p| p.live.user.clone(), |p, v| p.live.user = v));
    live.add(&text_row(s, "Locale", |p| p.live.locale.clone(), |p, v| p.live.locale = v));
    live.add(&text_row(s, "Keymap", |p| p.live.keymap.clone(), |p, v| p.live.keymap = v));
    live.add(&text_row(s, "Timezone", |p| p.live.timezone.clone(), |p, v| p.live.timezone = v));
    page.add(&live);

    page.upcast()
}
