use std::{collections::HashMap, ffi::c_void, sync::Mutex};

use super::Audio;
use crate::{
    audio::{
        reload_outputs_in_popout,
        shared_output_list::{self, set_default_output}, get_audio,
    },
    exception::Exception,
    popout::Popout,
    tray_icon::TrayIcon,
    AUDIO,
};

use libpulse_sys::*;
use once_cell::sync::Lazy;

static PA_CVOLUMES: Lazy<Mutex<HashMap<String, Box<pa_cvolume>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static GET_SINKS_CALLBACK_ID: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(0));
static IN_RECONNECT_LOOP: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

#[allow(unused)] // TODO: Clean up unused
pub struct Pulse {
    context: *mut pa_context,
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
            pa_context_set_state_callback(
                context,
                Some(context_state_callback),
                std::ptr::null_mut(),
            );
        }

        Pulse { context, mainloop }
    }

    fn get_server_info(&self) {
        unsafe {
            pa_context_get_server_info(
                self.context,
                Some(server_info_callback),
                std::ptr::null_mut(),
            );
        }
    }
}

struct GetSinkListUserdata {
    final_callback: Box<dyn Fn(Vec<shared_output_list::Output>) + 'static>,
    call_id: u32,
    list: Mutex<Vec<shared_output_list::Output>>,
}

impl Audio for Pulse {
    fn get_outputs(&self, after: Box<dyn Fn(Vec<shared_output_list::Output>) + 'static>) {
        self.get_server_info();

        *GET_SINKS_CALLBACK_ID.lock().unwrap() += 1;

        let userdata = Box::new(GetSinkListUserdata {
            final_callback: after,
            call_id: *GET_SINKS_CALLBACK_ID.lock().unwrap(),
            list: Mutex::new(vec![]),
        });

        unsafe {
            pa_context_get_sink_info_list(
                self.context,
                Some(sink_info_callback),
                Box::into_raw(userdata) as *mut c_void,
            );
        }
    }

    fn set_volume(&self, sink_id: String, volume: f32) {
        unsafe {
            let cvol = **PA_CVOLUMES.lock().unwrap().get(&sink_id).unwrap();
            let cvol_ptr = &cvol as *const pa_cvolume as *mut pa_cvolume;

            pa_cvolume_set(cvol_ptr, cvol.channels as u32, (volume * 1000.) as u32);

            pa_context_set_sink_volume_by_name(
                self.context,
                sink_id.as_ptr() as *const i8,
                cvol_ptr,
                None,
                std::ptr::null_mut(),
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
                std::ptr::null_mut(),
            );
        }
    }

    fn cleanup(&mut self) {
        unsafe {
            pa_context_disconnect(self.context);
            pa_context_unref(self.context);
            // pa_threaded_mainloop_stop(self.mainloop);
            // pa_threaded_mainloop_free(self.mainloop);
        }
    }
}

#[no_mangle]
extern "C" fn sink_info_callback(
    _: *mut pa_context,
    sink_info: *const pa_sink_info,
    eol: i32,
    userdata: *mut c_void,
) {
    let mut userdata = unsafe { Box::from_raw(userdata as *mut GetSinkListUserdata) };

    if userdata.call_id != *GET_SINKS_CALLBACK_ID.lock().unwrap() {
        if eol == 0 {
            // Leak userdata again
            Box::into_raw(userdata);
        }
        return;
    }

    if eol == 0 {
        let sink_info_ptr = sink_info as *mut pa_sink_info;

        let output_id = unsafe {
            let name_ptr = (*sink_info_ptr).name;
            let name = std::ffi::CStr::from_ptr(name_ptr);
            name.to_string_lossy().to_string()
        };

        let name = unsafe {
            let desc_ptr = (*sink_info_ptr).description;
            let desc = std::ffi::CStr::from_ptr(desc_ptr);
            desc.to_string_lossy().to_string()
        };

        let muted = unsafe { (*sink_info_ptr).mute != 0 };

        let volume: f32 = unsafe {
            let v = (*sink_info_ptr).volume;
            PA_CVOLUMES
                .lock()
                .unwrap()
                .insert(output_id.clone(), Box::new(v));
            (pa_cvolume_avg(&v) as f32) / 1000.
        };

        {
            let mut list = userdata.list.lock().unwrap();
            list.push(shared_output_list::Output {
                name,
                volume,
                muted,
                id: output_id,
            });
        }
        // Leak userdata again
        Box::into_raw(userdata);
    } else {
        // End of input
        let vec = userdata.list.lock().unwrap();
        userdata.final_callback.as_mut()(vec.to_vec());
    }
}

