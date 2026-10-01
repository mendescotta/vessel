use adw::prelude::*;

use super::{issues_group, page};
use crate::generate::generate;
use crate::ui::state::{self, Shared};

pub fn build(s: &Shared) -> gtk::Widget {
    let page = page("Review", "Everything that needs attention, and the build script this profile generates.");
    page.add(&issues_group(s, &[]));

    let group = adw::PreferencesGroup::builder().title("build.sh").build();
    let view = gtk::TextView::builder()
        .editable(false)
        .monospace(true)
        .wrap_mode(gtk::WrapMode::None)
        .top_margin(8)
        .bottom_margin(8)
        .left_margin(8)
        .right_margin(8)
        .build();
    let scroller = gtk::ScrolledWindow::builder().min_content_height(420).child(&view).css_classes(["card"]).build();
    group.add(&scroller);
    page.add(&group);

    let render = {
        let view = view.clone();
        let s = s.clone();
        move || {
            let text = match &s.borrow().valid {
                Some(v) => generate(v),
                None => "Fix the errors above to see the generated script.".to_string(),
            };
            view.buffer().set_text(&text);
        }
    };
    render();
    state::on_change(s, move |_, _, _| render());
    page.upcast()
}
