use std::{rc::Rc, ffi::c_void, sync::{Mutex, Arc}};

use crate::{audio::{new_output, finish_output_list}, popout::Popout};

use std::alloc::GlobalAlloc;

use super::{Audio, AudioOutput};

use libpulse_sys::*;

use std::alloc::System;

static TEMP_SINK_LIST_MUTEX: Mutex<Vec<u64>> = Mutex::new(Vec::new());

pub struct Pulse {
    context: *mut pa_context, // TODO make option
    mainloop: *mut pa_threaded_mainloop,
    sinks: Vec<Rc<dyn AudioOutput>>,
}

struct Sink {
    context: *mut pa_context,
    sink_info: *mut pa_sink_info,
    volume: pa_cvolume,
    name: String,
    description: String,
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
            sinks: Vec::new(),
        }
    }
}

impl Audio for Pulse {
    fn get_outputs(&mut self) -> Vec<Rc<dyn super::AudioOutput>> {
        // let mut sinks: Vec<Rc<dyn super::AudioOutput>> = Vec::new();
        //TEMP_SINK_LIST_MUTEX.lock().unwrap().clear();
        unsafe {
            pa_context_get_sink_info_list(self.context, Some(sink_info_callback), std::ptr::null_mut());
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
extern "C" fn sink_info_callback(context: *mut pa_context, sink_info: *const pa_sink_info, eol: i32, userdata: *mut c_void) {

    
    if eol == 0 {
        println!("FOUND SINK");
        let sink_info_ptr = sink_info as *mut pa_sink_info;
        let mut n = String::new();
        let mut d = String::new();
        unsafe {
            let name_ptr = (*sink_info_ptr).name;
            let name = std::ffi::CStr::from_ptr(name_ptr);
            n = name.to_string_lossy().to_string();
            let desc_ptr = (*sink_info_ptr).description;
            let desc = std::ffi::CStr::from_ptr(desc_ptr);
            d = desc.to_string_lossy().to_string();
        }
        new_output(Rc::new(Sink {
            context,
            sink_info: sink_info_ptr,
            name: n,
            description: d,
            volume: unsafe { (*sink_info_ptr).volume },
        }));
    } else {
        finish_output_list();
        // println!("{:?}", TEMP_SINK_LIST_MUTEX.lock().unwrap());
        println!("END OF SINKS");
    }
}

impl AudioOutput for Sink {
    fn get_volume(&self) -> f64 {
        unsafe {
            pa_cvolume_avg(&self.volume as *const pa_cvolume)as f64 / 1000.
        }
    }

    fn set_volume(&self, volume: f64) {
        unsafe {
            let cvol_ptr = &self.volume as *const pa_cvolume as *mut pa_cvolume; // LOL fuck off rust
            
            pa_cvolume_set(cvol_ptr, self.volume.channels as u32, (volume * 1000.) as u32);
            // pa_cvolume_set(cvol_ptr, 3, volume as u32);
            // pa_cvolume_set(cvol_ptr, 1, volume as u32);

            pa_context_set_sink_volume_by_name(
                self.context,
                self.name.as_ptr() as *const i8,
                cvol_ptr,
                None,
                std::ptr::null_mut()
            );
        }
    }

    fn get_muted(&self) -> bool {
        unsafe {
            (*self.sink_info).mute != 0
        }
    }

    fn set_muted(&self ,muted: bool) {
        unsafe {
            (*self.sink_info).mute = muted as i32;
        }
    }

    fn get_name(&self) -> String {
        self.description.clone()
    }
    
}


// #[no_mangle]
// pub extern "C" fn nothing_callback(context: *mut pa_context, success: i32, userdata: *mut c_void) {
//     println!("nothing callback");
// }

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

impl Drop for Pulse {
    fn drop(&mut self) {
        self.cleanup();
    }
}