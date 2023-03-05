
use std::sync::Arc;


use crate::{popout::Popout};

mod pulseaudio;
pub mod shared_output_list;

unsafe impl Send for WrappedAudio {}
unsafe impl Sync for WrappedAudio {}
pub struct WrappedAudio {
    pub aud: Arc<dyn Audio>
}

pub fn get_audio() -> WrappedAudio {
    // TODO: pick audio backend based on config
    WrappedAudio {
        aud: Arc::new(pulseaudio::Pulse::new())
    }   
}

pub fn finish_output_list() {
    Popout::update_outputs();
}

pub trait Audio {
    fn get_outputs(&self);
    fn set_volume(&self, sink_id: String, volume: f32);
    fn set_muted(&self, sink_id: String, muted: bool);

    fn cleanup(&mut self) {}
}