use std::rc::Rc;
use std::sync::{Arc, Mutex};

use gdk_sys::GdkRectangle;
use gtk::traits::{GtkWindowExt, WidgetExt, ButtonExt, ContainerExt, ProgressBarExt};
use gtk::{Application, ApplicationWindow, Inhibit, Container};

use crate::audio::{AudioOutput, Audio, get_audio};
use crate::elements::VolumeSlider;
use crate::tray_icon::TrayIcon;

pub struct Popout {
    pub container: gtk::Box,
    pub outputs: Vec<Rc<dyn AudioOutput>>,
    win: ApplicationWindow,
    visible: bool,
    audio: Box<dyn Audio>
}

impl Popout {
    pub fn new(app: &Application) -> Popout {
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

        let container = gtk::builders::BoxBuilder::new()
            .margin(10)
            .spacing(6)
            .orientation(gtk::Orientation::Vertical)
            .build();

        win.set_child(Some(&container));

        let audio = get_audio();

        let mut ret = Self {
            container,
            outputs: Vec::new(),
            win,
            visible: false,
            audio
        };

        ret.add_hide_on_loose_focus();
        ret
    }

    pub fn set_geomerty(&mut self, area: GdkRectangle, ori: i32) {
        self.win.move_(area.x, area.y - 200);
    }

    fn add_hide_on_loose_focus(&mut self) {
        // self.win.connect_focus_out_event(|r, f| -> Inhibit {
        //     self.win.hide();
        //     self.visible = false;
        //     gtk::Inhibit(false)
        // });
    }



    pub fn append_volume_slider_list(&self, container: &gtk::Box) {
        println!("h {:?}", self.outputs.len());
        self.container.foreach(|w| {
            self.container.remove(w);
        });
        for output in &self.outputs {
            println!("fuck you {:?}", output.get_name());
            self.append_volume_slider(container, output.clone());
        }
    }

    pub fn append_volume_slider(&self, container: &gtk::Box, audio_output: Rc<dyn AudioOutput>) -> VolumeSlider {
        let label = audio_output.get_name();
        let vol = audio_output.get_volume();
        let mut slider = VolumeSlider::new(&container, Some(label), vol,
        Rc::new(move |d: f64| {
            audio_output.set_volume(d);
        }));
        // slider.set_bar(vol);
        slider
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
            self.outputs.clear();
            self.audio.get_outputs();
            self.win.show_all();
        } else {
            self.append_volume_slider_list(&self.container);

//            self.win.hide();
        }
    }

}