mod backend;
mod generate;
mod output;
mod profile;
mod snapshot;
mod ui;

use adw::prelude::*;

const APP_ID: &str = "org.voidlinux.vessel";

fn main() -> glib::ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| {
        ui::window::build(app).present();
    });
    app.run()
}
