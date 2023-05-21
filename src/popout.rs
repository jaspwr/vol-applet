use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;

use gtk::gdk::{ SeatCapabilities, EventKey };
use gtk::glib::idle_add_once;
use gtk::traits::{ ContainerExt, WidgetExt, GtkWindowExt };
use gtk::{ Application, ApplicationWindow, Inhibit };

use crate::audio::reload_outputs_in_popout;
use crate::audio::shared_output_list;
use crate::elements::VolumeSlider;
use crate::tray_icon::TrayIcon;
use crate::{ audio, AUDIO };

static POPOUT: Mutex<Option<Popout>> = Mutex::new(None);

pub struct Popout {
    pub container: gtk::Box,
    pub popout_menu: ApplicationWindow,
    pub sliders: HashMap<String, Box<VolumeSlider>>,
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
            .type_hint(gtk::gdk::WindowTypeHint::PopupMenu)
            .decorated(false)
            .resizable(false)
            .build();

        let container = gtk::Box
            ::builder()
            .margin(10)
            .spacing(6)
            .orientation(gtk::Orientation::Vertical)
            .build();

        win.set_child(Some(&container));

        win.connect_key_press_event(|_, e: &EventKey| -> Inhibit {
            if let Some(keycode) = e.keycode() {
                const ESC: u16 = 9;
                if keycode == ESC {
                    Popout::hide();
                }
            }
            gtk::Inhibit(false)
        });

        win.connect_button_press_event(|win, e| {
            let (x, y) = e.position();
            let (w, h) = win.size();
            if x < 0.0 || y < 0.0 || x > w as f64 || y > h as f64 {
                Popout::hide();
            }
            gtk::Inhibit(false)
        });

        win.connect_focus_in_event(|win, _| {
            grab_seat(&win.window().unwrap());
            gtk::Inhibit(false)
        });

        win.connect_focus_out_event(|_, _| {
            Popout::hide();
            gtk::Inhibit(false)
        });

        let popout = Self {
            container,
            popout_menu: win,
            sliders: HashMap::new(),
            ignore_next_callback: false,
        };

        POPOUT.lock().unwrap().replace(popout);
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

    fn set_geomerty(&mut self) {
        self.popout_menu.set_size_request(320, 50);
        let (window_x, window_y) = self.popout_menu.position();
        let (window_width, window_height) = self.popout_menu.size();

        let (icon, orientation) = TrayIcon::get_geometry();

        let display = self.popout_menu.display();
        let monitor = display.monitor_at_point(window_x, window_y).unwrap();
        let monitor = monitor.geometry();

        #[allow(unused)]
        let mut x = 0;
        #[allow(unused)]
        let mut y = 0;

        if orientation == 1 {
            if icon.x + icon.width + window_width <= monitor.x() + monitor.width() {
                x = icon.x + icon.width;
            } else {
                x = icon.x - window_width;
            }
            if icon.y + window_height <= monitor.y() + monitor.height() {
                y = icon.y;
            } else {
                y = monitor.y() + monitor.height() - window_height;
            }
        } else {
            if icon.y + icon.height + window_height <= monitor.y() + monitor.height() {
                y = icon.y + icon.height;
            } else {
                y = icon.y - window_height;
            }
            if icon.x + window_width <= monitor.x() + monitor.width() {
                x = icon.x;
            } else {
                x = monitor.x() + monitor.width() - window_width;
            }
        }

        self.popout_menu.move_(x, y);
    }

    pub fn set_ignore_next_callback() {
        let mut a = POPOUT.lock().unwrap();
        let popout = a.as_mut().unwrap();
        popout.ignore_next_callback = true;
    }

    pub fn set_specific_volume(output_id: String, volume: f32) {
        idle_add_once(move || {
            let mut a = POPOUT.lock().unwrap();
            let popout = a.as_mut().unwrap();
            popout.sliders.get(&output_id).unwrap().set_volume_slider(volume);
        });
    }

