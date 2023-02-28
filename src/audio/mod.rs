use std::rc::Rc;

mod pulseaudio;

pub fn get_audio() -> Box<dyn Audio> {
    // TODO: pick audio backend based on config
    Box::new(pulseaudio::Pulse::new())
}

pub trait AudioOutput {
    fn get_volume(&self) -> f64;
    fn set_volume(&mut self, volume: f64);
    fn get_muted(&self) -> bool;
    fn set_muted(&mut self ,muted: bool);
    fn get_name(&self) -> String;
}

pub trait Audio {
    fn get_outputs(&mut self) -> Vec<Rc<dyn AudioOutput>>;
    fn cleanup(&mut self) {}
}