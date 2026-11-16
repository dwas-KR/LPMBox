//! Exact ADB CLI "reboot edl" for MTK partition tools.
//! The adb_client RebootType enum does not include EDL. Passing "edl" into
//! the native reboot adapter silently maps it to a SYSTEM reboot; avoid that.
use super::*;
use std::ffi::OsString;

fn adb_executable() -> Option<OsString> {
    let mut paths=Vec::new();
    if let Some(base)=std::env::var_os("ANDROID_HOME"){
        paths.push(PathBuf::from(base).join("platform-tools").join("adb.exe"));
    }
    if let Some(base)=std::env::var_os("ANDROID_SDK_ROOT"){
        paths.push(PathBuf::from(base).join("platform-tools").join("adb.exe"));
    }
    if let Some(base)=std::env::var_os("LOCALAPPDATA"){
        paths.push(PathBuf::from(base).join("Android").join("Sdk")
            .join("platform-tools").join("adb.exe"));
    }
    if let Ok(bin)=std::env::current_exe(){
        if let Some(parent)=bin.parent(){
            paths.push(parent.join("platform-tools").join("adb.exe"));
            paths.push(parent.join("tools").join("adb.exe"));
        }
    }
    if let Some(path)=paths.into_iter().find(|p|p.is_file()){
        return Some(path.into_os_string());
    }
    Some(OsString::from("adb.exe"))
}

