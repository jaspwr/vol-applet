
use std::sync::Arc;
use std::sync::Mutex;

use gtk::Application;


mod tray_icon;
mod exception;
mod popout;
mod elements;
mod audio;

use gtk::prelude::*;
use popout::Popout;

static mut TRAY_ICON: Option<Arc<Mutex<tray_icon::TrayIcon>>> = None;
fn main() {
    if gtk::init().is_err() {
        // TODO
        println!("Error loading GTK!");
        return;
    }

    let app = Application::builder()
        .application_id("com.github.jaspwr.vol-applet")
        .build();


    app.connect_activate(move |app| {
        let popout: Arc<Mutex<Popout>> = Arc::new(
            Mutex::new(
                Popout::new(app)
            )
        );
        let icon = tray_icon::TrayIcon::new(popout);
        unsafe {
            TRAY_ICON = Some(Arc::new(Mutex::new(icon)));
        }
    });

    app.run();
}

