use std::{
    collections::HashMap,
    ffi::{c_char, c_void},
    sync::{Arc, Mutex},
};

use super::{shared_output_list::VolumeType, Audio};
use crate::{
    audio::{
        get_audio, reload_outputs_in_popout,
        shared_output_list::{self, set_default_output},
    },
    exception::Exception,
    options::OPTIONS,
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

static NAME: &[u8] = b"volapplet\0";

impl Pulse {
    pub fn new() -> Pulse {
        let mainloop = unsafe { pa_threaded_mainloop_new() };
        let mainloop_api = unsafe { pa_threaded_mainloop_get_api(mainloop) };
        let context = unsafe { pa_context_new(mainloop_api, NAME.as_ptr() as *const c_char) };

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
            let op = pa_context_get_server_info(
                self.context,
                Some(server_info_callback),
                std::ptr::null_mut(),
            );

            if op.is_null() {
                Exception::Misc("Failed to get PA server info.".to_string()).log_and_ignore();
            } else {
                pa_operation_unref(op);
            }
        }
    }
}

struct GetSinkListUserdata {
    final_callback: Mutex<Box<dyn Fn(Vec<shared_output_list::Output>) + 'static>>,
    unfinished_callbacks: Mutex<u32>,
    call_id: u32,
    list: Mutex<Vec<shared_output_list::Output>>,
}

impl Audio for Pulse {
    fn get_outputs(&self, after: Box<dyn Fn(Vec<shared_output_list::Output>) + 'static>) {
        self.get_server_info();

        *GET_SINKS_CALLBACK_ID.lock().unwrap() += 1;

        let mut unfinished_callbacks: u32 = 1;

        if OPTIONS.show_inputs {
            unfinished_callbacks += 1;
        }

        if OPTIONS.show_streams {
            unfinished_callbacks += 1;
        }

        let userdata = Arc::new(GetSinkListUserdata {
            final_callback: Mutex::new(after),
            unfinished_callbacks: Mutex::new(unfinished_callbacks),
            call_id: *GET_SINKS_CALLBACK_ID.lock().unwrap(),
            list: Mutex::new(vec![]),
        });

        unsafe {
            let op = pa_context_get_sink_info_list(
                self.context,
                Some(sink_info_callback),
                Arc::into_raw(userdata.clone()) as *mut c_void,
            );

            if !op.is_null() {
                pa_operation_unref(op);
            }

            if OPTIONS.show_inputs {
                let op = pa_context_get_source_info_list(
                    self.context,
                    Some(source_info_callback),
                    Arc::into_raw(userdata.clone()) as *mut c_void,
                );

                if op.is_null() {
                    Exception::Misc("Failed to get source list.".to_string()).log_and_ignore();
                } else {
                    pa_operation_unref(op);
                }
            }

            if OPTIONS.show_streams {
                let op = pa_context_get_sink_input_info_list(
                    self.context,
                    Some(sink_input_info_callback),
                    Arc::into_raw(userdata) as *mut c_void,
                );

                if op.is_null() {
                    Exception::Misc("Failed to get sink input list.".to_string()).log_and_ignore();
                } else {
                    pa_operation_unref(op);
                }
            }
        }
    }

    fn set_volume(&self, sink_id: String, volume: f32, type_: VolumeType) {
        let volume = clamp_volume(volume);

        unsafe {
            pa_threaded_mainloop_lock(self.mainloop);

            let cvol = **PA_CVOLUMES.lock().unwrap().get(&sink_id).unwrap();
            let cvol_ptr = &cvol as *const pa_cvolume as *mut pa_cvolume;

            pa_cvolume_set(cvol_ptr, cvol.channels as u32, (volume * 1000.) as u32);

            let idx = shared_output_list::get_pa_index(&sink_id).unwrap();

            let op = match type_ {
                VolumeType::Sink => pa_context_set_sink_volume_by_index(
                    self.context,
                    idx,
                    cvol_ptr,
                    None,
                    std::ptr::null_mut(),
                ),
                VolumeType::Input => pa_context_set_source_volume_by_index(
                    self.context,
                    idx,
                    cvol_ptr,
                    None,
                    std::ptr::null_mut(),
                ),
                VolumeType::Stream => pa_context_set_sink_input_volume(
                    self.context,
                    idx,
                    cvol_ptr,
                    None,
                    std::ptr::null_mut(),
                ),
            };

            if op.is_null() {
                Exception::Misc("Failed to get PA server info.".to_string()).log_and_ignore();
            } else {
                pa_operation_unref(op);
            }

            pa_threaded_mainloop_unlock(self.mainloop);
        }
    }

