use std::rc::Rc;

use gtk::{glib::idle_add_once, traits::WidgetExt};

use crate::TRAY_ICON;

mod pulseaudio;
pub mod shared_output_list;

pub fn get_audio() -> Rc<dyn Audio> {
    // TODO: pick audio backend based on config
    Rc::new(pulseaudio::Pulse::new())
}

pub fn finish_output_list() {
    idle_add_once(|| {
        println!("Finished output list");

        let a = unsafe { TRAY_ICON.as_mut().unwrap() };
        let tray_icon = a.lock().unwrap();
        let mut popout = tray_icon.popout.lock().unwrap();
        let container = popout.container.clone();
        popout.update_outputs(&container);
        popout.win.show_all();
        // glib::Continue(false)
    });

    // TODO
}

pub trait Audio {
    fn get_outputs(&self);
    fn set_volume(&self, sink_id: String, volume: f64);
    fn set_muted(&self, sink_id: String, muted: bool);

    fn cleanup(&mut self) {}
}