use chrono::prelude::*;
use serde_json::{json, Value};
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn logger(_log: &str) {
    let filename = Utc::now().format("%Y-%m").to_string();
    let utc = Utc::now().format("%Y-%m-%d  %H:%M:%S").to_string();
    let pathstr = format!("./log/log{}.log", filename);
    if !folder_exist(&pathstr) {
        File::create(&pathstr).unwrap();
    }
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(pathstr)
        .unwrap();

    if let Err(e) = writeln!(file, "[{}]:{}\n", utc, _log) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

pub fn json_re_parse(_res: &str) -> Value {
    json!({ "result": _res })
}

pub fn folder_exist(_path: &str) -> bool {
    let path = Path::new(&_path);
    path.exists()
}

pub fn file_read_to_json(_filepath: &str) -> serde_json::Result<Value> {
    let pathstring = _filepath;
    match fs::read_to_string(&pathstring) {
        Err(e) => {
            logger(&e.to_string());
            Ok(json_re_parse(&e.to_string()))
        }
        Ok(file) => serde_json::from_str(&*file),
    }
}

pub fn file_save_from_json(_filepath: &str, _v: &Value) -> serde_json::Result<bool> {
    let path = Path::new(&_filepath);
    let json = serde_json::to_string(_v).unwrap();
    match File::create(&path) {
        Err(e) => {
            logger(&e.to_string());
            Ok(false)
        }
        Ok(mut file) => match file.write_all(&json.as_bytes()) {
            Err(e) => {
                logger(&e.to_string());
                Ok(false)
            }
            Ok(_) => Ok(true),
        },
    }
}

