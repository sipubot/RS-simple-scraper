use thirtyfour::prelude::*;
use std::time::Duration;
use anyhow::{Result, Context};

// WebDriver configuration constants
const WEBDRIVER_URL: &str = "http://localhost:4444";
const BODY_LOAD_TIMEOUT_SECS: u64 = 10;
const BODY_LOAD_POLL_MS: u64 = 500;
const SCROLL_DISTANCE: i32 = 200;
const MAX_SCROLL_COUNT: i32 = 50;
const SCROLL_DELAY_MS: u64 = 50;
const CONTENT_LOAD_WAIT_SECS: u64 = 3;
const IMAGE_LOAD_TIMEOUT_SECS: u64 = 5;

/// Firefox로 페이지 크롤링 (레이지 로딩 지원)
pub async fn get_html(url: &str) -> Result<String> {
    // WebDriver 연결
    let caps = DesiredCapabilities::firefox();
    let driver = WebDriver::new(WEBDRIVER_URL, caps).await
        .context(format!("Failed to connect to WebDriver at {}", WEBDRIVER_URL))?;

    // 페이지 이동
    driver.goto(url).await.context(format!("Failed to navigate to URL: {}", url))?;

    // body 태그 로딩 대기
    let body = driver
        .query(By::Tag("body"))
        .wait(Duration::from_secs(BODY_LOAD_TIMEOUT_SECS), Duration::from_millis(BODY_LOAD_POLL_MS))
        .first()
        .await
        .context(format!("Failed to find body tag within {} seconds", BODY_LOAD_TIMEOUT_SECS))?;

    // 스마트 스크롤 - 필요한 만큼만 스크롤
    driver.execute(r#"
        async function smartScroll() {
            let distance = arguments[0];
            let maxScrolls = arguments[1];
            let delay = ms => new Promise(res => setTimeout(res, ms));
            let scrollCount = 0;

            while ((window.innerHeight + window.scrollY) < document.body.scrollHeight && scrollCount < maxScrolls) {
                window.scrollBy(0, distance);
                scrollCount++;
                await delay(arguments[2]);
            }
            return scrollCount;
        }
        return smartScroll(arguments[0], arguments[1], arguments[2]);
    "#, vec![
        serde_json::json!(SCROLL_DISTANCE),
        serde_json::json!(MAX_SCROLL_COUNT),
        serde_json::json!(SCROLL_DELAY_MS)
    ]).await.context("Failed to execute smart scroll script")?;

    // 이미지 레이지 로딩 완료 대기
    wait_for_images(&driver).await?;

    // 추가 동적 콘텐츠 로딩 대기
    tokio::time::sleep(Duration::from_secs(CONTENT_LOAD_WAIT_SECS)).await;

    // HTML 추출
    let html = body.inner_html().await.context("Failed to extract inner HTML from body")?;

    // 브라우저 종료
    if let Err(e) = driver.quit().await {
        eprintln!("Warning: Failed to quit WebDriver cleanly: {}", e);
    }

    Ok(html)
}

/// 모든 이미지의 레이지 로딩 완료 대기
async fn wait_for_images(driver: &WebDriver) -> Result<()> {
    // data-src 속성을 가진 이미지들이 모두 로드될 때까지 대기
    let timeout = Duration::from_secs(IMAGE_LOAD_TIMEOUT_SECS);
    let start = std::time::Instant::now();
    
    loop {
        let result = driver.execute(r#"
            // 모든 이미지가 로드되었는지 확인
            const images = document.querySelectorAll('img');
            const pendingImages = [];
            
            for (const img of images) {
                // data-src가 있지만 src가 비어있거나 placeholder인 경우
                const dataSrc = img.getAttribute('data-src') || img.getAttribute('data-original');
                const currentSrc = img.getAttribute('src') || '';
                
                if (dataSrc && (currentSrc === '' || currentSrc.includes('placeholder') || currentSrc.includes('blank'))) {
                    pendingImages.push(img);
                    // 강제로 로드 트리거
                    img.src = dataSrc;
                }
                
                // 아직 로드되지 않은 이미지 확인
                if (!img.complete && img.src && img.src !== '') {
                    pendingImages.push(img);
                }
            }
            
            return {
                total: images.length,
                pending: pendingImages.length,
                loaded: images.length - pendingImages.length
            };
        "#, vec![]).await;
        
        match result {
            Ok(exec_result) => {
                let pending: u32 = exec_result.json().get("pending")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)
                    .unwrap_or(0);
                
                if pending == 0 {
                    return Ok(());
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to check image loading status: {}", e));
            }
        }
        
        if start.elapsed() > timeout {
            // 타임아웃 - 일부 이미지가 로드되지 않았을 수 있음
            return Ok(());
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
