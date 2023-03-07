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
    pub id: String,
}

pub fn get_output_list() -> Vec<Output> {
    let output_list = OUTPUT_LIST.lock().unwrap();
    output_list.clone()
}

pub fn is_default_output(output_id: &String) -> bool {
    *output_id == DEFAULT_OUTPUT_ID.lock().unwrap().to_string()
}

pub fn set_default_output(output_id: String) {
    *DEFAULT_OUTPUT_ID.lock().unwrap() = output_id;
}

pub fn get_default_output() -> Result<Output, Exception> {
    let output_list = OUTPUT_LIST.lock().unwrap();

    for output in output_list.iter() {
        if output.id == *DEFAULT_OUTPUT_ID.lock().unwrap() {
            return Ok(output.clone());
        }
    }
    Err(Exception::Misc("No default output found".to_string()))
}