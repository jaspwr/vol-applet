
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


static mut popout_glob: Option<Popout> = None;


fn main() {
    if gtk::init().is_err() {
        // TODO
        println!("Error loading GTK!");
        return;
    }

    let mut audio = get_audio();
    let outputs = audio.get_outputs();
    std::thread::sleep(Duration::from_secs(1));
    println!("Found {} outputs", outputs.len());

    let app = Application::builder()
        .application_id("org.example.HelloWorld")
        .build();


    app.connect_activate(move |app| {

        unsafe {
            let button = Button::with_label("Click me!");
            button.connect_clicked(|_| {
                eprintln!("Clicked!");
            });


            let mut icon = tray_icon::TrayIcon::new();
            popout_glob = Some(Popout::new(&app, &mut icon));
        }

    });

    app.run();
}

