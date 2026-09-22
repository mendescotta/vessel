use adw::prelude::*;

pub fn build(app: &adw::Application) -> adw::ApplicationWindow {
    adw::ApplicationWindow::builder()
        .application(app)
        .title("vessel")
        .default_width(880)
        .default_height(620)
        .build()
}
