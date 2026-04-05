use serde_json::{json, Value};
use std::path::Path;
use std::time::Duration;
use bytes::Bytes;
use lazy_static::lazy_static;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::AsyncWriteExt;
use anyhow::{Result, Context};
use chrono::Utc;
use std::cmp::Reverse;

const MAX_LOG_FILES: usize = 10;
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB
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

pub async fn logger(_log: &str) {
    let filename = Utc::now().format("%Y-%m").to_string();
    let utc = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_dir = "./log";
    let pathstr = format!("{}/log{}.log", log_dir, filename);

    // 로그 디렉토리 생성
    if !Path::new(log_dir).exists() {
        if let Err(e) = fs::create_dir_all(log_dir).await {
            eprintln!("Failed to create log directory: {}", e);
            return;
        }
    }

    // 파일 용량 체크
    if let Ok(meta) = fs::metadata(&pathstr).await {
        if meta.len() >= MAX_FILE_SIZE {
            // 새 파일 이름에 타임스탬프 추가
            let new_filename = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
            let new_path = format!("{}/log{}.log", log_dir, new_filename);
            if let Err(e) = File::create(&new_path).await {
                eprintln!("Failed to create new log file: {}", e);
                return;
            }
        }
    }

    // 최근 10개 파일만 유지
    if let Ok(mut entries) = fs::read_dir(log_dir).await {
        let mut files = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let modified = entry.metadata().await
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            files.push((entry, modified));
        }

        // 수정 시간 기준 정렬
        files.sort_by_key(|(_, modified)| Reverse(*modified));

        if files.len() > MAX_LOG_FILES {
            for (entry, _) in files.iter().skip(MAX_LOG_FILES) {
                if let Err(e) = fs::remove_file(entry.path()).await {
                    eprintln!("Failed to remove old log file: {}", e);
                }
            }
        }
    }

    // 로그 쓰기
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
