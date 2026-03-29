use serde_json::Value;
use std::path::Path;
use std::time::Duration;
use bytes::Bytes;
use lazy_static::lazy_static;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use anyhow::{Result, Context};
use tracing::info;

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
    // 기존의 수동 파일 쓰기 로직 대신 tracing 매크로 사용
    // 이 로그가 어디로 갈지(파일, 콘솔 등)는 main.rs의 subscriber 설정에 따릅니다.
    info!("{}", _log);
}

pub fn path_exist(_path: &str) -> bool {
    Path::new(_path).exists()
}

pub async fn make_file(path: &str, file_name: &str, bin: &Bytes) -> Result<()> {
    if !path_exist(path) {
        fs::create_dir_all(path).await.context(format!("Failed to create directory: {}", path))?;
    }
    let file_path = Path::new(path).join(file_name);
    let mut file = File::create(&file_path).await.context(format!("Failed to create file: {:?}", file_path))?;
    file.write_all(bin).await.context(format!("Failed to write to file: {:?}", file_path))?;
    Ok(())
}

pub async fn file_read_to_json(_filepath: &str) -> Result<Value> {
    let content = fs::read_to_string(_filepath).await
        .with_context(|| format!("Failed to read file: {}", _filepath))?;
    
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON from: {}", _filepath))
}

pub async fn file_save_from_json(_filepath: &str, _v: &Value) -> Result<()> {
    let json = serde_json::to_string(_v).context("Failed to serialize JSON")?;
    let mut file = File::create(_filepath).await.context(format!("Failed to create file: {}", _filepath))?;
    file.write_all(json.as_bytes()).await.context(format!("Failed to write to file: {}", _filepath))?;
    Ok(())
}

pub async fn get_text_response(_url: &str) -> Result<String> {
    let resp = HTTP_CLIENT.get(_url).send().await
        .with_context(|| format!("Failed to request URL: {}", _url))?;
    resp.text().await.context("Failed to get text content")
}

/// Result를 반환하는 버전 - 에러 전파를 위해 사용
pub async fn get_byte_response_result(_url: &str, reffer: &str) -> Result<Bytes> {
    let resp = HTTP_CLIENT.get(_url)
        .header("Referer", reffer)
        .send().await
        .with_context(|| format!("Failed to request bytes URL: {}", _url))?;
    resp.bytes().await.context("Failed to get bytes content")
}

pub async fn get_text_response_bot(_url: &str) -> Result<String> {
    let resp = HTTP_CLIENT_BOT.get(_url).send().await
        .with_context(|| format!("Failed to request bot URL: {}", _url))?;
    resp.text().await.context("Failed to get bot text content")
}
