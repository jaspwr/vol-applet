use std::{ffi::c_void, sync::Mutex, collections::HashMap};

use crate::{audio::{finish_output_list, shared_output_list}, AUDIO};


use super::Audio;

use libpulse_sys::*;
use once_cell::sync::Lazy;

static PA_CVOLUMES: Lazy<Mutex<HashMap<String, Box<pa_cvolume>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub struct Pulse {
    context: *mut pa_context, // TODO make option
    mainloop: *mut pa_threaded_mainloop,
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
            //      std::thread::sleep(std::time::Duration::from_millis(100));
            // }

        }

        Pulse {
            context,
            mainloop,
        }
    }
}

impl Audio for Pulse {
    fn get_outputs(&self) {
        unsafe {
            pa_context_get_sink_info_list(self.context, Some(sink_info_callback), std::ptr::null_mut());
        }
    }

    fn set_volume(&self, sink_id: String, volume: f32) {
        unsafe {
            // let cvol_ptr = &self.volume as *const pa_cvolume as *mut pa_cvolume; // LOL fuck off rust
            let cvol = **PA_CVOLUMES.lock().unwrap().get(&sink_id).unwrap();
            let cvol_ptr = &(cvol) as *const pa_cvolume as *mut pa_cvolume;

            pa_cvolume_set(cvol_ptr, cvol.channels as u32, (volume * 1000.) as u32);

            pa_context_set_sink_volume_by_name(
                self.context,
                sink_id.as_ptr() as *const i8,
                cvol_ptr,
                None,
                std::ptr::null_mut()
            );
        }
    }

    fn set_muted(&self, sink_id: String, muted: bool) {
        unsafe {
            pa_context_set_sink_mute_by_name(
                self.context,
                sink_id.as_ptr() as *const i8,
                muted as i32,
                None,
                std::ptr::null_mut()
            );
        }
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
extern "C" fn sink_info_callback(_: *mut pa_context, sink_info: *const pa_sink_info, eol: i32, _: *mut c_void) {
    if eol == 0 {
        let sink_info_ptr = sink_info as *mut pa_sink_info;
        
        let n = unsafe {
            let name_ptr = (*sink_info_ptr).name;
            let name = std::ffi::CStr::from_ptr(name_ptr);
            name.to_string_lossy().to_string()
        };

        let d = unsafe {
            let desc_ptr = (*sink_info_ptr).description;
            let desc = std::ffi::CStr::from_ptr(desc_ptr);
            desc.to_string_lossy().to_string()
        };

        let muted = unsafe {
            (*sink_info_ptr).mute != 0
        };

        let vol: f32 = unsafe {
            let v = (*sink_info_ptr).volume;
            PA_CVOLUMES.lock().unwrap().insert(n.clone(), Box::new(v));
            pa_cvolume_avg(&v) as f32 / 1000.
        };
        
        shared_output_list::add_output(
            d,
            vol,
            muted,
            n,
        );
    } else {
        // End of input
        finish_output_list();
    }
}

#[no_mangle]
pub extern "C" fn context_state_callback(context: *mut pa_context, _: *mut c_void) {
    unsafe {
        let state = pa_context_get_state(context);
        if state == PA_CONTEXT_READY {
            println!("PulseAudio context ready");
            AUDIO.lock().unwrap().aud.get_outputs();
            // pa_threaded_mainloop_signal(mainloop, 0);
        }
    }
}

impl Drop for Pulse {
    fn drop(&mut self) {
        self.cleanup();
    }
}