    pub fn set_specific_volume_label(output_id: String, volume: f32) {
        idle_add_once(move || {
            let mut a = POPOUT.lock().unwrap();
            let popout = a.as_mut().unwrap();
            popout.sliders.get(&output_id).unwrap().set_volume_label(volume);
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

            popout.container.show_all();
        });

        if let Ok(output) = shared_output_list::get_default_output() {
            TrayIcon::set_muted(output.muted);
            TrayIcon::set_volume(output.volume);
        }
    }

    fn append_volume_slider(
        &self,
        container: &gtk::Box,
        output: audio::shared_output_list::Output,
        is_default: bool
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
            })
        )
    }

    pub fn show() {
        AUDIO.lock()
            .unwrap()
            .aud.get_outputs(
                Box::new(|outputs: Vec<shared_output_list::Output>| {
                    reload_outputs_in_popout(outputs);
                })
            );

        let mut a = POPOUT.lock().unwrap();
        let popout = a.as_mut().unwrap();

        popout.popout_menu.show();
        popout.popout_menu.present();
        popout.set_geomerty();
    }

    pub fn hide() {
        let mut a = POPOUT.lock().unwrap();
        let popout = a.as_mut().unwrap();
        popout.popout_menu.hide();
        // ungrab(&popout.popout_menu.window().unwrap());
    }
}

fn add_outputs_from_list(popout: &mut Popout, container: gtk::Box) {
    let outputs = audio::shared_output_list::get_output_list();
    popout.sliders = HashMap::new();

    if outputs.is_empty() {
        popout.container.add(&gtk::Label::builder().label("No outputs found").build());
        return;
    }

    for output in outputs {
        let is_default = output.is_default();
        popout.sliders.insert(
            output.id.clone(),
            Box::new(popout.append_volume_slider(&container, output, is_default))
        );
    }
}

fn remove_child_widgets(popout: &mut Popout) {
    popout.container.foreach(|w| {
        popout.container.remove(w);
    });
}

fn handle_volume_slider_change(is_default: bool, vol: f32, id: String) {
    let vol = clamp_volume_to_percent(vol);

    if (vol - shared_output_list::get_stored_volume(&id)).abs() < 2. {
        return;
    }

    Popout::set_specific_volume_label(id.clone(), vol);

    if is_default {
        TrayIcon::set_volume(vol);
    }
    Popout::set_ignore_next_callback();

    AUDIO.lock().unwrap().aud.set_volume(id, vol);
}

fn clamp_volume_to_percent(vol: f32) -> f32 {
    if vol > 100. { 100. } else if vol < 0. { 0. } else { vol }
}

fn handle_mute_button(id: String) {
    let mut list = shared_output_list::OUTPUT_LIST.lock().unwrap();
    let mut muted = false;
    Popout::set_ignore_next_callback();
    for output in list.iter_mut() {
        if output.id == id {
            muted = !output.muted;
            output.muted = muted;
            if output.is_default() {
                TrayIcon::set_muted(muted);
            }
            break;
        }
    }
    Popout::set_specific_muted(id.clone(), muted);
    AUDIO.lock().unwrap().aud.set_muted(id, muted);
}

fn grab_seat(popout: &gtk::gdk::Window) {
    let display = popout.display();
    let seat = display.default_seat().unwrap();

    let capabilities = gdk_sys::GDK_SEAT_CAPABILITY_POINTER;

    let status = seat.grab(
        popout,
        unsafe {
            SeatCapabilities::from_bits_unchecked(capabilities)
        },
        true,
        None,
        None,
        None
    );

    if status != gtk::gdk::GrabStatus::Success {
        println!("Grab failed: {:?}", status);
    }
}

// fn ungrab(popout: &gtk::gdk::Window) {
//     let display = popout.display();
//     let seat = display.default_seat().unwrap();
//     seat.ungrab();
// }
