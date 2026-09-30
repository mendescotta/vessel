use adw::prelude::*;

pub fn build(app: &adw::Application) -> adw::ApplicationWindow {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("vessel")
        .default_width(960)
        .default_height(680)
        .build();
    let status = adw::StatusPage::builder().title("vessel").build();
    window.set_content(Some(&status));
    window
}
