
use gtk::Application;
use gtk::Button;


mod tray_icon;
mod exception;
mod popout;

use gtk::prelude::*;
use popout::Popout;

static mut popout_glob: Option<Popout> = None;


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
        unsafe {
            
            // win_ref.set_opacity(0.5);

            let button = Button::with_label("Click me!");
            button.connect_clicked(|_| {
                eprintln!("Clicked!");
            });
    
            // let slider = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
            // win_ref.add(&slider);
            

            // // win_ref.


            let mut icon = tray_icon::TrayIcon::new();
            popout_glob = Some(Popout::new(&app, &mut icon));
        }

        // win.show_all();
    });

    app.run();
}

