# TB373FU/TB375FC 루팅, AAL 해제 및 144Hz 고정 검증 기록

이 문서는 한 대의 기기에서 실제로 성공한 작업 결과를 정리한 사용자 검증 기록입니다. LPMBox의 기능 개발과 문제 해결에 참고할 수 있도록 작성했습니다. 펌웨어, Magisk, 패치된 LK, `init_boot`, LK 또는 DTBO 바이너리는 포함하거나 재배포하지 않습니다.

## 1. 검증 환경

```text
하드웨어: Xiaoxin Pad Pro 12.7 2025 중국 내수용 기기
글로벌 롬 전환 후 Android 모델: TB373FU
부트로더 제품 코드: P98300DA2
빌드: TB373FU_ROW_OPEN_USER_M21.814_A16_ZUI_17.5.10.057_ST_260311
Android 버전: 16
처음 활성화된 슬롯: a
파티션 구성: A/B 방식
Magisk 패치 대상: init_boot
작업 환경: Windows 11 / PowerShell 7
검증 당시 Platform Tools 버전: 36.0.0
```

이 기기는 원래 중국 내수용 롬을 사용했으며, LPMBox를 이용하여 TB373FU 글로벌 롬으로 전환한 상태였습니다.

## 2. 이 문서에서 검증한 목표

1. 패치된 LK를 임시로 사용하는 방식으로 부트로더 잠금을 해제합니다.
2. Android로 부팅하기 전에 서로 호환되는 LK 및 DTBO 네 파티션을 복구합니다.
3. 현재 글로벌 롬과 정확히 일치하는 `init_boot.img`를 Magisk로 패치하여 현재 슬롯을 루팅합니다.
4. 화면 내용에 따라 백라이트가 변하는 MediaTek AAL 기능을 비활성화합니다.
5. 디스플레이 주사율 정책을 144Hz로 고정합니다.

## 3. 작업 전 확인한 정보

부트로더 잠금 해제를 시작하기 전에 다음 Android 속성과 파티션을 확인했습니다.

```text
ro.product.device=TB373FU
ro.boot.flash.locked=1
ro.boot.verifiedbootstate=green
ro.boot.slot_suffix=_a
```

관련 파티션은 다음과 같습니다.

```text
boot_a
boot_b
init_boot_a
init_boot_b
vbmeta_a
vbmeta_b
vendor_boot_a
vendor_boot_b
```

Fastboot에서는 다음과 같이 표시됐습니다.

```text
product: P98300DA2
current-slot: a
unlocked: no
unlock_ability is true
```

## 4. 실제로 성공한 부트로더 잠금 해제 순서

다음 MD5와 일치하는 패치된 LK를 사용했습니다.

```text
2C01C2ECF1768555D75ABF0E42CF7A78
```

해당 바이너리 파일은 이 문서에 포함하지 않습니다.

실제로 검증한 순서는 다음과 같습니다.

```powershell
# 이 단계 전에 현재 슬롯과 패치 파일의 해시를 확인합니다.
.\fastboot flash lk .\lk_patched
.\fastboot reboot bootloader
.\fastboot devices

.\fastboot flashing unlock_critical
.\fastboot flashing unlock

.\fastboot reboot bootloader
.\fastboot getvar unlocked 2>&1
```

마지막에 반드시 다음 결과가 나와야 했습니다.

```text
unlocked: yes
```

### LPMBox 구현 시 참고할 점

- `unlock_critical`이 `OKAY`를 반환하는 것만으로는 충분하지 않습니다. 이 검증에서는 일반 잠금 해제 명령을 실행하기 전까지 기기가 계속 `unlocked: no`로 표시됐습니다.
- 일반 잠금 해제 명령에서 `Please Implement check key function`이 출력됐지만, 최종 상태는 `unlocked: yes`가 됐습니다. 따라서 개별 출력 문구보다 마지막 상태 조회 결과를 성공 기준으로 사용하는 편이 안전합니다.
- 패치된 임시 LK가 적용된 상태에서는 Android로 정상 재부팅하는 선택지를 제공하면 안 됩니다.

## 5. Android 부팅 전에 반드시 필요한 LK 및 DTBO 복구

글로벌 롬 전환 상태의 중국 내수용 하드웨어에서는 일반 TB373FU 글로벌 순정 LK가 호환되지 않았습니다. 안전하게 복구하려면 서로 맞는 다음 네 파일을 한 세트로 사용해야 했습니다.

