use std::sync::Mutex;

use once_cell::sync::Lazy;

pub static OUTPUT_LIST: Lazy<Mutex<Vec<Output>>> = Lazy::new(|| Mutex::new(vec![]));

#[derive(Clone)]
pub struct Output {
    pub name: String,
    pub volume: f64,
    pub muted: bool,
    pub output_id: String,
}

pub fn clear_output_list() {
    let mut output_list = OUTPUT_LIST.lock().unwrap();
    output_list.clear();
}

pub fn add_output(name: String, volume: f64, muted: bool, output_id: String) {
    let mut output_list = OUTPUT_LIST.lock().unwrap();
    output_list.push(Output { name, volume, muted, output_id });
}

pub fn get_output_list() -> Vec<Output> {
    let output_list = OUTPUT_LIST.lock().unwrap();
    output_list.clone()
}