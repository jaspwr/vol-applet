use std::rc::Rc;
use std::sync::{Arc, Mutex};

use gdk_sys::GdkRectangle;
use gtk::traits::{GtkWindowExt, WidgetExt, ButtonExt, ContainerExt, ProgressBarExt};
use gtk::{Application, ApplicationWindow, Inhibit, Container};

use crate::{TRAY_ICON, audio};
use crate::audio::{Audio, get_audio};
use crate::elements::VolumeSlider;
use crate::tray_icon::TrayIcon;

pub struct Popout {
    pub container: gtk::Box,
    pub win: ApplicationWindow,
    visible: bool,
    audio: Rc<dyn Audio>
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

        let audio = get_audio();

        let mut ret = Self {
            container,
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

    pub fn update_outputs(&self, container: &gtk::Box) {

        self.container.foreach(|w| {
            self.container.remove(w);
        });

        let outputs = audio::shared_output_list::get_output_list();
        println!("h {:?}", outputs.len());

        for output in outputs {
            self.append_volume_slider(container, output);
        }
    }

    pub fn append_volume_slider(&self, 
        container: &gtk::Box,
        output: audio::shared_output_list::Output) -> VolumeSlider {

        let id = output.output_id.clone();
        let id_ = output.output_id.clone();
        let aud = self.audio.clone();
        let aud_ = self.audio.clone();
        println!("{} {} {} {}", output.name, output.volume, output.muted, output.output_id);
        let mut slider = VolumeSlider::new(&container, 
            Some(output.name), output.volume, output.muted,
            Rc::new(move |vol: f64| {
                aud.set_volume(id.clone(), vol);
            }),
            Rc::new(move |mute: bool| {
                aud_.set_muted(id_.clone(), mute);
            })
    );
        // slider.set_bar(vol);
        slider
    }

    fn hide(&mut self) {
        self.win.hide();
        self.visible = false;
    }

    fn show(&mut self) {
        audio::shared_output_list::clear_output_list();
        self.audio.get_outputs();
        self.win.show_all();
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