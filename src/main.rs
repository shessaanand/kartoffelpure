use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};

fn main() {
    let app = Application::builder()
        .application_id("com.kartoffelpure.browser")
        .build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("KartoffelPure")
            .default_width(1200)
            .default_height(800)
            .build();

        window.present();
    });

    app.run();
}
