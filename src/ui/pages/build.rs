use adw::prelude::*;

use crate::backend::build_runner::{spawn_build, write_argfile, BuildEvent, BuildResult};
use crate::backend::mklive::{build_argv, BuildOptions};

pub struct BuildPage {
    pub widget: gtk::Box,
    status_label: gtk::Label,
    log_view: gtk::TextView,
}

impl BuildPage {
    pub fn new() -> Self {
        let widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .margin_top(24)
            .margin_bottom(24)
            .margin_start(24)
            .margin_end(24)
            .build();

        let status_label = gtk::Label::new(Some("Ready to build."));
        let log_view = gtk::TextView::builder().editable(false).monospace(true).build();
        let log_scroller = gtk::ScrolledWindow::builder().vexpand(true).build();
        log_scroller.set_child(Some(&log_view));

        widget.append(&status_label);
        widget.append(&log_scroller);

        Self { widget, status_label, log_view }
    }

    pub fn start<F>(&self, opts: &BuildOptions, on_finished: F)
    where
        F: Fn(bool) + 'static,
    {
        self.status_label.set_text("Building…");
        self.log_view.buffer().set_text("");

        let argv = build_argv(opts);
        let argfile = match write_argfile(&argv) {
            Ok(path) => path,
            Err(e) => {
                self.status_label.set_text(&format!("Failed to prepare build: {e}"));
                on_finished(false);
                return;
            }
        };

        let (tx, rx) = async_channel::unbounded::<BuildEvent>();
        spawn_build(
            "pkexec",
            &[
                "/usr/libexec/vessel/run-mkiso".to_string(),
                argfile.to_string_lossy().to_string(),
                opts.checkout.to_string_lossy().to_string(),
            ],
            move |event| {
                let _ = tx.send_blocking(event);
            },
        );

        let status_label = self.status_label.clone();
        let log_view = self.log_view.clone();
        glib::spawn_future_local(async move {
            while let Ok(event) = rx.recv().await {
                match event {
                    BuildEvent::Log(line) => {
                        let buf = log_view.buffer();
                        let mut end = buf.end_iter();
                        buf.insert(&mut end, &format!("{line}\n"));
                    }
                    BuildEvent::Finished(BuildResult::Success) => {
                        status_label.set_text("Build finished successfully.");
                        on_finished(true);
                        break;
                    }
                    BuildEvent::Finished(BuildResult::Failed(code)) => {
                        status_label.set_text(&format!("Build failed (exit {code})."));
                        on_finished(false);
                        break;
                    }
                }
            }
        });
    }
}