```text
lk_a
lk_b
dtbo_a
dtbo_b
```

실제로 성공한 복구 순서는 다음과 같습니다.

```powershell
.\fastboot flash dtbo_a .\dtbo_a
.\fastboot flash lk_a .\lk_a
.\fastboot flash dtbo_b .\dtbo_b
.\fastboot flash lk_b .\lk_b

.\fastboot --set-active=a
.\fastboot reboot bootloader
.\fastboot getvar current-slot 2>&1
.\fastboot getvar unlocked 2>&1
.\fastboot reboot
```

기기는 Orange State 안내 화면을 지난 뒤 Android 초기 설정 화면으로 정상 부팅했습니다.

### 권장하는 안전 검사

LPMBox에서는 다음 조건이 모두 충족될 때까지 Android 재부팅을 막는 방식이 안전할 것 같습니다.

- 네 번의 파티션 플래시 작업이 모두 성공했습니다.
- 부트로더를 다시 시작한 뒤 `current-slot`을 정상적으로 조회할 수 있습니다.
- `unlocked` 상태가 계속 `yes`입니다.
- 선택한 복구 파일 묶음이 지원하는 하드웨어 계열과 일치합니다.

현재 LPMBox가 TB375FC와 TB373FU에 대해 LK 및 DTBO 네 파티션을 모두 처리하도록 구성한 정책은 이번 실제 검증 결과와 일치합니다.

## 6. `init_boot`를 이용한 Magisk 루팅

현재 설치된 글로벌 롬 빌드와 정확히 일치하는 `init_boot.img`를 기기로 복사한 뒤 Magisk 앱에서 패치했습니다.

현재 활성 슬롯 하나에만 패치된 이미지를 플래시했습니다.

```powershell
.\adb shell getprop ro.boot.slot_suffix
.\adb reboot bootloader
.\fastboot getvar current-slot 2>&1
.\fastboot flash init_boot_a .\magisk_patched_init_boot.img
.\fastboot reboot
```

부팅 후 다음 명령으로 루트 권한을 확인했습니다.

```powershell
.\adb shell su -c id
```

실제 확인 결과는 다음과 같습니다.

```text
uid=0(root) gid=0(root) context=u:r:magisk:s0
```

### 루팅 도우미에 권장하는 기능

- 설치된 롬의 빌드와 선택한 `init_boot.img`가 정확히 일치하는지 확인해야 합니다.
- 플래시하기 전에 순정 `init_boot.img`를 보관해야 합니다.
- 기본값으로 현재 활성 슬롯 하나만 플래시해야 합니다.
- 순정 `init_boot`를 바로 복구할 수 있는 기능을 제공하면 좋습니다.
- 기기별로 검증된 절차가 요구하지 않는 한, 기본 과정에서 AVB를 비활성화하거나 `vbmeta`를 플래시하지 않는 편이 안전합니다.

## 7. MediaTek AAL 원인 확인과 비활성화

Android의 자동 밝기를 꺼도, 표시되는 화면 내용에 따라 실제 백라이트 밝기가 달라졌습니다.

관련 값은 다음과 같습니다.

```text
screen_brightness_mode=0
ro.vendor.mtk_aal_support=1
ro.vendor.pq.mtk_aal_support=1
persist.vendor.sys.pq.disp.aal.bypass=0
```

AAL을 우회하기 전에는 흰 화면과 검은 화면을 번갈아 표시할 때 `AALService` 로그에서 백라이트 값이 크게 변했습니다.

```text
BL=4095
...
BL=2238
```

Magisk 루팅 후 다음 방식으로 안정적으로 해결했습니다.

```powershell
.\adb shell su -c 'resetprop persist.vendor.sys.pq.disp.aal.bypass 1'
.\adb shell getprop persist.vendor.sys.pq.disp.aal.bypass
.\adb shell su -c 'resetprop -p persist.vendor.sys.pq.disp.aal.bypass'
.\adb reboot
```

재부팅 후에도 값이 `1`로 유지됐습니다. 흰 화면과 검은 화면 전환 시험을 다시 했을 때 `AALService` 출력이 나타나지 않았고, 눈에 보이던 화면 내용 기반 밝기 변화도 멈췄습니다.

원래 상태로 되돌리는 명령은 다음과 같습니다.