    fn set_muted(&self, sink_id: String, muted: bool, type_: VolumeType) {
        unsafe {
            pa_threaded_mainloop_lock(self.mainloop);

            let idx = shared_output_list::get_pa_index(&sink_id).unwrap();

            let op = match type_ {
                VolumeType::Sink => pa_context_set_sink_mute_by_index(
                    self.context,
                    idx,
                    muted as i32,
                    None,
                    std::ptr::null_mut(),
                ),
                VolumeType::Input => pa_context_set_source_mute_by_index(
                    self.context,
                    idx,
                    muted as i32,
                    None,
                    std::ptr::null_mut(),
                ),
                VolumeType::Stream => pa_context_set_sink_input_mute(
                    self.context,
                    idx,
                    muted as i32,
                    None,
                    std::ptr::null_mut(),
                ),
            };

            if op.is_null() {
                Exception::Misc("Failed to set muted.".to_string()).log_and_ignore();
            } else {
                pa_operation_unref(op);
            }

            pa_threaded_mainloop_unlock(self.mainloop);
        }
    }

    fn cleanup(&mut self) {
        unsafe {
            if !self.context.is_null() {
                pa_context_disconnect(self.context);
                pa_context_unref(self.context);
            }

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
    let userdata = unsafe { Arc::from_raw(userdata as *mut GetSinkListUserdata) };

    if userdata.call_id != *GET_SINKS_CALLBACK_ID.lock().unwrap() {
        if eol == 0 {
            // Leak userdata again
            let _ = Arc::into_raw(userdata);
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

        let pa_index = unsafe { (*sink_info_ptr).index };

        update_list(
            &userdata,
            name,
            volume,
            muted,
            output_id,
            pa_index,
            None,
            VolumeType::Sink,
        );
        // Leak userdata again
        let _ = Arc::into_raw(userdata);
    } else {
        // End of input
        try_finish_callback(userdata);
    }
}

fn try_finish_callback(userdata: Arc<GetSinkListUserdata>) {
    let vec = userdata.list.lock().unwrap();
    let mut unfinished_callbacks = userdata.unfinished_callbacks.lock().unwrap();
    *unfinished_callbacks -= 1;
    if *unfinished_callbacks == 0 {
        userdata.final_callback.lock().unwrap()(vec.to_vec());
    }
}

#[derive(PartialEq)]
enum SourceType {
    Hardware,
    Virtual,
    Monitor,
}

#[no_mangle]
extern "C" fn source_info_callback(
    _: *mut pa_context,
    source_info: *const pa_source_info,
    eol: i32,
    userdata: *mut c_void,
) {
    let userdata = unsafe { Arc::from_raw(userdata as *mut GetSinkListUserdata) };

    if userdata.call_id != *GET_SINKS_CALLBACK_ID.lock().unwrap() {
        if eol == 0 {
            // Leak userdata again
            let _ = Arc::into_raw(userdata);
        }
        return;
    }

    if eol == 0 {
        let source_info_ptr = source_info as *mut pa_source_info;

        let source_type = if PA_INVALID_INDEX != unsafe { (*source_info_ptr).monitor_of_sink } {
            SourceType::Monitor
        } else {
            if unsafe { (*source_info_ptr).flags } & PA_SOURCE_HARDWARE != 0 {
                SourceType::Hardware
            } else {
                SourceType::Virtual
            }
        };

        if source_type == SourceType::Monitor {
            // Leak userdata again
            let _ = Arc::into_raw(userdata);
            return;
        }

        let output_id = unsafe {
            let name_ptr = (*source_info_ptr).name;
            let name = std::ffi::CStr::from_ptr(name_ptr);
            name.to_string_lossy().to_string()
        };

        let name = unsafe {
            let desc_ptr = (*source_info_ptr).description;
            let desc = std::ffi::CStr::from_ptr(desc_ptr);
            desc.to_string_lossy().to_string()
        };

        let muted = unsafe { (*source_info_ptr).mute != 0 };

        let volume: f32 = unsafe {
            let v = (*source_info_ptr).volume;
            PA_CVOLUMES
                .lock()
                .unwrap()
                .insert(output_id.clone(), Box::new(v));
            (pa_cvolume_avg(&v) as f32) / 1000.
        };

        let pa_index = unsafe { (*source_info_ptr).index };

        update_list(
            &userdata,
            name,
            volume,
            muted,
            output_id,
            pa_index,
            None,
            VolumeType::Input,
        );
        // Leak userdata again
        let _ = Arc::into_raw(userdata);
    } else {
        // End of input
        try_finish_callback(userdata);
    }
}

#[no_mangle]
extern "C" fn sink_input_info_callback(
    _: *mut pa_context,
    sink_info: *const pa_sink_input_info,
    eol: i32,
    userdata: *mut c_void,
) {
    let userdata = unsafe { Arc::from_raw(userdata as *mut GetSinkListUserdata) };

    if userdata.call_id != *GET_SINKS_CALLBACK_ID.lock().unwrap() {
        if eol == 0 {
            // Leak userdata again
            let _ = Arc::into_raw(userdata);
        }
        return;
    }

    if eol == 0 {
        let sink_info_ptr = sink_info as *mut pa_sink_input_info;

        let output_id = unsafe {
            let name_ptr = (*sink_info_ptr).name;
            let name = std::ffi::CStr::from_ptr(name_ptr);
            name.to_string_lossy().to_string()
        };

        let name = unsafe {
            let desc_ptr = (*sink_info_ptr).name;
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

        let pa_index = unsafe { (*sink_info_ptr).index };

        let icon_name = unsafe {
            get_icon_name(sink_info_ptr).map(|ico_name_ptr| {
                let icon_name = std::ffi::CStr::from_ptr(ico_name_ptr);
                icon_name.to_string_lossy().to_string()
            })
        };

        update_list(
            &userdata,
            name,
            volume,
            muted,
            output_id,
            pa_index,
            icon_name,
            VolumeType::Stream,
        );
        // Leak userdata again
        let _ = Arc::into_raw(userdata);
    } else {
        // End of input
        try_finish_callback(userdata);
    }
}

unsafe fn get_icon_name(sink_info_ptr: *mut pa_sink_input_info) -> Option<*const i8> {
    static PA_PROP_MEDIA_ICON_NAME_: &[u8] = b"media.icon_name\0";
    static PA_PROP_WINDOW_ICON_NAME_: &[u8] = b"window.icon_name\0";
    static PA_PROP_APPLICATION_ICON_NAME_: &[u8] = b"application.icon_name\0";

    let proplist_ptr = (*sink_info_ptr).proplist;

    if let Some(value) = try_get_icon(proplist_ptr, PA_PROP_MEDIA_ICON_NAME_.as_ptr() as *const i8)
    {
        return Some(value);
    }

    if let Some(value) = try_get_icon(
        proplist_ptr,
        PA_PROP_WINDOW_ICON_NAME_.as_ptr() as *const i8,
    ) {
        return Some(value);
    }

    if let Some(value) = try_get_icon(
        proplist_ptr,
        PA_PROP_APPLICATION_ICON_NAME_.as_ptr() as *const i8,
    ) {
        return Some(value);
    };

    None
}

unsafe fn try_get_icon(proplist_ptr: *mut pa_proplist, key: *const c_char) -> Option<*const i8> {
    let ico_name_ptr = pa_proplist_gets(proplist_ptr, key);

    if !ico_name_ptr.is_null() {
        return Some(ico_name_ptr);
    }
    None
}

fn update_list(
    userdata: &Arc<GetSinkListUserdata>,
    name: String,
    volume: f32,
    muted: bool,
    output_id: String,
    pa_index: u32,
    icon_name: Option<String>,
    type_: VolumeType,
) {
    let mut list = userdata.list.lock().unwrap();
    list.push(shared_output_list::Output {
        name,
        volume,
        muted,
        id: output_id,
        pa_index: Some(pa_index),
        icon_name,
        type_,
    });
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

            let mut flags = PA_SUBSCRIPTION_MASK_SINK;

            if OPTIONS.show_icons {
                flags |= PA_SUBSCRIPTION_MASK_SINK_INPUT;
            }

            if OPTIONS.show_inputs {
                flags |= PA_SUBSCRIPTION_MASK_SOURCE;
            }

            let op = pa_context_subscribe(context, flags, None, std::ptr::null_mut());
            if op.is_null() {
                Exception::Misc("PulseAudio context subscription failed".to_string())
                    .log_and_ignore();
            } else {
                pa_operation_unref(op);
            }

            AUDIO.lock().unwrap().aud.get_outputs(Box::new(
                |outputs: Vec<shared_output_list::Output>| {
                    reload_outputs_in_popout(outputs);
                },
            ));
        } else if state == PA_CONTEXT_FAILED {
            Exception::Misc("Failed to connect to PulseAudio (PA_CONTEXT_FAILED).".to_string())
                .log_and_ignore();
            retry_connection_loop();
        } else if state == PA_CONTEXT_TERMINATED {
            Exception::Misc("Disconnected from PulseAudio (PA_CONTEXT_TERMINATED)".to_string())
                .log_and_ignore();
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
    let event_type = event_type & PA_SUBSCRIPTION_EVENT_FACILITY_MASK;

    if event_type == PA_SUBSCRIPTION_EVENT_SINK
        || event_type == PA_SUBSCRIPTION_EVENT_SINK_INPUT
        || event_type == PA_SUBSCRIPTION_EVENT_SOURCE
    {
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

fn clamp_volume(vol: f32) -> f32 {
    if vol > 100. {
        100.
    } else if vol < 0. {
        0.
    } else {
        vol
    }
}
