use std::process::{Command, Child};
use thirtyfour::prelude::*;
use tokio;
use std::time::Duration;
 use std::path::PathBuf;

use crate::FireFoxPath;

/// Geckodriver를 Firefox 바이너리 경로와 함께 실행
fn start_geckodriver_with_binary(path: &FireFoxPath) -> Result<Child, Box<dyn std::error::Error>> {
    let gecko_path = PathBuf::from(path.geckodriver_path.as_str());
    let child = Command::new(gecko_path)
        .arg("--port")
        .arg("4444")    
        .arg("--binary")
        .arg(path.path.as_str())
        .spawn()?; // 백그라운드 실행
    std::thread::sleep(Duration::from_secs(2)); // 포트 준비 대기
    Ok(child)
}

/// Firefox로 페이지 크롤링
pub async fn get_html(url: &str, path: &FireFoxPath) -> Result<String, Box<dyn std::error::Error>> {
    // Geckodriver 실행 (Firefox 경로 포함)
    let mut gecko = start_geckodriver_with_binary(path)?;

    // WebDriver 연결
    let caps = DesiredCapabilities::firefox();
    let driver = WebDriver::new("http://localhost:4444", caps).await?;

    // 페이지 이동
    driver.get(url).await?;

    // body 태그 로딩 대기
    let body = driver.query(By::Tag("body")).first().await?;

    // 스크롤 실행
    driver.execute(r#"
        async function scrollDown() {
            let distance = 100;
            let delay = ms => new Promise(res => setTimeout(res, ms));
            while ((window.innerHeight + window.scrollY) < document.body.scrollHeight) {
                window.scrollBy(0, distance);
                await delay(100);
            }
            return true;
        }
        return scrollDown();
    "#, vec![]).await?;

    tokio::time::sleep(Duration::from_secs(5)).await;

    // HTML 추출
    let html = body.inner_html().await?;
    driver.quit().await?;

    // Geckodriver 종료
    gecko.kill()?;

    Ok(html)
}