fn run_adb_cli(args:&[&str])->Result<Output>{
    let executable=adb_executable().ok_or_else(||
        LpmError::Adb("ADB 명령 실행 파일을 찾지 못했습니다.".into()))?;
    let mut child=Command::new(&executable);
    child.args(args).stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    child.creation_flags(CREATE_NO_WINDOW);
    let mut process=child.spawn().map_err(|e|LpmError::Adb(format!(
        "adb.exe 실행 불가: {e}. Android SDK platform-tools 설치 또는 PATH 설정이 필요합니다."
    )))?;
    let start=Instant::now();
    loop{
        match process.try_wait(){
            Ok(Some(_))=>return process.wait_with_output().map_err(|e|
                LpmError::Adb(format!("ADB 실행 결과 수신 실패: {e}"))),
            Ok(None)=>{
                if start.elapsed()>Duration::from_secs(12){
                    let _=process.kill();
                    let _=process.wait();
                    return Err(LpmError::Adb("adb 명령 12초 응답 시간 초과".into()));
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e)=>return Err(LpmError::Adb(format!("adb 프로세스 확인 실패: {e}")))
        }
    }
}
fn parse_single_authorized_serial(stdout:&str)->Result<String>{
    let mut devices=Vec::new();
    let mut unauthorized=false;
    for line in stdout.lines(){
        let parts:Vec<&str>=line.split_whitespace().collect();
        if parts.len()<2 { continue; }
        match parts[1]{
            "device"=>devices.push(parts[0].to_string()),
            "unauthorized"=>unauthorized=true,
            _=>{}
        }
    }
    if devices.len()!=1 || unauthorized{
        return Err(LpmError::Adb(format!(
            "adb devices에서 권한 승인된 USB 기기가 정확히 1대여야 합니다. 승인된 기기 {}대, unauthorized={unauthorized}",
            devices.len()
        )));
    }
    Ok(devices.remove(0))
}

pub fn verify_partition_mtk_device<F>(mut on_log:F)->Result<String>
where F:FnMut(String){
    wait_for_adb_device_ready_until(&mut on_log,Duration::from_secs(30))?;
    let platform=adb_shell_checked("getprop ro.vendor.mediatek.platform")?;
    if !platform.trim().to_ascii_uppercase().starts_with("MT"){
        return Err(LpmError::Adb(format!(
            "MediaTek 칩셋이 감지되지 않아 파티션 작업 중단: {platform}"
        )));
    }
    let model=adb_shell_checked("getprop ro.product.model")?;
    let normalized=model.trim().to_ascii_uppercase();
    if !["TB335FC","TB336FU","TB365FC","TB361FU","TB375FC","TB373FU"]
        .iter().any(|m|normalized.contains(m)){
        return Err(LpmError::Adb(format!(
            "LPMBOX 지원 대상이 아닌 모델: {normalized}"
        )));
    }
    on_log(format!("[MTK] 연결된 기기: {normalized}; 플랫폼: {}",platform.trim()));
    Ok(normalized)
}
/// Execute the literal "adb -s <serial> reboot edl".
/// Does NOT emulate it as "adb reboot" or "adb shell reboot".
pub fn prepare_partition_preloader_serial<F>(mut on_log:F)->Result<String>
where F:FnMut(String){
    on_log("[ADB] adb devices (기기 및 권한 확인)".into());
    let list=run_adb_cli(&["devices"])?;
    if !list.status.success(){
        return Err(LpmError::Adb(format!(
            "adb devices 실패: {}",String::from_utf8_lossy(&list.stderr)
        )));
    }
    let devices=String::from_utf8_lossy(&list.stdout);
    let serial=parse_single_authorized_serial(&devices)?;
    on_log(format!("[ADB] 승인된 단일 기기 확인: {serial}"));
    Ok(serial)
}
pub fn request_partition_preloader_edl_for_serial<F>(
    serial:&str,mut on_log:F
)->Result<()>
where F:FnMut(String){
    on_log("[ADB] adb reboot edl 명령 전송 · PreLoader COM 감시 동시 실행".into());
    let result=run_adb_cli(&["-s",serial,"reboot","edl"])?;
    if !result.status.success(){
        return Err(LpmError::Adb(format!(
            "adb reboot edl 명령 실패 (기기에서 지원하지 않을 수 있음): {}",
            String::from_utf8_lossy(&result.stderr)
        )));
    }
    on_log("[ADB] reboot edl 명령 전송 완료. 실제 MediaTek PreLoader 진입은 별도 확인합니다.".into());
    Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn accepts_one_authorized_device_only(){
        assert_eq!(
            parse_single_authorized_serial("List of devices attached\nABC123\tdevice\n").unwrap(),
            "ABC123"
        );
        assert!(parse_single_authorized_serial("List of devices attached\nA\tdevice\nB\tdevice\n").is_err());
        assert!(parse_single_authorized_serial("List of devices attached\nA\tunauthorized\n").is_err());
    }
}

// Keep the original public API intact for other callers.
pub fn request_partition_preloader_edl<F>(mut on_log:F)->Result<()>
where F:FnMut(String){
    let serial=prepare_partition_preloader_serial(|line|on_log(line))?;
    request_partition_preloader_edl_for_serial(&serial,|line|on_log(line))
}

// Fastboot recovery is advisory and must not hold the UI indefinitely.
fn run_recovery_fastboot(executable:&OsString,args:&[&str])->Option<Output>{
    let mut process=Command::new(executable);
    process.args(args).stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    process.creation_flags(CREATE_NO_WINDOW);
    let mut child=process.spawn().ok()?;
    let start=Instant::now();
    loop{
        match child.try_wait(){
            Ok(Some(_))=>return child.wait_with_output().ok(),
            Ok(None)=>{
                if start.elapsed()>Duration::from_secs(8){
                    let _=child.kill();
                    let _=child.wait();
                    return None;
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(_)=>return None,
        }
    }
}
fn matching_fastboot_serial_only(output:&str,serial:&str)->bool{
    let names=output.lines().filter_map(|line|{
        let mut parts=line.split_whitespace();
        let name=parts.next()?;
        (parts.next()==Some("fastboot")).then_some(name)
    }).collect::<Vec<_>>();
    names.len()==1 && names[0]==serial
}

/// Best-effort recovery after SP Flash Tool has exited unsuccessfully.
/// A PreLoader port is not an ADB/Fastboot transport. Reboot only when a
/// matching authorized device is confirmed; never address an arbitrary USB
/// COM port with reboot or flash commands.
pub fn recover_partition_after_failure<F>(serial:&str,mut on_log:F)->bool
where F:FnMut(String){
    on_log("[복구] SP Flash Tool 종료 이후 기기 연결 상태 확인을 시작합니다.".into());
    // A failed DA load commonly leaves PreLoader transiently attached; first
    // allow the device to present itself as ADB or Fastboot.
    for attempt in 1..=3{
        if let Ok(list)=run_adb_cli(&["devices"]){
            if list.status.success(){
                let output=String::from_utf8_lossy(&list.stdout);
                if parse_single_authorized_serial(&output).ok().as_deref()==Some(serial){
                    on_log(format!("[복구] 승인된 동일 기기 ADB 확인 ({attempt}/3). adb reboot 명령을 전송합니다."));
                    match run_adb_cli(&["-s",serial,"reboot"]){
                        Ok(response) if response.status.success()=>{
                            on_log("[복구] adb reboot 전송 성공. 실제 정상 부팅 여부는 기기에서 확인하세요.".into());
                            return true;
                        }
                        Ok(response)=>{
                            on_log(format!("[복구] adb reboot 거부: {}",
                                String::from_utf8_lossy(&response.stderr)));
                        }
                        Err(err)=>on_log(format!("[복구] adb reboot 실행 실패: {err}")),
                    }
                }
            }
        }
        if attempt<3 {thread::sleep(Duration::from_millis(1000));}
    }
    // A Fastboot device may have a different serial than Android. Explicitly
    // prohibit automatic reboot unless it reports the exact known serial.
    let fastboot_binary=adb_executable()
        .and_then(|exe|{
            let path=PathBuf::from(exe);
            path.parent().map(|parent|parent.join("fastboot.exe"))
        })
        .filter(|p|p.is_file())
        .map(|p|p.into_os_string())
        .unwrap_or_else(||OsString::from("fastboot.exe"));
    if let Some(output)=run_recovery_fastboot(&fastboot_binary,&["devices"]){
        if output.status.success(){
            let devices=String::from_utf8_lossy(&output.stdout);
            if matching_fastboot_serial_only(&devices,serial){
                on_log("[복구] 동일 시리얼 Fastboot 기기 1대 확인. fastboot reboot 시도".into());
                if let Some(result)=run_recovery_fastboot(
                    &fastboot_binary,&["-s",serial,"reboot"]
                ){
                    if result.status.success(){
                        on_log("[복구] fastboot reboot 전송 성공. 정상 부팅은 확인이 필요합니다.".into());
                        return true;
                    }
                    on_log(format!("[복구] fastboot reboot 실패: {}",
                        String::from_utf8_lossy(&result.stderr)));
                }
            }
        }
    }
    on_log("[복구 안내] 같은 기기의 ADB/Fastboot가 확인되지 않아 자동 재부팅을 실행하지 않았습니다. USB 분리 후 전원 버튼을 약 15~20초 길게 눌러 재부팅하세요. 기종별 복구 방법을 우선 확인하세요.".into());
    false
}

#[cfg(test)]
mod recovery_tests{
    use super::*;
    #[test]
    fn fastboot_recovery_requires_exact_single_serial(){
        assert!(matching_fastboot_serial_only("HA2BR1CL\tfastboot\n","HA2BR1CL"));
        assert!(!matching_fastboot_serial_only("OTH\tfastboot\n","HA2BR1CL"));
        assert!(!matching_fastboot_serial_only("HA2BR1CL\tfastboot\nOTHER\tfastboot\n","HA2BR1CL"));
        assert!(!matching_fastboot_serial_only("","HA2BR1CL"));
    }
}
