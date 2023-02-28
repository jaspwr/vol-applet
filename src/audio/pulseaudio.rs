use std::{rc::Rc, ffi::c_void};

use super::{Audio, AudioOutput};

use libpulse_sys::*;

pub struct Pulse {
    context: *mut pa_context,
    mainloop: *mut pa_threaded_mainloop,
    sinks: Vec<Rc<dyn AudioOutput>>
}

impl Pulse {
    pub fn new() -> Pulse {
        let mainloop = unsafe { pa_threaded_mainloop_new() };
        let mainloop_api = unsafe { pa_threaded_mainloop_get_api(mainloop) };
        let context = unsafe { pa_context_new(mainloop_api, std::ptr::null()) };
        unsafe {

            pa_context_connect(context, std::ptr::null(), 0, std::ptr::null_mut());
            pa_threaded_mainloop_start(mainloop);
            pa_context_set_state_callback(context, Some(context_state_callback), std::ptr::null_mut());
            // pa_threaded_mainloop_lock(mainloop);
            // while pa_context_get_state(context) != PA_CONTEXT_READY {

            //     pa_threaded_mainloop_wait(mainloop);
            //     // std::thread::sleep(std::time::Duration::from_millis(100));
            // }
            // pa_threaded_mainloop_unlock(mainloop);
        }

        Pulse {
            context,
            mainloop,
            sinks: Vec::new()
        }
    }
}

impl Audio for Pulse {
    fn get_outputs(&mut self) -> Vec<Rc<dyn super::AudioOutput>> {
        // let mut sinks: Vec<Rc<dyn super::AudioOutput>> = Vec::new();
        unsafe {
            pa_context_get_sink_info_list(self.context, Some(sink_info_callback), &mut self.sinks as *mut _ as *mut c_void);
        }
        self.sinks.clone()
    }

    fn cleanup(&mut self) {
        unsafe {
            pa_context_disconnect(self.context);
            pa_context_unref(self.context);
            pa_threaded_mainloop_stop(self.mainloop);
            pa_threaded_mainloop_free(self.mainloop);
        }
    }
}

#[no_mangle]
extern "C" fn sink_info_callback(_: *mut pa_context, sink_info: *const pa_sink_info, eol: i32, userdata: *mut c_void) {
    println!("FOUND SINK");
    
    if eol == 0 {
        let sink = Sink {
            sink_info: sink_info as *mut pa_sink_info
        };
        let sink = Rc::new(sink);
        let sinks = unsafe { &mut *(userdata as *mut Vec<Rc<dyn super::AudioOutput>>) };
        sinks.push(sink);
    }
}

struct Sink {
    sink_info: *mut pa_sink_info,
}

impl AudioOutput for Sink {
    fn get_volume(&self) -> f64 {
        unsafe {
            (*self.sink_info).volume.values[0] as f64
        }
    }

    fn set_volume(&mut self, volume: f64) {
        unsafe {
            let cvol_ptr: *mut pa_cvolume = &mut (*self.sink_info).volume;
            pa_cvolume_set(cvol_ptr, 1, volume as u32);
        }
    }

    fn get_muted(&self) -> bool {
        unsafe {
            (*self.sink_info).mute != 0
        }
    }

    fn set_muted(&mut self ,muted: bool) {
        unsafe {
            (*self.sink_info).mute = muted as i32;
        }
    }

    fn get_name(&self) -> String {
        unsafe {
            let name_ptr = (*self.sink_info).name;
            let name = std::ffi::CStr::from_ptr(name_ptr);
            name.to_string_lossy().to_string()
        }
    }
}

#[no_mangle]
pub extern "C" fn context_state_callback(context: *mut pa_context, userdata: *mut c_void) {
    unsafe {
        let state = pa_context_get_state(context);
        if state == PA_CONTEXT_READY {
            println!("PulseAudio context ready");
            // pa_threaded_mainloop_signal(mainloop, 0);
        }
    }
}