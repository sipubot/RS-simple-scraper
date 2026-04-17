use serde_json::{json, Value};
use std::path::Path;
use std::time::Duration;
use bytes::Bytes;
use lazy_static::lazy_static;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use anyhow::{Result, Context};
// ⭐ log 매크로 사용 (flexi_logger가 이 로그들을 받아 처리합니다)
use log::{info, warn, error};

// HTTP client configuration constants
const HTTP_TIMEOUT_SECS: u64 = 30;
const POOL_MAX_IDLE_PER_HOST: usize = 10;
const APP_USER_AGENT: &str = "RS Simple Scraper/1.0";
const BOT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:98.0) Gecko/20100101 Firefox/98.0";

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .pool_max_idle_per_host(POOL_MAX_IDLE_PER_HOST)
        .user_agent(APP_USER_AGENT)
        .build()
        .expect("Failed to create HTTP client");

    static ref HTTP_CLIENT_BOT: reqwest::Client = {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_static("secret"));
        reqwest::Client::builder()
            .user_agent(BOT_USER_AGENT)
            .default_headers(headers)
            .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
            .pool_max_idle_per_host(POOL_MAX_IDLE_PER_HOST)
            .build()
            .expect("Failed to create bot HTTP client")
    };
}

/// ⭐ 기존의 복잡한 수동 로깅 함수를 대체합니다.
/// 이제 이 함수를 호출하는 대신 직접 log::info! 등을 사용해도 됩니다.
pub fn logger(msg: &str) {
    info!("{}", msg);
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
            // ⭐ 수동 logger() 대신 log 매크로 사용
            error!("Error reading {}: {}", _filepath, e);
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
                    warn!("Failed to get text from {}: {}", _url, e);
                    String::new()
                }
            }
        },
        Err(e) => {
            warn!("Failed to request {}: {}", _url, e);
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
                    warn!("Failed to get bytes from {}: {}", _url, e);
                    Bytes::new()
                }
            }
        },
        Err(e) => {
            warn!("Failed to request bytes from {}: {}", _url, e);
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
                    warn!("Failed to get bot text from {}: {}", _url, e);
                    String::new()
                }
            }
        },
        Err(e) => {
            warn!("Failed to request bot text from {}: {}", _url, e);
            String::new()
        }
    }
}