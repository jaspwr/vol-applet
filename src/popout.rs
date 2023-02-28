use std::rc::Rc;

use gtk::traits::{GtkWindowExt, WidgetExt, ButtonExt, ContainerExt, ProgressBarExt};
use gtk::{Application, ApplicationWindow, Inhibit};

use crate::elements::VolumeSlider;
use crate::tray_icon::TrayIcon;

pub struct Popout {
    win: ApplicationWindow,
    container: gtk::Box,
    visible: bool
}

impl Popout {
    pub fn new(app: &Application, tray_icon: &mut TrayIcon) -> Popout {
        let win = ApplicationWindow::builder()
                .application(app)
                .default_width(320)
                .default_height(200)
                .title("Volume")
                .build();

        win.set_decorated(false);
        win.set_keep_above(true);
        win.set_skip_pager_hint(true);
        win.set_skip_taskbar_hint(true);
        win.set_type_hint(gtk::gdk::WindowTypeHint::PopupMenu);
        win.set_resizable(false);

        let (area, ori) = tray_icon.get_geometry();
        win.move_(area.x, area.y - 200);

        let container = gtk::builders::BoxBuilder::new()
            .margin(10)
            .spacing(6)
            .orientation(gtk::Orientation::Vertical)
            .build();

        win.set_child(Some(&container));

        let mut ret = Self {
            win,
            container,
            visible: false
        };

        ret.add_hide_on_loose_focus();
        ret.construct_ui();
        ret
    }

    fn add_hide_on_loose_focus(&mut self) {
        // self.win.connect_focus_out_event(|r, f| -> Inhibit {
        //     self.win.hide();
        //     self.visible = false;
        //     gtk::Inhibit(false)
        // });
    }

    fn construct_ui(&mut self) {
        // let button = gtk::Button::with_label("Click me!");
        // button.connect_clicked(|_| {
        //     eprintln!("Clicked!");
        // });
        // self.win.add(&button);


        // let slider = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
        // self.win.add(&slider);

        let _ = VolumeSlider::new(&self.container, Some("hi".to_string()),
        Rc::new(|d: f64| {
            println!("{:?}", d);
        }));
    }

    fn hide(&mut self) {
        self.win.hide();
        self.visible = false;
    }

    fn show(&mut self) {
        self.win.show_all();
        self.visible = true;
    }

    pub fn toggle_vis(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.win.show_all();
        } else {
            self.win.hide();
        }
    }

}