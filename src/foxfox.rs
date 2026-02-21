use thirtyfour::prelude::*;
use std::time::Duration;
use anyhow::{Result, Context};

/// Firefox로 페이지 크롤링 (최적화 버전)
pub async fn get_html(url: &str) -> Result<String> {
    // WebDriver 연결
    let caps = DesiredCapabilities::firefox();
    let driver = WebDriver::new("http://localhost:4444", caps).await
        .context("Failed to connect to WebDriver at http://localhost:4444")?;

    // 페이지 이동
    driver.goto(url).await.context(format!("Failed to navigate to URL: {}", url))?;

    // body 태그 로딩 대기 (최대 10초, 폴링 간격 500ms)
    let body = driver
        .query(By::Tag("body"))
        .wait(Duration::from_secs(10), Duration::from_millis(500))
        .first()
        .await
        .context("Failed to find body tag within 10 seconds")?;

    // 스마트 스크롤 - 필요한 만큼만 스크롤, 최대 50번 제한
    driver.execute(r#"
        async function smartScroll() {
            let distance = 200;
            let delay = ms => new Promise(res => setTimeout(res, ms));
            let maxScrolls = 50;
            let scrollCount = 0;

            while ((window.innerHeight + window.scrollY) < document.body.scrollHeight && scrollCount < maxScrolls) {
                window.scrollBy(0, distance);
                scrollCount++;
                await delay(50); // 50ms 딜레이로 더 빠른 스크롤
            }
            return scrollCount;
        }
        return smartScroll();
    "#, vec![]).await.context("Failed to execute smart scroll script")?;

    // 동적 콘텐츠 로딩을 위한 대기 (코멘트 요구사항: 3초 vs 기존 코드: 5초)
    // 최적화: 3초로 단축
    tokio::time::sleep(Duration::from_secs(3)).await;

    // HTML 추출
    let html = body.inner_html().await.context("Failed to extract inner HTML from body")?;

    // 브라우저 종료
    let _ = driver.quit().await;

    Ok(html)
}