```powershell
.\adb shell su -c 'resetprop persist.vendor.sys.pq.disp.aal.bypass 0'
.\adb reboot
```

### AAL 기능에 권장하는 구성

- AAL 지원 속성이 확인되는 기기에서만 기능을 표시합니다.
- Magisk 루트 권한이 있는지 확인하고 실제 명령 실행도 검증합니다.
- 현재 우회 설정값을 화면에 표시합니다.
- **화면 내용 기반 백라이트 끄기**(`1`)와 **기본값 복구**(`0`)를 제공합니다.
- 재부팅 후 값을 다시 확인합니다.
- 선택 사항으로 짧은 `AALService` 로그 시험을 제공할 수 있습니다.

## 8. 디스플레이를 144Hz로 고정하기

이 기기는 기본적으로 30Hz, 60Hz, 144Hz 사이를 동적으로 전환했습니다. 다음 설정을 함께 변경하자 144Hz가 유지됐으며 재부팅 후에도 설정이 남았습니다.

먼저 실제 해상도를 자동으로 읽었습니다.

```powershell
$SizeLine = (
    .\adb shell wm size |
    Select-String -Pattern '^Physical size:'
).Line

$Width, $Height = (
    ($SizeLine -replace '^Physical size:\s*', '') -split 'x'
)
```

그다음 사용자 선호 디스플레이 모드와 주사율 범위를 설정했습니다.

```powershell
.\adb shell cmd display set-user-preferred-display-mode $Width $Height 144.0 0
.\adb shell settings put system min_refresh_rate 144.0
.\adb shell settings put system peak_refresh_rate 144.0
.\adb shell cmd display set-match-content-frame-rate-pref 0
```

다음 명령으로 적용 여부를 확인했습니다.

```powershell
.\adb shell settings get system min_refresh_rate
.\adb shell settings get system peak_refresh_rate
.\adb shell cmd display get-user-preferred-display-mode 0
.\adb shell cmd display get-match-content-frame-rate-pref
```

재부팅 후에도 모든 값이 올바르게 유지됐고, 검증한 상황에서 개발자 옵션의 주사율 표시가 계속 144Hz로 나타났습니다.

원래의 동적 주사율 설정으로 되돌리는 명령은 다음과 같습니다.

```powershell
.\adb shell settings delete system min_refresh_rate
.\adb shell settings delete system peak_refresh_rate
.\adb shell cmd display clear-user-preferred-display-mode 0
.\adb shell cmd display set-match-content-frame-rate-pref 1
.\adb reboot
```

### 주사율 기능에 권장하는 구성

- 144Hz를 고정값으로 넣지 말고 실제 해상도와 지원 디스플레이 모드를 감지합니다.
- 범용 기능 이름은 **최대 지원 주사율로 고정**처럼 표시하고, 지원 모드 중 가장 높은 값을 선택합니다.
- 변경하기 전에 기존 설정값을 모두 백업합니다.
- 원래 설정으로 되돌리는 명확한 복구 버튼을 제공합니다.
- 패널이 144Hz로 작동해도 애플리케이션이 반드시 초당 144프레임을 렌더링하는 것은 아니라는 점을 안내합니다.
- 배터리 사용량과 발열이 증가할 수 있음을 경고합니다.
- 발열 보호 제한은 우회하지 않습니다.

## AP500U/AP501U 계열 펜과 144Hz 완전 고정의 호환성

144Hz 완전 고정을 적용한 뒤 Lenovo Tab Pen Plus AP500U/AP501U
계열 정품 펜의 펜촉 입력이 작동하지 않는 문제를 확인했습니다.

### 나타난 증상

다음 기능은 정상적으로 작동했습니다.

- Bluetooth 연결
- 펜 버튼 입력
- 펜 근접 감지
- 펜 근접 상태에서 손가락 입력 차단

그러나 다음 기능은 작동하지 않았습니다.

- 펜촉으로 화면 누르기
- 펜촉을 이용한 선택과 스크롤
- 필기 및 그리기
- 필압 입력

즉, 펜의 근접 감지는 정상인 상태에서 실제 펜촉 접촉 입력만
작동하지 않았습니다.

### 문제를 발생시킨 설정

```text
min_refresh_rate=144.0
peak_refresh_rate=144.0
user preferred display mode=1840x2944 @ 144Hz
match content frame rate preference=0
```

### 원래 동적 설정으로 복구한 방법

