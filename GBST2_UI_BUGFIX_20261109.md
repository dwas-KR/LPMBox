# LPMBox 3.1.2 — GBST 2.0.0 GUI 이식 후 버그 수정 보고서 (2026-11-09)

## 작업 기준
- 원본 GBST: G:\TB375FC, TB373FU\GBST (v2.0.0)
- 대상 LPMBox: G:\TB375FC, TB373FU\LPMBox-rs-3.1.2
- GUI / 설정 / 표시 / 경로 선택 / 현지화 코드를 수정하고 기존 ROM·복구 worker를 보존했습니다.
- 백업 사본: G:\TB375FC, TB373FU\ResearchReadOnly\LPMBox_before_20261109_bugfix
- 기존 Git 백업 브랜치: backup/pre-gbst-v2-port-20261109

## 수정된 문제
1. 시작 시 콘솔 점멸: Windows GUI는 이미 windows_subsystem=windows로 설정되었으며 시작 시 테마 조회용 reg.exe 실행에 CREATE_NO_WINDOW를 추가했습니다.
2. 정보 화면 로고 1초 주기 점멸: 매 렌더 때 생성된 image::Handle을 LazyLock의 고정 핸들로 교체했습니다.
3. Proinfo 백업: 사이드바 메뉴에서만 제거했으며 기존 Proinfo 백업·국가코드·ROM 백엔드는 보존했습니다.
4. image 폴더: 직접 선택한 image 폴더 또는 선택한 상위 경로의 image 하위 폴더를 사용합니다. image가 없으면 검사 전에 오류가 표시됩니다.
5. 현지화: 공통 UI 언어별 394 JSON키에 신규 오류 메시지를 추가했고, 228개 기존 고정 표현을 legacy-*.json으로 유지하고, 전체 고정 한글 소스 문자열 1,512개를 12개 언어 fixed-*.json으로 이관했습니다. 원래 번역 단계는 기존 호환용으로 남겼습니다. 모든 동적 로그를 JSON으로 전환한 것은 아닙니다.
6. 다크 모드: ROM 검사·결과·작업/옵션 카드·국가코드 팝업·드라이버 팝업·언어 선택 스타일 등 고정 흰색 배경을 테마 반응형 팔레트로 변경했습니다.

## GBST v2.0.0 변경 로그와 LPMBox의 차이
| GBST | LPMBox 반영 상태 |
| --- | --- |
| 1. GUI 전면 개편 | 공통 GUI, 메뉴, 설정, 정보, 테마, 로그 이식 및 이번 버그 수정 |
| 2. Google Services 설치 안정화 | GMS 설치기 기능이므로 미이식. LPMBox ROM 고유 작업 유지 |
| 3. USB ADB 통신·64KiB Bulk 최적화 | GBST의 개조 ADB 엔진은 이식되지 않음. 동등 속도 보장 불가 |
| 4. APK 다운로드·SHA256 검증 | GMS APK 전용. LPMBox ROM image 검사기는 별도 |
| 5. 진행률·로그 | 로그와 ROM 작업 단계 화면 이식. GMS 4단계/퍼센트와 다른 작업 |
| 6. JSON 다국어 | 공통 UI 394키 × 12개 + 기존 228개 및 전체 고정 소스 문자열 1,512개 × 12개. 동적 로그 fallback 보존 |
| 7. 로그·캐시 정리 | PC 로그·임시 파일 정리 도입. 기기 측 APK 캐시는 미이식 |
| 8. GPLv3 | 동일 GPL-3.0-only. 기존 제3자 출처 저작권 별도 확인 필요 |

## 검증
- 최초 수정 cargo check --offline: 성공.
- 수정 이후 cargo test --workspace --offline --quiet: 28개 테스트 성공(코어 4, GUI 24), 실패 0.
- image 폴더 경로 판정 테스트 추가: 직접 선택, 상위 폴더 선택, 잘못된 폴더 차단.
- 최종 Windows Release 빌드: cargo build --release -p lpmbox-gui --offline.
- 실제 태블릿 ROM 플래시, 리커버리, 초기화, 재부팅 실행 없음.
- GitHub 업로드/커밋/push 없음. 로컬 작업 트리 수정.
- 실기기 ROM 회귀 검증, 다양한 화면 배율 UI 점검, 전체 동적 로그 자연어 번역 감수는 별도 필요.
