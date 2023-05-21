use std::sync::Mutex;

use audio::get_audio;
use audio::WrappedAudio;
use exception::Exception;
use gtk::Application;

mod audio;
mod elements;
mod exception;
mod popout;
mod tray_icon;

use gtk::prelude::*;
use once_cell::sync::Lazy;
use popout::Popout;
use tray_icon::TrayIcon;

static AUDIO: Lazy<Mutex<WrappedAudio>> = Lazy::new(|| Mutex::new(get_audio()));

pub fn run() {
    if gtk::init().is_err() {
        Exception::Misc("Failed to initialize GTK.".to_string()).log_and_exit();
    }

    let _audio = AUDIO.lock().unwrap();
    drop(_audio);

    let app = Application::builder()
        .application_id("com.github.jaspwr.vol-applet")
        .build();

    app.connect_activate(move |app| {
        Popout::initialise(app);
        TrayIcon::initialise();
    });

    app.run();
}
