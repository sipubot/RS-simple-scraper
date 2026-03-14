# RS Simple Scraper 변경 로그

## 2025-03-14 개선 사항

### 1. 의존성 정리

#### 제거된 의존성
| 패키지 | 이유 |
|--------|------|
| `time` (0.3.47) | `chrono`와 중복되어 불필요 |
| `serde_derive` (1.0.228) | `serde`의 derive feature로 통합 |
| `derive_more` (2.0.1) | 코드에서 미사용 |

#### 변경된 Cargo.toml
```toml
# 변경 전
serde = "1.0.228"
serde_derive = "1.0.228"
time = "0.3.47"
derive_more = { version = "2.0.1", features = ["full"] }

# 변경 후
serde = { version = "1.0.228", features = ["derive"] }
```

### 2. 매직 넘버 상수화

#### main.rs
```rust
// 설정 파일 경로
const SAVE_PATH: &str = "./save.json";
const SITE_PATH: &str = "./site.json";
const DOWN_PATH: &str = "./down.json";
const NICK_RULE: &str = "./nick.json";

// 타이밍 상수
const SCRAPE_INTERVAL_SECS: u64 = 300;      // 5분
const REQUEST_DELAY_MS: u64 = 500;          // 요청 간 지연
const NEW_MARKER_AGE_SECS: i64 = 28800;     // 8시간 (new 플래그 유지)
const MAX_POST_AGE_SECS: i64 = 172800;      // 48시간 (게시물 필터링)
```

#### utils.rs
```rust
const HTTP_TIMEOUT_SECS: u64 = 30;
const POOL_MAX_IDLE_PER_HOST: usize = 10;
const APP_USER_AGENT: &str = "RS Simple Scraper/1.0";
const BOT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:98.0) Gecko/20100101 Firefox/98.0";
```

#### foxfox.rs
```rust
const WEBDRIVER_URL: &str = "http://localhost:4444";
const BODY_LOAD_TIMEOUT_SECS: u64 = 10;
const BODY_LOAD_POLL_MS: u64 = 500;
const SCROLL_DISTANCE: i32 = 200;
const MAX_SCROLL_COUNT: i32 = 50;
const SCROLL_DELAY_MS: u64 = 50;
const CONTENT_LOAD_WAIT_SECS: u64 = 3;
const IMAGE_LOAD_TIMEOUT_SECS: u64 = 5;
```

#### dc.rs
```rust
const MAX_POST_AGE_SECS: i64 = 172800;  // 48시간
```

### 3. 이미지 레이지 로딩 개선 (foxfox.rs)

#### 추가된 기능
- `wait_for_images()` 함수 신규 추가
- `data-src`/`data-original` 속성을 가진 이미지 강제 로드
- 이미지 로드 완료 여부 확인 후 HTML 추출
- 5초 타임아웃으로 안전장치 구현

```rust
async fn wait_for_images(driver: &WebDriver) -> Result<()> {
    // data-src 속성을 가진 이미지들이 모두 로드될 때까지 대기
    let timeout = Duration::from_secs(IMAGE_LOAD_TIMEOUT_SECS);
    // ... 이미지 로딩 로직
}
```

### 4. WebDriver 리소스 정리 개선

#### 변경 전
```rust
let _ = driver.quit().await;  // 에러 무시
```

#### 변경 후
```rust
if let Err(e) = driver.quit().await {
    eprintln!("Warning: Failed to quit WebDriver cleanly: {}", e);
}
```

### 5. 설정 파일 예제 추가

| 파일 | 설명 |
|------|------|
| `site.json.example` | 스크래핑할 사이트 설정 예제 |
| `save.json.example` | 저장 경로 설정 예제 |
| `down.json.example` | 이미지 다운로드 설정 예제 |
| `nick.json.example` | 차단 닉네임 설정 예제 |

### 6. 의존성 업데이트

`cargo update` 실행으로 44개 패키지 최신 버전으로 업데이트:
- `tokio`: 1.49.0 → 1.50.0
- `chrono`: 0.4.43 → 0.4.44
- `futures`: 0.3.31 → 0.3.32
- `anyhow`: 1.0.101 → 1.0.102
- 기타 40개 패키지 업데이트

---

## 파일 변경 요약

| 파일 | 변경 유형 | 설명 |
|------|----------|------|
| `Cargo.toml` | 수정 | 의존성 정리 (3개 제거) |
| `src/main.rs` | 수정 | 상수 추출, 매직 넘버 제거 |
| `src/utils.rs` | 수정 | HTTP 클라이언트 설정 상수화 |
| `src/foxfox.rs` | 수정 | 레이지 로딩 지원, 상수 추출 |
| `src/scrapers/dc.rs` | 수정 | MAX_POST_AGE_SECS 상수 추가 |
| `src/scrapers/fm.rs` | - | 변경 없음 |
| `src/scrapers/mp.rs` | - | 변경 없음 |
| `src/models.rs` | - | 변경 없음 (serde_derive 사용 유지) |
| `.gitignore` | 수정 | `*.json.example` 예외 추가 |
| `site.json.example` | 추가 | 사이트 설정 예제 |
| `save.json.example` | 추가 | 저장 설정 예제 |
| `down.json.example` | 추가 | 다운로드 설정 예제 |
| `nick.json.example` | 추가 | 닉네임 필터 예제 |
| `CHANGELOG.md` | 추가 | 변경 로그 파일 |
