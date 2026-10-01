use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use adw::prelude::*;
use gtk::gio;

use super::page;
use crate::backend::build_runner::{spawn_build, BuildEvent, BuildResult};
use crate::output::save_output;
use crate::ui::state::{self, Shared};

const LAUNCHER: &str = "/usr/libexec/vessel/run-build";

pub fn build(s: &Shared) -> gtk::Widget {
    let toasts = adw::ToastOverlay::new();
    let page = page(
        "Output & build",
        "Save writes profile.toml, build.sh and its helper files to a folder you can keep, copy or run later with `sudo ./build.sh`. Build now runs it here.",
    );
    toasts.set_child(Some(&page));

    let chosen: Rc<RefCell<Option<PathBuf>>> = Rc::default();
    let outdir = {
        let (chosen, s) = (chosen.clone(), s.clone());
        move || -> PathBuf {
            chosen.borrow().clone().unwrap_or_else(|| {
                let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
                home.join("vessel").join(&s.borrow().profile.name)
            })
        }
    };

    let group = adw::PreferencesGroup::builder().title("Output folder").build();
    let folder = adw::ActionRow::builder().title("Folder").subtitle(outdir().display().to_string()).build();
    let choose = gtk::Button::builder().label("Choose…").valign(gtk::Align::Center).build();
    folder.add_suffix(&choose);
    group.add(&folder);
    page.add(&group);

    let actions = adw::PreferencesGroup::new();
    let buttons = gtk::Box::builder().spacing(12).halign(gtk::Align::Center).build();
    let save = gtk::Button::builder().label("Save").css_classes(["pill", "suggested-action"]).build();
    let run = gtk::Button::builder().label("Build now").css_classes(["pill"]).sensitive(false).build();
    buttons.append(&save);
    buttons.append(&run);
    actions.add(&buttons);
    page.add(&actions);

    let log_group = adw::PreferencesGroup::builder().title("Build log").visible(false).build();
    let status = gtk::Label::builder().xalign(0.0).build();
    let log = gtk::TextView::builder().editable(false).monospace(true).build();
    let scroller = gtk::ScrolledWindow::builder().min_content_height(360).child(&log).css_classes(["card"]).build();
    log_group.add(&status);
    log_group.add(&scroller);
    page.add(&log_group);

    let saved: Rc<RefCell<Option<PathBuf>>> = Rc::default();

    {
        let (folder, s, outdir, run, saved) = (folder.clone(), s.clone(), outdir.clone(), run.clone(), saved.clone());
        state::on_change(&s, move |_, _, _| {
            folder.set_subtitle(&outdir().display().to_string());
            saved.borrow_mut().take();
            run.set_sensitive(false);
        });
    }
    {
        let (folder, chosen, outdir) = (folder.clone(), chosen.clone(), outdir.clone());
        choose.connect_clicked(move |btn| {
            let dialog = gtk::FileDialog::builder().title("Choose output folder").build();
            let window = btn.root().and_downcast::<gtk::Window>();
            let (folder, chosen, outdir) = (folder.clone(), chosen.clone(), outdir.clone());
            dialog.select_folder(window.as_ref(), gio::Cancellable::NONE, move |res| {
                if let Some(path) = res.ok().and_then(|f| f.path()) {
                    *chosen.borrow_mut() = Some(path);
                    folder.set_subtitle(&outdir().display().to_string());
                }
            });
        });
    }
    {
        let (s, toasts, run, saved) = (s.clone(), toasts.clone(), run.clone(), saved.clone());
        save.connect_clicked(move |_| {
            let dir = outdir();
            let valid = s.borrow().valid.clone();
            let Some(valid) = valid else {
                toasts.add_toast(adw::Toast::new("The profile has errors; see Review."));
                return;
            };
            match save_output(&valid, &dir) {
                Ok(script) => {
                    toasts.add_toast(adw::Toast::new(&format!("Saved {}", script.display())));
                    *saved.borrow_mut() = Some(dir);
                    run.set_sensitive(true);
                }
                Err(e) => toasts.add_toast(adw::Toast::new(&format!("Couldn't save to {}: {e}", dir.display()))),
            }
        });
    }
    {
        let saved = saved.clone();
        run.connect_clicked(move |run| {
            let Some(dir) = saved.borrow().clone() else { return };
            run.set_sensitive(false);
            log_group.set_visible(true);
            status.set_text("Building… (authentication required)");
            log.buffer().set_text("");

            let (tx, rx) = async_channel::unbounded::<BuildEvent>();
            spawn_build("pkexec", &[LAUNCHER.to_string(), dir.to_string_lossy().into_owned()], move |event| {
                let _ = tx.send_blocking(event);
            });
            let (status, log, run, saved) = (status.clone(), log.clone(), run.clone(), saved.clone());
            glib::spawn_future_local(async move {
                while let Ok(event) = rx.recv().await {
                    match event {
                        BuildEvent::Log(line) => {
                            let buf = log.buffer();
                            buf.insert(&mut buf.end_iter(), &format!("{line}\n"));
                            let end = buf.create_mark(None, &buf.end_iter(), false);
                            log.scroll_to_mark(&end, 0.0, false, 0.0, 1.0);
                            buf.delete_mark(&end);
                        }
                        BuildEvent::Finished(result) => {
                            status.set_text(&match result {
                                BuildResult::Success => format!("Build finished. The ISO is in {}", dir.join("out").display()),
                                BuildResult::Failed(126 | 127) => format!(
                                    "Not authorized, or {LAUNCHER} isn't installed. You can run it yourself: cd {} && sudo ./build.sh",
                                    dir.display()
                                ),
                                BuildResult::Failed(code) => format!("Build failed (exit {code}); see the log."),
                            });
                            run.set_sensitive(saved.borrow().is_some());
                            break;
                        }
                    }
                }
            });
        });
    }

    toasts.upcast()
}
