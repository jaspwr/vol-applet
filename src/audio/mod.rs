use std::{rc::Rc, sync::{Arc, Mutex}};

use gtk::{glib::idle_add_once, traits::WidgetExt};

use crate::{popout::Popout, TRAY_ICON};

mod pulseaudio;
pub mod shared_output_list;

pub fn get_audio() -> Rc<dyn Audio> {
    // TODO: pick audio backend based on config
    Rc::new(pulseaudio::Pulse::new())
}

// pub fn new_output(output: Rc<dyn AudioOutput>) {
//     println!("New output: {}", output.get_name());

//     let a = unsafe { TRAY_ICON.as_mut().unwrap() };
//     let tray_icon = a.lock().unwrap();
//     tray_icon.popout.lock().unwrap().outputs.push(output);
// }

pub fn finish_output_list() {
    idle_add_once(|| {
        println!("Finished output list");

        let a = unsafe { TRAY_ICON.as_mut().unwrap() };
        let tray_icon = a.lock().unwrap();
        let popout = tray_icon.popout.lock().unwrap();
        popout.update_outputs(&popout.container);
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