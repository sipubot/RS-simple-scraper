use chrono::prelude::*;
use serde_json::{json, Value};
use std::path::Path;
use std::time::Duration;
use bytes::Bytes;
use lazy_static::lazy_static;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::AsyncWriteExt;
use anyhow::{Result, Context};

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(10)
        .user_agent("RS Simple Scraper/1.0")
        .build()
        .expect("Failed to create HTTP client");

    static ref HTTP_CLIENT_BOT: reqwest::Client = {
        static APP_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:98.0) Gecko/20100101 Firefox/98.0";
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_static("secret"));
        reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .expect("Failed to create bot HTTP client")
    };
}

pub async fn logger(_log: &str) {
    let filename = Utc::now().format("%Y-%m").to_string();
    let utc = Utc::now().format("%Y-%m-%d  %H:%M:%S").to_string();
    let log_dir = "./log";
    let pathstr = format!("{}/log{}.log", log_dir, filename);

    if !Path::new(log_dir).exists() {
        let _ = fs::create_dir_all(log_dir).await;
    }

    let mut file = match OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&pathstr)
        .await {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open log file {}: {}", pathstr, e);
                return;
            }
        };

    if let Err(e) = file.write_all(format!("[{}]:{}\n", utc, _log).as_bytes()).await {
        eprintln!("Couldn't write to log file: {}", e);
    }
}

pub fn path_exist(_path: &str) -> bool {
    Path::new(_path).exists()
}

pub async fn make_file(path: &str, file_name: &str, bin: &Bytes) -> Result<()> {
    if !path_exist(path) {
        fs::create_dir_all(path).await.context(format!("Failed to create directory: {}", path))?;
    }
    let file_path = format!("{}/{}", path, file_name);
    let mut file = File::create(&file_path).await.context(format!("Failed to create file: {}", file_path))?;
    file.write_all(bin).await.context(format!("Failed to write to file: {}", file_path))?;
    Ok(())
}

pub fn json_result(_res: &str) -> Value {
    json!({ "result": _res })
}

pub async fn file_read_to_json(_filepath: &str) -> Result<Value> {
    match fs::read_to_string(_filepath).await {
        Ok(content) => {
            serde_json::from_str(&content).context("Failed to parse JSON")
        }
        Err(e) => {
            logger(&format!("Error reading {}: {}", _filepath, e)).await;
            Ok(json_result(&e.to_string()))
        }
    }
}

pub async fn file_save_from_json(_filepath: &str, _v: &Value) -> Result<()> {
    let json = serde_json::to_string(_v).context("Failed to serialize JSON")?;
    let mut file = File::create(_filepath).await.context(format!("Failed to create file: {}", _filepath))?;
    file.write_all(json.as_bytes()).await.context(format!("Failed to write to file: {}", _filepath))?;
    Ok(())
}

pub async fn get_text_response(_url: &str) -> String {
    match HTTP_CLIENT.get(_url).send().await {
        Ok(resp) => {
            match resp.text().await {
                Ok(result) => result,
                Err(e) => {
                    logger(&format!("Failed to get text from {}: {}", _url, e)).await;
                    String::new()
                }
            }
        },
        Err(e) => {
            logger(&format!("Failed to request {}: {}", _url, e)).await;
            String::new()
        }
    }
}

pub async fn get_byte_response(_url: &str, reffer: &str) -> Bytes {
    match HTTP_CLIENT.get(_url).header("Referer", reffer).send().await {
        Ok(resp) => {
            match resp.bytes().await {
                Ok(bin) => bin,
                Err(e) => {
                    logger(&format!("Failed to get bytes from {}: {}", _url, e)).await;
                    Bytes::new()
                }
            }
        },
        Err(e) => {
            logger(&format!("Failed to request bytes from {}: {}", _url, e)).await;
            Bytes::new()
        }
    }
}

pub async fn get_text_response_bot(_url: &str) -> String {
    match HTTP_CLIENT_BOT.get(_url).send().await {
        Ok(resp) => {
            match resp.text().await {
                Ok(result) => result,
                Err(e) => {
                    logger(&format!("Failed to get bot text from {}: {}", _url, e)).await;
                    String::new()
                }
            }
        },
        Err(e) => {
            logger(&format!("Failed to request bot text from {}: {}", _url, e)).await;
            String::new()
        }
    }
}
