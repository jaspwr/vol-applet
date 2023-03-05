
use std::sync::Mutex;

use audio::WrappedAudio;
use audio::get_audio;
use gtk::Application;


mod tray_icon;
mod exception;
mod popout;
mod elements;
mod audio;

use gtk::prelude::*;
use once_cell::sync::Lazy;
use popout::Popout;
use tray_icon::TrayIcon;


static TRAY_ICON: Mutex<Option<TrayIcon>> = Mutex::new(None);
static POPOUT: Mutex<Option<Popout>> = Mutex::new(None);
static AUDIO: Lazy<Mutex<WrappedAudio>> = Lazy::new(|| Mutex::new(get_audio()));

fn main() {
    if gtk::init().is_err() {
        // TODO
        println!("Error loading GTK!");
        return;
    }

    let _a = AUDIO.lock().unwrap();

    let app = Application::builder()
        .application_id("com.github.jaspwr.vol-applet")
        .build();

    app.connect_activate(move |app| {
        POPOUT.lock().unwrap().replace(Popout::new(app));
        TRAY_ICON.lock().unwrap().replace(TrayIcon::new());
    });

    app.run();
}

