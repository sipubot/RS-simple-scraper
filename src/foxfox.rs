use thirtyfour::prelude::*;
use tokio;
use std::time::Duration;

/// Firefox로 페이지 크롤링
pub async fn get_html(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    // WebDriver 연결
    let caps = DesiredCapabilities::firefox();
    let driver = WebDriver::new("http://localhost:4444", caps).await?;

    // 페이지 이동
    driver.goto(url).await?;

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

    Ok(html)
}
