# GeckoDriver 정보 보고서

## 최신 버전 정보
- **버전**: v0.36.0
- **발행일**: 2025-02-25
- **다운로드**: https://github.com/mozilla/geckodriver/releases/tag/v0.36.0

## 현재 프로젝트 사용 현황
- **thirtyfour crate**: v0.36.1
- **GeckoDriver 호환성**: ✅ thirtyfour v0.36.1은 GeckoDriver v0.36.0과 호환됨

## 주요 변경사항 (v0.36.0)

### 새로운 기능
1. **macOS Firefox Developer Edition 경로 지원**
2. **Android WebExtension 설치 지원**
3. **Private Browsing 모드 WebExtension 지원**
4. **`--allow-system-access` 플래그 추가**
   - Firefox 138.0+에서 `chrome` context 테스트에 필요
   - 시스템 레벨 접근 권한이 필요한 경우 사용
5. **Crash Dump 저장 지원**
   - `MINIDUMP_SAVE_PATH` 환경변수 설정 시 크래시 덤프 자동 저장

### 버그 수정
- **WebAuthn 명령 경로 수정** (v0.34.0에서 `/sessions/` → `/session/`)

### 주의사항 (Breaking Changes)
- **`-no-remote` 옵션 제거**: Firefox에서 더 이상 지원하지 않음
- **`--enable-crash-reporter` 폐기**: 다음 버전에서 완전 제거 예정

## 알려진 문제 (Known Issues)

### 1. Container 환경에서의 시작 지연
- **영향**: Ubuntu 22.04 기본 Firefox (snap/flatpak)
- **증상**: Firefox 시작 시 hang 발생 가능
- **해결책**: https://firefox-source-docs.mozilla.org/testing/geckodriver/Usage.html#Running-Firefox-in-an-container-based-package

### 2. Virtual Authenticator 불안정
- **영향**: WebAuthn 관련 기능
- **권고**: v0.34.0부터 문제 보고됨, 해결될 때까지 사용 자제

## 보안 관련 정보

### CVE (공식 보안 취약점)
- **현재까지 공식 CVE 없음**
- v0.36.0 릴리즈 노트에 특정 보안 취약점 언급 없음

### 보안 권장사항
1. **항상 최신 버전 사용**: v0.36.0 권장
2. **Firefox 버전 호환성**:
   - Firefox 137+: fractional 좌표 지원
   - Firefox 138.0+: `--allow-system-access` 필요
3. **Container 환경 주의**: snap/flatpak 환경에서 파일시스템 접근 이슈

## 업그레이드 체크리스트

- [ ] GeckoDriver v0.36.0 다운로드
- [ ] Firefox 버전 확인 (권장: 137+)
- [ ] (옵션) `--allow-system-access` 플래그 테스트 (Firefox 138.0+)
- [ ] (옵션) `MINIDUMP_SAVE_PATH` 환경변수 설정 (디버깅용)
- [ ] 스크래핑 정상 작동 확인

## 참고 링크
- https://github.com/mozilla/geckodriver/releases
- https://firefox-source-docs.mozilla.org/testing/geckodriver/
