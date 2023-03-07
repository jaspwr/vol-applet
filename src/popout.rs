use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;

use gdk_sys::GdkRectangle;
use gtk::glib::idle_add_once;
use gtk::traits::{ContainerExt, GtkWindowExt, WidgetExt};
use gtk::{Application, ApplicationWindow, Inhibit};

use crate::audio::reload_outputs_in_popout;
use crate::audio::shared_output_list::{self, is_default_output};
use crate::elements::VolumeSlider;
use crate::tray_icon::TrayIcon;
use crate::{audio, AUDIO};

static POPOUT: Mutex<Option<Popout>> = Mutex::new(None);

pub struct Popout {
    pub container: gtk::Box,
    pub win: ApplicationWindow,
    pub sliders: HashMap<String, Box<VolumeSlider>>,
    visible: bool,
    geometry_last: Option<(GdkRectangle, i32)>,
    ignore_next_callback: bool,
}
unsafe impl Sync for Popout {}
unsafe impl Send for Popout {}

impl Popout {
    pub fn initialise(app: &Application) {
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

        let mut popout = Self {
            container,
            win,
            visible: false,
            geometry_last: None,
            sliders: HashMap::new(),
            ignore_next_callback: false,
        };

        popout.add_hide_on_loose_focus();

        POPOUT.lock().unwrap().replace(popout);
    }

    pub fn pub_set_geometry(area: GdkRectangle, ori: i32) {
        idle_add_once(move || {
            let mut a = POPOUT.lock().unwrap();
            let popout = a.as_mut().unwrap();
            popout.set_geomerty(area, ori);
        });
    }

    fn set_geomerty(&mut self, area: GdkRectangle, ori: i32) {
        let (width, height) = self.win.size();
        self.geometry_last = Some((area, ori));

        let (screen_wid, screen_hei) = (1920, 1080); // TODO
        let left = (area.x as f32) / (screen_wid as f32) < 0.5;
        let top = (area.y as f32) / (screen_hei as f32) < 0.5;

        if top && left {
            println!("up left");
            self.win.move_(area.x + area.width, area.y);
        } else if top && !left {
            println!("up right");
            self.win.move_(area.x - width, area.y);
        } else if !top && left {
            println!("down left");
            self.win.move_(area.x + area.width, area.y + area.height - height);
        } else if !top && !left {
            println!("down right");
            self.win.move_(area.x - width, area.y + area.height - height);
        }
    }

    pub fn handle_callback(f: fn(&mut Popout)) {
        let mut a = POPOUT.lock().unwrap();
        let popout = a.as_mut().unwrap();
        if popout.ignore_next_callback {
            popout.ignore_next_callback = false;
            return;
        }
        f(popout);
    }

    fn fix_window_position(&mut self) {
        if self.geometry_last.is_none() {
            return;
        }
        let (area, ori) = self.geometry_last.unwrap();
        self.set_geomerty(area, ori);
    }

    pub fn set_ignore_next_callback() {
        let mut a = POPOUT.lock().unwrap();
        let popout = a.as_mut().unwrap();
        popout.ignore_next_callback = true;
    }

    fn add_hide_on_loose_focus(&mut self) {
        self.win.connect_focus_out_event(|_, _| -> Inhibit {
            POPOUT.lock().unwrap().as_mut().unwrap().hide();
            gtk::Inhibit(false)
        });
    }

    pub fn set_specific_volume(output_id: String, volume: f32) {
        idle_add_once(move || {
            let mut a = POPOUT.lock().unwrap();
            let popout = a.as_mut().unwrap();
            popout
                .sliders
                .get(&output_id)
                .unwrap()
                .set_volume_slider(volume);
        });
    }

    pub fn set_specific_muted(output_id: String, muted: bool) {
        idle_add_once(move || {
            let mut a = POPOUT.lock().unwrap();
            let popout = a.as_mut().unwrap();
            popout.sliders.get(&output_id).unwrap().set_muted(muted);
        });
    }

    pub fn update_outputs() {
        idle_add_once(|| {
            let mut a = POPOUT.lock().unwrap();
            let popout = a.as_mut().unwrap();
            let container = popout.container.clone();

            remove_child_widgets(popout);

            add_outputs_from_list(popout, container);

            if popout.visible {
                popout.win.show_all();
            }

            popout.fix_window_position();
        });

        if let Ok(output) = shared_output_list::get_default_output() {
            TrayIcon::set_volume(output.volume);
        }
    }

    fn append_volume_slider(
        &self,
        container: &gtk::Box,
        output: audio::shared_output_list::Output,
        is_default: bool,
    ) -> VolumeSlider {
        let id = output.id.clone();
        let id_ = output.id.clone();
        VolumeSlider::new(
            container,
            Some(output.name),
            output.volume,
            output.muted,
            Rc::new(move |vol: f32| {
                handle_volume_slider_change(is_default, vol, id.clone());
            }),
            Rc::new(move || {
                handle_mute_button(id_.clone());
            }),
        )
    }

    fn hide(&mut self) {
        self.win.hide();
        self.visible = false;
    }

    fn show(&mut self) {
        AUDIO.lock().unwrap().aud.get_outputs(Box::new(
            |outputs: Vec<shared_output_list::Output>| {
                reload_outputs_in_popout(outputs);
            },
        ));

        self.fix_window_position();

        self.visible = true;
    }

    pub fn toggle_vis() {
        let mut a = POPOUT.lock().unwrap();
        let popout = a.as_mut().unwrap();
        if popout.visible {
            popout.hide();
        } else {
            popout.show();
        }
    }
}

fn add_outputs_from_list(popout: &mut Popout, container: gtk::Box) {
    let outputs = audio::shared_output_list::get_output_list();
    for output in outputs {
        let id = output.id.clone();
        popout.sliders.insert(
            output.id.clone(),
            Box::new(popout.append_volume_slider(&container, output, is_default_output(&id))),
        );
    }
}

fn remove_child_widgets(popout: &mut Popout) {
    popout.container.foreach(|w| {
        popout.container.remove(w);
    });
}

fn handle_volume_slider_change(is_default: bool, vol: f32, id: String) {
    if is_default {
        TrayIcon::set_volume(vol);
    }
    Popout::set_ignore_next_callback();

    AUDIO.lock().unwrap().aud.set_volume(id, vol);
}

fn handle_mute_button(id: String) {
    let mut list = shared_output_list::OUTPUT_LIST.lock().unwrap();
    let mut muted = false;
    Popout::set_ignore_next_callback();
    for output in list.iter_mut() {
        if output.id == id {
            muted = !output.muted;
            output.muted = muted;

            TrayIcon::check_if_shows_muted(output);

            break;
        }
    }
    Popout::set_specific_muted(id.clone(), muted);
    AUDIO.lock().unwrap().aud.set_muted(id, muted);
}
