mod backend;
mod cli;
mod generate;
mod output;
mod profile;
mod snapshot;
mod ui;

use adw::prelude::*;

const APP_ID: &str = "org.voidlinux.vessel";

fn main() -> glib::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(code) = cli::run(&args) {
        return glib::ExitCode::from(code);
    }
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| {
        ui::window::build(app).present();
    });
    app.run()
}
