
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use audio::Audio;
use audio::get_audio;
use gtk::Application;
use gtk::Button;


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
        .application_id("org.example.HelloWorld")
        .build();


    app.connect_activate(move |app| {
        let mut popout: Arc<Mutex<Popout>> = Arc::new(
            Mutex::new(
                Popout::new(&app)
            )
        );
        let mut icon = tray_icon::TrayIcon::new(popout);
        unsafe {
            TRAY_ICON = Some(Arc::new(Mutex::new(icon)));
        }
    });

    app.run();
}

