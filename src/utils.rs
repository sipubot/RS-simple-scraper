use chrono::prelude::*;
use serde_json::{json, Value};
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use bytes::Bytes;


pub fn logger(_log: &str) {
    let filename = Utc::now().format("%Y-%m").to_string();
    let utc = Utc::now().format("%Y-%m-%d  %H:%M:%S").to_string();
    let pathstr = format!("./log/log{}.log", filename);

    if !path_exist(&pathstr) {
        if !Path::new("./log").exists() {
            fs::create_dir_all("./log").unwrap();
        }
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

pub fn path_exist(_path: &str) -> bool {
    let path = Path::new(&_path);
    path.exists()
}

pub fn make_file(path: &str, file_name: &str, bin: &Bytes) {
    if !path_exist(path) {
        fs::create_dir_all(path).unwrap();
    }
    let mut file : std::fs::File = std::fs::File::create(format!(r"{}\{}",&path,&file_name)).unwrap();
    file.write_all(bin).unwrap();
}

pub fn json_result(_res: &str) -> Value {
    json!({ "result": _res })
}

pub fn file_read_to_json(_filepath: &str) -> serde_json::Result<Value> {
    let pathstring = _filepath;
    match fs::read_to_string(&pathstring) {
        Err(e) => {
            logger(&e.to_string());
            Ok(json_result(&e.to_string()))
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

pub async fn get_text_response(_url: &str) -> String {
    reqwest::get(_url).await.unwrap().text().await.unwrap()
}

pub async fn get_byte_response(_url: &str, reffer:&str) -> Bytes {
    let client = reqwest::Client::new();
    client.get(_url).header("Referer", reffer).send().await.unwrap().bytes().await.unwrap()
}

pub async fn get_text_response_bot(_url: &str) -> String {
    static APP_USER_AGENT: &str = concat!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:98.0) Gecko/20100101 Firefox/98.0"
    );
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_static("secret"));
    let client = reqwest::Client::builder().user_agent(APP_USER_AGENT).default_headers(headers).build().unwrap();

    client.get(&*_url).send().await.unwrap().text().await.unwrap()
}
