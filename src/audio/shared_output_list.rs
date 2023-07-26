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
    pub pa_index: Option<u32>,
    pub icon_name: Option<String>,
    pub type_: VolumeType,
}

#[derive(Clone)]
pub enum VolumeType {
    Sink,
    Stream,
    Input,
}

pub fn get_output_list() -> Vec<Output> {
    let output_list = OUTPUT_LIST.lock().unwrap();
    output_list.clone()
}

fn is_default_output(output_id: &String) -> bool {
    *output_id == DEFAULT_OUTPUT_ID.lock().unwrap().to_string()
}

impl Output {
    pub fn is_default(&self) -> bool {
        is_default_output(&self.id)
    }
}

pub fn set_default_output(output_id: String) {
    *DEFAULT_OUTPUT_ID.lock().unwrap() = output_id;
}

pub fn get_pa_index(output_id: &String) -> Option<u32> {
    let output_list = OUTPUT_LIST.lock().unwrap();

    for output in output_list.iter() {
        if output.id == *output_id {
            return output.pa_index;
        }
    }
    None
}

pub fn type_of(output_id: &String) -> VolumeType {
    let output_list = OUTPUT_LIST.lock().unwrap();

    for output in output_list.iter() {
        if output.id == *output_id {
            return output.type_.clone();
        }
    }
    VolumeType::Sink
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

pub fn get_stored_volume(output_id: &String) -> f32 {
    let output_list = OUTPUT_LIST.lock().unwrap();

    for output in output_list.iter() {
        if output.id == *output_id {
            return output.volume;
        }
    }
    0.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_list() {
        let mut list = OUTPUT_LIST.lock().unwrap();
        list.push(Output {
            name: "Headphones".to_string(),
            volume: 23.0,
            muted: false,
            id: "1".to_string(),
            pa_index: None,
            icon_name: None,
            type_: VolumeType::Sink,
        });
        list.push(Output {
            name: "Speakers".to_string(),
            volume: 77.0,
            muted: true,
            id: "2".to_string(),
            pa_index: None,
            icon_name: None,
            type_: VolumeType::Sink,
        });
        list.push(Output {
            name: "Microphone".to_string(),
            volume: 22.0,
            muted: false,
            id: "3".to_string(),
            pa_index: None,
            icon_name: None,
            type_: VolumeType::Input,
        });
        drop(list);

        assert_eq!(get_output_list().len(), 3);

        set_default_output("2".to_string());

        let default = get_default_output().unwrap();
        assert!(default.is_default());
        assert_eq!(default.name, "Speakers");
        assert_eq!(default.volume, 77.0);
        assert_eq!(default.muted, true);
        assert_eq!(default.id, "2");
    }
}
