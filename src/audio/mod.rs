use std::sync::Arc;

use crate::popout::Popout;

mod pipewire;
mod pulseaudio;
pub mod shared_output_list;

unsafe impl Send for WrappedAudio {}
unsafe impl Sync for WrappedAudio {}
pub struct WrappedAudio {
    pub aud: Arc<dyn Audio>,
}

pub fn get_audio() -> WrappedAudio {
    // TODO: pick audio backend based on config
    WrappedAudio {
        aud: Arc::new(pulseaudio::Pulse::new()),
    }
}

pub fn reload_outputs_in_popout(outputs: Vec<shared_output_list::Output>) {
    *shared_output_list::OUTPUT_LIST.lock().unwrap() = outputs;
    Popout::update_outputs();
}

pub trait Audio {
    fn get_outputs(&self, after: Box<dyn Fn(Vec<shared_output_list::Output>) + 'static>);
    fn set_volume(&self, sink_id: String, volume: f32);
    fn set_muted(&self, sink_id: String, muted: bool);

    fn cleanup(&mut self) {}
}
