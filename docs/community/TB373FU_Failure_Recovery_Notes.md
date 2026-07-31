# TB373FU/TB375FC 실패 사례와 복구 기록

이 문서는 TB373FU 글로벌 롬, 부트로더 잠금 해제, Magisk 루팅, AAL 비활성화 및 144Hz 고정 작업을 한 대의 기기에서 성공시키는 과정에서 실제로 겪은 실패를 정리한 기록입니다. LPMBox의 기능 구현과 문제 해결에 참고할 수 있도록 작성했습니다.

## 1. PowerShell에서는 현재 폴더의 실행 파일 앞에 `.\`가 필요함

### 증상

```text
adb is not recognized
```

### 해결 방법

Platform Tools 폴더 안에서 PowerShell 명령을 다음과 같이 실행했습니다.

```powershell
.\adb devices
.\fastboot devices
```

사용자가 PowerShell 안내를 선택한 경우, 화면에 표시하는 명령에도 PowerShell에 필요한 점과 역슬래시 접두사인 `.\`를 포함하는 편이 좋습니다.

## 2. ADB에서 `unauthorized`가 표시됨

### 증상

```text
SERIAL    unauthorized
```

### 해결 방법

- 태블릿 화면의 잠금을 풉니다.
- USB 디버깅 RSA 승인 창을 허용합니다.
- **이 컴퓨터에서 항상 허용**을 선택합니다.
- 필요하면 USB 디버깅 승인을 취소한 뒤 케이블을 다시 연결합니다.

LPMBox는 이미 승인되지 않은 기기를 감지하고 있습니다. 여기에 화면을 보면서 따라 할 수 있는 단계별 설명이나 그림을 추가하면 혼동을 줄일 수 있습니다.

## 3. 일반 ADB 권한으로 AAL 속성 변경이 거부됨

### 실패한 명령

```powershell
.\adb shell setprop persist.vendor.sys.pq.disp.aal.bypass 1
```

### 결과

```text
Failed to set property ...
See dmesg for error reason.
```

### 의미

일반 ADB 셸은 해당 vendor 속성을 읽을 수는 있었지만, 제품용 SELinux 정책 때문에 수정할 수 없었습니다.

### Magisk 루팅 후 성공한 방법

```powershell
.\adb shell su -c 'resetprop persist.vendor.sys.pq.disp.aal.bypass 1'
```

루트 권한이 필요한 기능에서는 일반 `setprop`가 성공할 것으로 가정하고 먼저 실행하는 방식보다, 루트 권한을 확인한 뒤 `resetprop`를 사용하는 편이 맞습니다.

## 4. MediaTek PQ AIDL의 `dumpsys`는 제어 인터페이스로 사용할 수 없었음

### 실패한 명령

```text
dumpsys vendor.mediatek.hardware.pq_aidl.IPictureQuality_AIDL/default --help
dumpsys vendor.mediatek.hardware.pq_aidl.IPictureQuality_AIDL/default --aal --function 0x0
```

### 결과

```text
FAILED_TRANSACTION
```

### 결론

검증한 `dumpsys` 명령으로는 해당 서비스를 제어할 수 없었습니다. AAL을 실제로 비활성화하려면 루트 권한과 `resetprop`가 필요했습니다.

## 5. 패치된 LK를 적용하기 전에는 표준 및 OEM 잠금 해제가 실패함

### 표준 잠금 해제

```text
fastboot flashing unlock
FAILED: Please usr fastboot oem unlock
```

### OEM 잠금 해제

```text
fastboot oem unlock
ERROR: Sn Image Auth fail
```

### 결론

검증 당시의 글로벌 롬 상태에서는 일반 Lenovo 인증 방식으로 잠금을 해제할 수 없었습니다. 대신 패치된 LK를 임시로 적용하는 방식을 사용했습니다.

## 6. `unlock_critical` 성공이 일반 부트로더 잠금 해제를 의미하지 않았음

### 출력

```text
fastboot flashing unlock_critical
OKAY

