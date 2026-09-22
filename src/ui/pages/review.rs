use adw::prelude::*;

use crate::backend::mklive::{build_argv, BuildOptions};

pub struct ReviewPage {
    pub widget: gtk::Label,
}

impl ReviewPage {
    pub fn new() -> Self {
        let widget = gtk::Label::builder()
            .wrap(true)
            .xalign(0.0)
            .valign(gtk::Align::Start)
            .margin_top(24)
            .margin_bottom(24)
            .margin_start(24)
            .margin_end(24)
            .build();
        Self { widget }
    }

    pub fn set_summary(&self, opts: &BuildOptions) {
        self.widget.set_label(&format!("mkiso.sh {}", build_argv(opts).join(" ")));
    }
}