```powershell
.\adb shell settings delete system min_refresh_rate
.\adb shell settings delete system peak_refresh_rate
.\adb shell cmd display clear-user-preferred-display-mode 0
.\adb shell cmd display set-match-content-frame-rate-pref 1
.\adb shell reboot -p
```

완전히 전원을 껐다가 다시 켠 뒤 펜촉과 필압 입력이 정상으로
돌아왔습니다.

`dumpsys display`에서 확인한 지원 모드는 다음과 같습니다.

```text
144.00002Hz
120.00001Hz
60.0Hz
30.000002Hz
```

펜이 정상적으로 작동하는 상태에서는 실제 활성 디스플레이 모드가
120.00001Hz로 표시됐습니다.

```text
mActiveSfDisplayMode=120.00001Hz
renderFrameRate=120.00001Hz
```

내부 드라이버의 정확한 동작 원리까지 확인한 것은 아니므로,
모든 기기와 모든 펜에서 동일하다고 단정할 수는 없습니다.
다만 검증한 기기에서는 144Hz 완전 고정 상태에서 문제가
재현됐고, 강제 설정을 제거하여 120Hz가 선택되자 즉시
정상으로 복구됐습니다.

### 펜 사용자를 위한 권장 설정

AP500U/AP501U 계열 펜을 사용하는 경우에는 144Hz 완전 고정보다
120Hz 완전 고정을 권장합니다.

```powershell
.\adb shell cmd display `
    set-user-preferred-display-mode `
    1840 2944 120.00001 0

.\adb shell settings put system min_refresh_rate 120.0
.\adb shell settings put system peak_refresh_rate 120.0
.\adb shell cmd display set-match-content-frame-rate-pref 0
.\adb reboot
```

적용 상태는 다음 명령으로 확인합니다.

```powershell
.\adb shell settings get system min_refresh_rate
.\adb shell settings get system peak_refresh_rate
.\adb shell cmd display get-user-preferred-display-mode 0
.\adb shell cmd display get-match-content-frame-rate-pref
```

목표 결과는 다음과 같습니다.

```text
120.0
120.0
User preferred display mode: 1840 2944 120.00001
Match content frame rate type: 0
```

검증한 환경에서는 이 설정으로 화면이 120Hz에 고정되었으며,
펜촉 입력, 필압, 버튼, Bluetooth 연결과 손바닥 입력 차단 기능이
모두 정상적으로 작동했습니다.

### LPMBox 구현 시 권장 구성

주사율 기능을 하나의 최대값 고정 버튼으로 제공하기보다는 다음과
같이 구분하는 편이 안전합니다.

- **시스템 기본 동적 주사율**
- **120Hz 고정, 펜 사용자 권장**
- **최대 지원 주사율 고정, 펜 호환성 경고 표시**

144Hz 완전 고정을 선택할 때 스타일러스 입력 장치가 감지되면
AP500U/AP501U 계열 펜의 펜촉과 필압 입력이 작동하지 않을 수
있다는 경고를 표시하는 것이 좋습니다.

## 9. 개인정보를 제거한 진단 보고서 제안

LPMBox에서 GitHub Issue용 진단 보고서를 만들 때 다음 정보를 포함하면 좋을 것 같습니다.

```text
ro.product.model
ro.build.display.id
ro.build.version.release
ro.vendor.mediatek.platform
ro.boot.slot_suffix
ro.boot.flash.locked
ro.boot.verifiedbootstate
init_boot 파티션 존재 여부
AAL 지원 속성과 우회 설정값
min_refresh_rate
peak_refresh_rate
사용자 선호 디스플레이 모드
콘텐츠 프레임 속도 일치 설정
마지막 ADB/Fastboot 단계와 결과
```

다음 정보는 자동으로 제거해야 합니다.

- 기기 일련번호
- 개인 Windows 사용자 경로
- 인증 토큰
- 이메일 주소

## 10. 최종 확인 상태

```text
ro.boot.flash.locked=0
ro.boot.verifiedbootstate=orange
Magisk 루트 권한: uid=0(root)
persist.vendor.sys.pq.disp.aal.bypass=1
최종 화면 전환 시험 후 AALService: 출력 없음
min_refresh_rate=144.0
peak_refresh_rate=144.0
콘텐츠 프레임 속도 일치 설정=0
```

부트로더 잠금이 해제되어 있는 동안 Orange State 안내 화면이 나오는 것은 정상입니다.
