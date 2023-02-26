use gtk::traits::{GtkWindowExt, WidgetExt};
use gtk::{Application, ApplicationWindow, Inhibit};

use crate::tray_icon::TrayIcon;

pub struct Popout {
    win: ApplicationWindow,
    visible: bool
}

impl Popout {
    pub fn new(app: &Application, tray_icon: &mut TrayIcon) -> Popout {
        let mut win = ApplicationWindow::builder()
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

        let mut ret = Self {
            win,
            visible: false
        };
        ret.add_hide_on_loose_focus();
        ret
    }

    fn add_hide_on_loose_focus(&mut self) {
        // self.win.connect_focus_out_event(|r, f| -> Inhibit {
        //     self.win.hide();
        //     self.visible = false;
        //     gtk::Inhibit(false)
        // });
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