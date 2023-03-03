
use std::sync::Arc;

use gtk::glib::idle_add_once;

use crate::POPOUT;

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
    idle_add_once(|| {
        println!("Finished output list");
        let mut a = POPOUT.lock().unwrap();
        let popout = a.as_mut().unwrap();
        let container = popout.container.container.clone();
        popout.update_outputs(&container);
        
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