fastboot getvar unlocked
unlocked: no
```

### 해결 방법

일반 잠금 해제 명령도 추가로 실행해야 했습니다.

```powershell
.\fastboot flashing unlock
.\fastboot reboot bootloader
.\fastboot getvar unlocked 2>&1
```

성공 기준은 다음과 같습니다.

```text
unlocked: yes
```

LPMBox에서는 상태 조회 결과를 최종 기준으로 사용해야 하며, `unlock_critical`만 성공했다고 전체 잠금 해제 과정을 완료 처리하면 안 됩니다.

## 7. 성공한 잠금 해제 과정에서도 `Please Implement check key function`이 나타남

### 출력

```text
Start unlock flow
Please Implement check key function
OKAY
```

부트로더를 다시 시작한 뒤에는 다음과 같이 표시됐습니다.

```text
unlocked: yes
```

최종 상태 조회가 성공을 확인한다면 이 문구는 자동 실패가 아니라 경고로 표시하는 편이 맞습니다.

## 8. 호환되지 않는 LK 때문에 자동 종료 화면이 나타남

### 경고 화면

```text
the current system is not compatible with the hardware
the device will automatically poweroff
it can work normally after flashing back to the factory version system
```

### 이번 검증에서 확인한 원인

중국 내수용 하드웨어에 글로벌 롬을 전환하여 사용하던 상태에서, 잠금 해제 후 일반 글로벌 순정 LK를 복구용으로 플래시했습니다.

### 해결되지 않았던 시도

- 일반 중국 롬의 LK 하나만 플래시함
- 슬롯 A에서 슬롯 B로 전환함
- 서로 맞는 DTBO 파일을 복구하지 않은 채 재부팅함

### 실제로 성공한 복구

서로 맞는 검증된 네 파일을 모두 복원했습니다.

```powershell
.\fastboot flash dtbo_a .\dtbo_a
.\fastboot flash lk_a .\lk_a
.\fastboot flash dtbo_b .\dtbo_b
.\fastboot flash lk_b .\lk_b
.\fastboot --set-active=a
.\fastboot reboot bootloader
.\fastboot reboot
```

그 뒤 Android 초기 설정 화면까지 정상적으로 부팅됐습니다.

### 제품 기능으로 구현할 때의 권장 방식

패치된 LK를 임시로 사용하는 과정은 다음과 같이 하나의 연속된 작업으로 강제하는 편이 안전합니다.

1. 패치된 LK를 플래시하기 전에 복구용 네 파일이 모두 있는지 확인합니다.
2. 현재 슬롯을 기록합니다.
3. 잠금 해제 순서를 실행합니다.
4. LK 및 DTBO 네 파일을 모두 복원합니다.
5. 각 플래시 명령의 성공 여부를 확인합니다.
6. 부트로더를 다시 시작한 뒤 슬롯과 잠금 상태를 다시 확인합니다.
7. 모든 검사가 통과한 경우에만 Android 재부팅 버튼을 활성화합니다.

## 9. 슬롯 변경 결과는 부트로더를 다시 시작한 뒤에 반영됨

다음 명령을 실행한 직후에는 같은 세션에서 `current-slot`이 계속 `a`로 표시됐습니다.

```powershell
.\fastboot --set-active=b
```

다음 명령으로 부트로더를 다시 시작한 뒤에는 `b`로 표시됐습니다.

```powershell
.\fastboot reboot bootloader
```

LPMBox에서도 부트로더를 다시 시작한 뒤 최종 슬롯 상태를 안내하는 편이 안전합니다.

## 10. Windows의 ADB 서버가 반복해서 다시 시작됨

### 증상

```text
daemon not running; starting now at tcp:5037
protocol fault
connection reset
no devices/emulators found
```

### 안정화에 사용한 방법

```powershell
Get-Process adb -ErrorAction SilentlyContinue | Stop-Process -Force
$env:ADB_MDNS_AUTO_CONNECT = "0"
.\adb start-server
.\adb devices -l
```

USB 디버깅 승인을 다시 허용한 뒤 여러 번 기기 상태를 확인해도 연결이 안정적으로 유지됐습니다.

여기에서 `daemon not running`은 Android 안의 Magisk 데몬이 아니라 Windows에서 실행되는 ADB 서버를 뜻합니다. 오류 안내에서는 두 구성 요소를 분명하게 구분하는 편이 좋습니다.

## 11. AAL용 Magisk 모듈을 따로 설치할 필요가 없었음

처음에는 같은 속성을 여러 부팅 단계에서 적용하는 시험용 모듈을 사용했지만 제거했습니다. 다음과 같이 더 단순한 영구 명령 하나만으로 충분했습니다.

```powershell
.\adb shell su -c 'resetprop persist.vendor.sys.pq.disp.aal.bypass 1'
```

재부팅 후에도 값이 `1`로 유지됐습니다. LPMBox에 내장 기능으로 추가한다면 별도 모듈을 설치하기보다 실제로 필요한 최소 변경만 적용하는 편이 좋습니다.

## 12. Magisk 루팅 후 부팅 실패에 대비한 복구 방법

현재 활성 슬롯의 `init_boot`만 패치했습니다. 패치 이미지 때문에 부팅 문제가 생긴다면 현재 롬과 정확히 일치하는 순정 이미지를 다시 플래시하는 방식으로 복구할 수 있습니다.

```powershell
.\fastboot flash init_boot_a .\init_boot_stock.img
.\fastboot reboot
```

루팅 기능을 구현할 때는 플래시하기 전에 정확한 순정 이미지와 슬롯 정보를 함께 보관해야 합니다.

## 13. 144Hz 성공 여부를 확인한 기준

다음 네 항목을 모두 확인하고, 재부팅 후에도 유지되는 경우에만 144Hz 고정이 성공한 것으로 판단했습니다.

```text
min_refresh_rate=144.0
peak_refresh_rate=144.0
사용자 선호 디스플레이 모드=실제 해상도 @ 144.0
콘텐츠 프레임 속도 일치 설정=0
```

이 펌웨어에서는 설정 하나만 바꾸는 것으로 원하는 주사율 정책을 확실히 적용할 수 없었습니다.

## 14. 기본 동작으로 사용하지 말아야 할 작업

- 패치된 임시 LK가 적용된 상태에서 Android로 부팅하지 않습니다.
- 중국 내수용 기기를 글로벌 롬으로 전환한 상태에서 글로벌 순정 LK 하나만 복구용으로 플래시하지 않습니다.
- 일반 중국 롬의 LK 하나와 서로 맞지 않는 DTBO 파일을 섞지 않습니다.
- `OKAY` 출력만 보고 부트로더 잠금 해제를 성공으로 처리하지 않습니다.
- 기본값으로 `init_boot`의 두 슬롯을 모두 플래시하지 않습니다.
- 기기별로 검증된 절차가 요구하지 않는 한 `vbmeta` 검증을 비활성화하지 않습니다.
- 변조된 파티션이나 글로벌 롬 전환용 파티션이 적용된 상태에서 부트로더를 다시 잠그지 않습니다.

## 144Hz 완전 고정 후 AP500U/AP501U 펜촉 입력이 작동하지 않음

### 증상

144Hz 완전 고정을 적용한 뒤 다음 기능은 정상적으로 작동했습니다.

- 펜 Bluetooth 연결
- 펜 버튼
- 화면 근접 감지
- 펜 근접 시 손가락 입력 차단

그러나 펜촉으로 누르기, 선택, 필기 및 필압 입력은 작동하지
않았습니다.

### 문제가 발생한 설정

```text
min_refresh_rate=144.0
peak_refresh_rate=144.0
user preferred display mode=144Hz
match content frame rate preference=0
```

### 해결 방법

주사율 강제 설정을 모두 제거하고 완전히 전원을 껐다가 다시 켰습니다.

```powershell
.\adb shell settings delete system min_refresh_rate
.\adb shell settings delete system peak_refresh_rate
.\adb shell cmd display clear-user-preferred-display-mode 0
.\adb shell cmd display set-match-content-frame-rate-pref 1
.\adb shell reboot -p
```

그 뒤 시스템이 120Hz 물리 모드를 선택했고 펜촉 및 필압 입력이
정상으로 복구됐습니다.

펜을 계속 사용하면서 고정 주사율을 원하는 경우에는 120Hz 완전
고정을 사용하는 것이 안전했습니다.

```powershell
.\adb shell cmd display `
    set-user-preferred-display-mode `
    1840 2944 120.00001 0

.\adb shell settings put system min_refresh_rate 120.0
.\adb shell settings put system peak_refresh_rate 120.0
.\adb shell cmd display set-match-content-frame-rate-pref 0
.\adb reboot
```

이 현상은 한 대의 TB373FU 글로벌 롬 환경에서 재현하고 검증한
결과이므로, 다른 모델과 펌웨어에서는 별도 검증이 필요합니다.
