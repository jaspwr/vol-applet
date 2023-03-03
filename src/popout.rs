use std::rc::Rc;

use gdk_sys::GdkRectangle;
use gtk::traits::{GtkWindowExt, WidgetExt, ContainerExt};
use gtk::{Application, ApplicationWindow, Inhibit};

use crate::{audio, AUDIO, POPOUT};
use crate::elements::VolumeSlider;

pub struct Popout {
    pub container: GtkBoxWrapper,
    pub win: ApplicationWindowWrapper,
    visible: bool,
    geometry_last: Option<(GdkRectangle, i32)>
}

unsafe impl Sync for GtkBoxWrapper {}
unsafe impl Send for GtkBoxWrapper {}
pub struct GtkBoxWrapper {
    pub container: gtk::Box
}

unsafe impl Sync for ApplicationWindowWrapper {}
unsafe impl Send for ApplicationWindowWrapper {}
pub struct ApplicationWindowWrapper {
    pub win: ApplicationWindow
}

impl Popout {
    pub fn new(app: &Application) -> Popout {
        let win = ApplicationWindow::builder()
                .application(app)
                .default_width(320)
                .default_height(50)
                .title("Volume")
                .build();

        win.set_decorated(false);
        win.set_keep_above(true);
        win.set_skip_pager_hint(true);
        win.set_skip_taskbar_hint(true);
        win.set_type_hint(gtk::gdk::WindowTypeHint::PopupMenu);
        win.set_resizable(false);

        let container = gtk::builders::BoxBuilder::new()
            .margin(10)
            .spacing(6)
            .orientation(gtk::Orientation::Vertical)
            .build();

        win.set_child(Some(&container));

        let mut ret = Self {
            container: GtkBoxWrapper { container },
            win: ApplicationWindowWrapper { win },
            visible: false,
            geometry_last: None
        };


        ret.add_hide_on_loose_focus();
        ret
    }

    pub fn set_geomerty(&mut self, area: GdkRectangle, ori: i32) {
        let (width, height) = self.win.win.size();
        self.geometry_last = Some((area, ori));

        println!("{} {} {} {} {} {} ori {}", area.x, area.y, area.width, width, height,  area.height, ori);

        let (screen_wid, screen_hei) = (1920, 1080); // TODO
        let left = (area.x as f32 / screen_wid as f32) > 0.5;
        let top = (area.y as f32 / screen_hei as f32) > 0.5;

        if top && left {
            self.win.win.move_(area.x - width, area.y - height);
        } else if top && !left {
            self.win.win.move_(area.x + area.width, area.y - height);
        } else if !top && left {
            self.win.win.move_(area.x - width, area.y + area.height);
        } else if !top && !left {
            self.win.win.move_(area.x + area.width, area.y + area.height);
        }
    }

    pub fn fix_window_position(&mut self) {
        if self.geometry_last.is_none() { return; }
        let (area, ori) = self.geometry_last.unwrap();
        self.set_geomerty(area, ori);
    }

    fn add_hide_on_loose_focus(&mut self) {
        self.win.win.connect_focus_out_event(|r, f| -> Inhibit {
            POPOUT.lock().unwrap().as_mut().unwrap().hide();
            gtk::Inhibit(false)
        });
    }

    pub fn update_outputs(&mut self, container: &gtk::Box) {

        self.container.container.foreach(|w| {
            self.container.container.remove(w);
        });

        let outputs = audio::shared_output_list::get_output_list();

        for output in outputs {
            self.append_volume_slider(container, output);
        }


        self.win.win.show_all();

        self.fix_window_position();
    }

    pub fn append_volume_slider(&self, 
        container: &gtk::Box,
        output: audio::shared_output_list::Output) -> VolumeSlider {

        let id = output.output_id.clone();
        let id_ = output.output_id.clone();
        println!("{} {} {} {}", output.name, output.volume, output.muted, output.output_id);
        let mut slider = VolumeSlider::new(container, 
            Some(output.name), output.volume, output.muted,
            Rc::new(move |vol: f64| {
                AUDIO.lock().unwrap().aud.set_volume(id.clone(), vol);
            }),
            Rc::new(move |mute: bool| {
                AUDIO.lock().unwrap().aud.set_muted(id_.clone(), mute);
            })
        );
        // slider.set_bar(vol);
        slider
    }

    fn hide(&mut self) {
        self.win.win.hide();
        self.visible = false;
    }

    fn show(&mut self) {
        audio::shared_output_list::clear_output_list();
        AUDIO.lock().unwrap().aud.get_outputs();
        // self.win.win.show_all();
        // self.win.win.emit_grab_focus();
        // self.win.win.activate();
        // self.win.win.activate_focus();
        // self.win.win.grab_focus();
        self.fix_window_position();
        self.visible = true;
    }

    pub fn toggle_vis(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }

}