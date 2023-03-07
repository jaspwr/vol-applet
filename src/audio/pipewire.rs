use super::{shared_output_list, Audio};

struct Pipewire {}

#[allow(unused_variables)] // Remove once implemented
impl Audio for Pipewire {
    fn get_outputs(&self, after: Box<dyn Fn(Vec<shared_output_list::Output>) + 'static>) {
        todo!()
    }

    fn set_volume(&self, sink_id: String, volume: f32) {
        todo!()
    }

    fn set_muted(&self, sink_id: String, muted: bool) {
        todo!()
    }
}
