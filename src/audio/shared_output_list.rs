use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::exception::Exception;

pub static OUTPUT_LIST: Lazy<Mutex<Vec<Output>>> = Lazy::new(|| Mutex::new(vec![]));
pub static DEFAULT_OUTPUT_ID: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("".to_string()));

#[derive(Clone)]
pub struct Output {
    pub name: String,
    pub volume: f32,
    pub muted: bool,
    pub output_id: String,
}

pub fn clear_output_list() {
    let mut output_list = OUTPUT_LIST.lock().unwrap();
    output_list.clear();
}

pub fn add_output(name: String, volume: f32, muted: bool, output_id: String) {
    let mut output_list = OUTPUT_LIST.lock().unwrap();
    output_list.push(Output { name, volume, muted, output_id });
}

pub fn get_output_list() -> Vec<Output> {
    let output_list = OUTPUT_LIST.lock().unwrap();
    output_list.clone()
}

pub fn is_default_output(output_id: String) -> bool {
    // TODO: Don't just return first.
    let output_list = OUTPUT_LIST.lock().unwrap();
    return output_id == output_list.first().unwrap().output_id;

    output_id == DEFAULT_OUTPUT_ID.lock().unwrap().to_string()
}

pub fn get_default_output() -> Result<Output, Exception> {
    let output_list = OUTPUT_LIST.lock().unwrap();
    
    // TODO: Don't just return first.
    return output_list.first().ok_or(Exception::Misc("No default output found".to_string())).map(|o| o.clone());
    
    for output in output_list.iter() {
        if output.output_id == *DEFAULT_OUTPUT_ID.lock().unwrap() {
            return Ok(output.clone());
        }
    }
    Err(Exception::Misc("No default output found".to_string()))
}