use std::{rc::Rc, sync::{Arc, Mutex}};

use crate::{popout::Popout, TRAY_ICON};

mod pulseaudio;

pub fn get_audio() -> Box<dyn Audio> {
    // TODO: pick audio backend based on config
    Box::new(pulseaudio::Pulse::new())
}

pub fn new_output(output: Rc<dyn AudioOutput>) {
    println!("New output: {}", output.get_name());

    let a = unsafe { TRAY_ICON.as_mut().unwrap() };
    let tray_icon = a.lock().unwrap();
    tray_icon.popout.lock().unwrap().outputs.push(output);
}

pub fn finish_output_list() {
    let a = unsafe { TRAY_ICON.as_mut().unwrap() };
    let tray_icon = a.lock().unwrap();
    let popout = tray_icon.popout.lock().unwrap();
    // popout.append_volume_slider_list(&popout.container);
}

pub trait AudioOutput {
    fn get_volume(&self) -> f64;
    fn set_volume(&self, volume: f64);
    fn get_muted(&self) -> bool;
    fn set_muted(&self ,muted: bool);
    fn get_name(&self) -> String;
}

pub trait Audio {
    fn get_outputs(&mut self) -> Vec<Rc<dyn AudioOutput>>;
    fn cleanup(&mut self) {}
}