#[no_mangle]
pub extern "C" fn context_state_callback(context: *mut pa_context, _: *mut c_void) {
    unsafe {
        let state = pa_context_get_state(context);
        if state == PA_CONTEXT_READY {
            *IN_RECONNECT_LOOP.lock().unwrap() = false;
            pa_context_set_subscribe_callback(
                context,
                Some(subscribe_callback),
                std::ptr::null_mut(),
            );

            let o = pa_context_subscribe(
                context,
                PA_SUBSCRIPTION_MASK_SINK,
                None,
                std::ptr::null_mut(),
            );
            if o.is_null() {
                Exception::Misc("PulseAudio context subscription failed".to_string())
                    .log_and_ignore();
            }
            pa_operation_unref(o);

            AUDIO.lock().unwrap().aud.get_outputs(Box::new(
                |outputs: Vec<shared_output_list::Output>| {
                    reload_outputs_in_popout(outputs);
                },
            ));

            // pa_threaded_mainloop_signal(mainloop, 0);
        } else if state == PA_CONTEXT_FAILED {
            Exception::Misc("Failed to connect to PulseAudio (PA_CONTEXT_FAILED).".to_string()).log_and_ignore();
            retry_connection_loop();
        } else if state == PA_CONTEXT_TERMINATED {
            Exception::Misc("Disconnected from PulseAudio (PA_CONTEXT_TERMINATED)".to_string()).log_and_ignore();
            retry_connection_loop();
        }
    }
}

#[no_mangle]
pub extern "C" fn subscribe_callback(
    _: *mut pa_context,
    event_type: pa_subscription_event_type_t,
    _: u32,
    _: *mut c_void,
) {
    if (event_type & PA_SUBSCRIPTION_EVENT_FACILITY_MASK) == PA_SUBSCRIPTION_EVENT_SINK {
        Popout::handle_callback(|_| {
            AUDIO.lock().unwrap().aud.get_outputs(Box::new(
                |outputs: Vec<shared_output_list::Output>| {
                    sink_change_subscription_event_handler(outputs);
                },
            ));
        });
    }
}

#[no_mangle]
pub extern "C" fn server_info_callback(
    _: *mut pa_context,
    server_info: *const pa_server_info,
    _: *mut c_void,
) {
    unsafe {
        let default_sink_name = (*server_info).default_sink_name;
        let default_sink_name = std::ffi::CStr::from_ptr(default_sink_name);
        let default_sink_name = default_sink_name.to_string_lossy().to_string();
        set_default_output(default_sink_name);
    }
}

fn sink_change_subscription_event_handler(outputs: Vec<shared_output_list::Output>) {
    let mut old_outputs = shared_output_list::OUTPUT_LIST.lock().unwrap();
    if outputs.len() != old_outputs.len() {
        drop(old_outputs);
        reload_outputs_in_popout(outputs);
    } else {
        for (i, output) in outputs.iter().enumerate() {
            if output.volume != old_outputs[i].volume {
                old_outputs[i].volume = output.volume;
                Popout::set_specific_volume(output.id.clone(), output.volume);

                if output.is_default() {
                    TrayIcon::set_volume(output.volume);
                }
            }
            if output.muted != old_outputs[i].muted {
                old_outputs[i].muted = output.muted;
                Popout::set_specific_muted(output.id.clone(), output.muted);
                if output.is_default() {
                    TrayIcon::set_muted(output.muted);
                }
            }
        }
    }
}

fn retry_connection_loop() {
    if *IN_RECONNECT_LOOP.lock().unwrap() {
        return;
    }
    *IN_RECONNECT_LOOP.lock().unwrap() = true;
    let mut loop_counter = 0;
    while loop_counter < 40 {
        println!("Retrying connection...");
        *AUDIO.lock().unwrap() = get_audio();
        std::thread::sleep(std::time::Duration::from_secs(5));
        if !*IN_RECONNECT_LOOP.lock().unwrap() {
            return;
        }
        loop_counter += 1;
    }
    Exception::Misc("PulseAudio context not found.".to_string()).log_and_exit();
}

impl Drop for Pulse {
    fn drop(&mut self) {
        self.cleanup();
    }
}
