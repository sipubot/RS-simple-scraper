# GeckoDriver 정보 보고서

## 최신 버전 정보
- **버전**: v0.37.1
- **발행일**: 2026-07-20
- **다운로드**: https://github.com/mozilla/geckodriver/releases/tag/v0.37.1

## 현재 프로젝트 사용 현황
- **thirtyfour crate**: v0.37.4
- **GeckoDriver 호환성**: ✅ 현재 프로젝트는 thirtyfour v0.37.4와 GeckoDriver v0.37.1 조합을 권장합니다

## 최신 기준 요약 (v0.37.1)

- 최신 GeckoDriver 공개 릴리스는 v0.37.1이며, 현재 프로젝트는 `thirtyfour` v0.37.4와 함께 사용할 것을 권장합니다.
- 이는 공개 버전 기준으로 확인된 최신 조합이며, 문서의 이전 버전 참조를 정리한 상태입니다.

### 주의사항
- Firefox 138.0+에서는 `--allow-system-access` 옵션이 필요한 경우가 있습니다.
- Container 환경(snap/flatpak)에서는 Firefox 실행 시 파일시스템 접근 문제를 겪을 수 있습니다.

## 알려진 문제 (Known Issues)

### 1. Container 환경에서의 시작 지연
- **영향**: Ubuntu 22.04 기본 Firefox (snap/flatpak)
- **증상**: Firefox 시작 시 hang 발생 가능
- **해결책**: https://firefox-source-docs.mozilla.org/testing/geckodriver/Usage.html#Running-Firefox-in-an-container-based-package

### 2. Virtual Authenticator 불안정
- **영향**: WebAuthn 관련 기능
- **권고**: 관련 기능이 필요할 때는 최신 Firefox/GeckoDriver 조합과 함께 테스트를 권장합니다

## 보안 관련 정보

### CVE (공식 보안 취약점)
- **현재까지 공식 CVE 없음**
- 최신 릴리즈 기준으로 별도 공개 보안 취약점 보고는 확인되지 않았습니다

### 보안 권장사항
1. **항상 최신 버전 사용**: v0.37.1 권장
2. **Firefox 버전 호환성**:
   - Firefox 137+: fractional 좌표 지원
   - Firefox 138.0+: `--allow-system-access` 필요
3. **Container 환경 주의**: snap/flatpak 환경에서 파일시스템 접근 이슈

## 업그레이드 체크리스트

- [ ] GeckoDriver v0.37.1 다운로드
- [ ] Firefox 버전 확인 (권장: 137+)
- [ ] (옵션) `--allow-system-access` 플래그 테스트 (Firefox 138.0+)
- [ ] (옵션) `MINIDUMP_SAVE_PATH` 환경변수 설정 (디버깅용)
- [ ] 스크래핑 정상 작동 확인

## 참고 링크
- https://github.com/mozilla/geckodriver/releases
- https://firefox-source-docs.mozilla.org/testing/geckodriver/
