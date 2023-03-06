use super::Audio;

struct Pipewire {
}

impl Audio for Pipewire {
    fn get_outputs(&self) {
        todo!()
    }

    fn set_volume(&self, sink_id: String, volume: f32) {
        todo!()
    }

    fn set_muted(&self, sink_id: String, muted: bool) {
        todo!()
    }
}