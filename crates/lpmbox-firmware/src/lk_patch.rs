//! TB375FC/TB373FU LK compatibility preparation (no network downloads).
//! The 084 stock LK input and the patched result are both pinned by SHA-256.
//! Never patch an unrecognized 095/107 LK or guess offsets for a newer LK.

use lpmbox_core::{LpmError, Result, RomRegion, ScatterPartition};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub const LPMBOX_PATCHED_LK_FILENAME:&str="LPMBOX_lk.img";

pub const ROW_084_STOCK_SHA256: &str =
    "44b79fda69b2693502982dae8d22c99a9aee9704c929e0f5ce7549e803f69903";
pub const ROW_084_PATCHED_SHA256: &str =
    "f277c69c122e6f1c88e0fdf71c10ca2b07abf921da04550dec5ce98e04abca33";
pub const PRC_103_STOCK_SHA256: &str =
    "6051d385ed8292dc270b2b0a0e1493da2c427bca0454a94c5e32bf1b6e20d428";
pub const PRC_103_PATCHED_SHA256: &str =
    "f1e5b7f3a2cc9ead1c4fc2f57821e27a9ba5a49410cd7d91e1e9922087bbfa66";

// User-provided LPMBOX-LK release 1.0.0. This is the PRC103-compatible
// boot-reported cross-region candidate, NOT the older ROW084 patched file.
// Include it in the binary so installed EXEs never depend on CWD or downloads.
pub const TB375_RELEASE_SHA256: &str = PRC_103_PATCHED_SHA256;
pub const TB375_RELEASE_SIZE: usize = 3_372_448;
const TB375_RELEASE_BYTES: &[u8] =
    include_bytes!("../assets/TB375FC.TB373FU_lk_patched.img");

pub fn verified_tb375_release_lk() -> Result<&'static [u8]> {
    if TB375_RELEASE_BYTES.len() != TB375_RELEASE_SIZE {
        return Err(invalid("내장 TB375FC/TB373FU LK의 길이 검증 실패"));
    }
    verify_sha256(TB375_RELEASE_BYTES, TB375_RELEASE_SHA256, "내장 TB375FC/TB373FU LK")?;
    if !TB375_RELEASE_BYTES.starts_with(&[0x88, 0x16, 0x88, 0x58]) {
        return Err(invalid("내장 TB375FC/TB373FU LK의 인증 컨테이너 헤더 검증 실패"));
    }
    Ok(TB375_RELEASE_BYTES)
}
const PRC_103_PATCH_BLOB: &str = concat!(
    "a05806076086769316020403210095d997ba0e81289aac0e65aaf0d5e1b2b24fff31d90307",
    "1da2d0fffb8b6622a7060760867693160201032100bb7cb135755fb921eadeabb65e8a6867",
    "6549be7684a8c4196f41dda9eeb8d3d0"
);
const ROW_084_STOCK_SIZE: usize = 3_372_368;
const ROW_084_PATCHED_SIZE: usize = 3_372_448;
const PATCH_BLOB_084: &str = concat!(
    "a05806076086769316020403210095d997ba0e81289aac0e65aaf0d5e1b2b24fff31d90307",
    "1da2d0fffb8b6622a7060760867693160201032100a4fa2095475ed73fcb0c17502290a35d",
    "4b1a354a10039f046332cb038829d71c"
);

fn invalid(message: impl Into<String>) -> LpmError {
    LpmError::InvalidFirmwareFolder(message.into())
}

pub fn is_tb375_family(model: &str) -> bool {
    model.eq_ignore_ascii_case("TB375FC") || model.eq_ignore_ascii_case("TB373FU")
}

fn sha256_hex(data: &[u8]) -> String {
    Sha256::digest(data).iter().map(|byte| format!("{byte:02x}")).collect()
}

fn verify_sha256(data: &[u8], expected: &str, label: &str) -> Result<()> {
    let actual = sha256_hex(data);
    if actual != expected {
        return Err(invalid(format!(
            "{label} SHA-256 검증 실패: expected={expected}, actual={actual}"
        )));
    }
    Ok(())
}

fn hex_bytes(hex: &str) -> Result<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return Err(invalid("084 LK 패치 데이터 길이가 잘못되었습니다."));
    }
    let bytes = hex
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let s = std::str::from_utf8(pair).map_err(|err| invalid(err.to_string()))?;
            u8::from_str_radix(s, 16).map_err(|err| invalid(err.to_string()))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(bytes)
}

/// Port of the MIT-licensed ifg/TB375FC-LK-patch v1.1.0, ROW 084 entry only.
/// The original vendor LK is unmodified; the full output is SHA-256-verified.
pub fn patch_row_084_lk(stock: &[u8]) -> Result<Vec<u8>> {
    if stock.len() != ROW_084_STOCK_SIZE {
        return Err(invalid(format!("084 순정 LK 크기 불일치: {}", stock.len())));
    }
    verify_sha256(stock, ROW_084_STOCK_SHA256, "ROW 084 순정 LK")?;

    const E1_OFF: usize = 0x1065c;
    const E2_OFF: usize = 0x103bc0;
    const E3_OFF: usize = 0x1039c0;
    if stock[E1_OFF..E1_OFF + 4] != [0xff, 0x03, 0x01, 0xd1] {
        return Err(invalid("084 LK 0x1065c 기존 명령 검증 실패"));
    }

    let mut area = hex_bytes(PATCH_BLOB_084)?;
    area.extend_from_slice(&stock[E2_OFF..E2_OFF + 995]);
    if area.len() > 1088 {
        return Err(invalid("084 LK 패치 영역 크기 초과"));
    }
    area.resize(1088, 0);
    let mut patched = Vec::with_capacity(ROW_084_PATCHED_SIZE);
    patched.extend_from_slice(&stock[..E2_OFF]);
    patched.extend_from_slice(&area);
    patched.extend_from_slice(&stock[E2_OFF + 1008..]);
    if patched.len() != ROW_084_PATCHED_SIZE {
        return Err(invalid("084 LK 패치 후 크기 불일치"));
    }
    patched[E3_OFF + 4..E3_OFF + 8].copy_from_slice(&1085u32.to_le_bytes());
    patched[E1_OFF..E1_OFF + 4].copy_from_slice(&[0xc0, 0x03, 0x5f, 0xd6]);
    verify_sha256(&patched, ROW_084_PATCHED_SHA256, "ROW 084 패치 LK")?;
    Ok(patched)
}

/// Official PRC 1.5.10.103 LK uses the same verified structural patch
/// with a different embedded data blob. All other unknown LK versions fail
/// closed and are NOT dynamically rewritten.
pub fn patch_prc_103_lk(stock: &[u8]) -> Result<Vec<u8>> {
    if stock.len() != ROW_084_STOCK_SIZE {
        return Err(invalid("PRC 103 LK 크기 불일치"));
    }
    verify_sha256(stock, PRC_103_STOCK_SHA256, "PRC 103 원본 LK")?;

    const E1_OFF: usize = 0x1065c;
    const E2_OFF: usize = 0x103bc0;
    const E3_OFF: usize = 0x1039c0;
    if stock[E1_OFF..E1_OFF + 4] != [0xff, 0x03, 0x01, 0xd1] {
        return Err(invalid("PRC 103 LK 원본 명령 검증 실패"));
    }
    let mut area = hex_bytes(PRC_103_PATCH_BLOB)?;
    area.extend_from_slice(&stock[E2_OFF..E2_OFF + 995]);
    if area.len() > 1088 {
        return Err(invalid("PRC 103 LK 패치 영역 초과"));
    }
    area.resize(1088, 0);
    let mut patched = Vec::with_capacity(ROW_084_PATCHED_SIZE);
    patched.extend_from_slice(&stock[..E2_OFF]);
    patched.extend_from_slice(&area);
    patched.extend_from_slice(&stock[E2_OFF + 1008..]);
    if patched.len() != ROW_084_PATCHED_SIZE {
        return Err(invalid("PRC 103 LK 결과 크기 불일치"));
    }
    patched[E3_OFF + 4..E3_OFF + 8].copy_from_slice(&1085u32.to_le_bytes());
    patched[E1_OFF..E1_OFF + 4].copy_from_slice(&[0xc0, 0x03, 0x5f, 0xd6]);
    verify_sha256(&patched, PRC_103_PATCHED_SHA256, "PRC 103 패치 LK")?;
    Ok(patched)
}

/// Experimental candidate patch for another PRC LK with exactly the same
/// RSA-4096 / cert2 container layout as the verified PRC 103 build.
/// Unknown layouts are never byte-patched. Unlike the pinned PRC 103 output,
/// cryptographic acceptance of this output is UNVERIFIED on real hardware.
fn patch_prc_103_compatible_layout(stock: &[u8]) -> Result<Vec<u8>> {
    const E1: usize = 0x1065c;
    const E2: usize = 0x103bc0;
    const E3: usize = 0x1039c0;
    if stock.len() != ROW_084_STOCK_SIZE {
        return Err(invalid(format!(
            "미확인 PRC LK 길이 {}: PRC 103과 동일한 cert2 구조가 아닙니다.",stock.len()
        )));
    }
    if stock[..4] != [0x88,0x16,0x88,0x58]
        || stock[E1..E1+4] != [0xff,0x03,0x01,0xd1]
        || stock[E3..E3+4] != [0x88,0x16,0x88,0x58]
        || stock[E3+4..E3+8] != 995_u32.to_le_bytes()
        || stock[E3+8..E3+13] != *b"cert2"
        || stock[E2..E2+4] != [0x30,0x82,0x03,0xdf]
        || stock[E2+995..E2+1008].iter().any(|byte| *byte != 0)
    {
        return Err(invalid(
            "미확인 PRC LK에 PRC 103의 RSA-4096/cert2 레이아웃을 확인할 수 없어              안전하지 않은 바이트 패치를 거부합니다."
        ));
    }

    let mut replacement = hex_bytes(PRC_103_PATCH_BLOB)?;
    replacement.extend_from_slice(&stock[E2..E2+995]);
    if replacement.len() > 1088 {
        return Err(invalid("PRC LK 패치 영역 1088B 초과"));
    }
    replacement.resize(1088,0);
    let mut output=Vec::with_capacity(ROW_084_PATCHED_SIZE);
    output.extend_from_slice(&stock[..E2]);
    output.extend_from_slice(&replacement);
    output.extend_from_slice(&stock[E2+1008..]);
    if output.len() != ROW_084_PATCHED_SIZE {
        return Err(invalid("PRC LK 패치 결과 크기 불일치"));
    }
    output[E3+4..E3+8].copy_from_slice(&1085_u32.to_le_bytes());
    output[E1..E1+4].copy_from_slice(&[0xc0,0x03,0x5f,0xd6]);
    if output[E2..E2+hex_bytes(PRC_103_PATCH_BLOB)?.len()]
            != hex_bytes(PRC_103_PATCH_BLOB)?
        || output[E3..E3+4] != stock[E3..E3+4]
        || output[E3+4..E3+8] != 1085_u32.to_le_bytes()
    {
        return Err(invalid("PRC LK 패치 후 cert2 헤더 무결성 검사 실패"));
    }
    Ok(output)
}

/// A SPFT partition dump is often 16 MiB; use only a recognized official LK
/// prefix followed by inert 00/FF padding. Never trim unknown signed code.
fn known_dump_prefix<'a>(dump: &'a [u8], prefix_len: usize, expected: &str) -> Option<&'a [u8]> {
    if dump.len() < prefix_len || dump.len() > 32 * 1024 * 1024 {
        return None;
    }
    let prefix = &dump[..prefix_len];
    let trailing = &dump[prefix_len..];
    let padded = trailing.is_empty()
        || trailing.iter().all(|b| *b == 0)
        || trailing.iter().all(|b| *b == 0xff);
    (padded && sha256_hex(prefix) == expected).then_some(prefix)
}

/// Select a known image from an active-slot LK readback (no device writes).
/// Unknown older/newer LK is *not* blindly patched. Use only verified 084 shim.
fn patch_active_slot_dump<F>(
    dump_path: &Path, image_dir: &Path, on_log: &mut F
) -> Result<Vec<u8>>
where F: FnMut(String) {
    let dump = fs::read(dump_path)?;
    if dump.len() < ROW_084_STOCK_SIZE || dump.len() > 32 * 1024 * 1024
        || !dump.starts_with(&[0x88,0x16,0x88,0x58]) {
        return Err(invalid(format!(
            "활성 슬롯 LK 리드백 이미지 헤더/길이 검증 실패: {} ({}바이트)",
            dump_path.display(), dump.len()
        )));
    }
    on_log(format!(
        "[LK] 활성 LK 리드백: {} / {}바이트 / SHA-256 {}",
        dump_path.display(), dump.len(), sha256_hex(&dump)
    ));
    if let Some(stock) = known_dump_prefix(&dump, ROW_084_STOCK_SIZE, PRC_103_STOCK_SHA256) {
        on_log("[LK] PRC 1.5.10.103 원본 LK 식별: 현재 슬롯 LK를 직접 패치합니다.".into());
        return patch_prc_103_lk(stock);
    }
    if let Some(stock) = known_dump_prefix(&dump, ROW_084_STOCK_SIZE, ROW_084_STOCK_SHA256) {
        on_log("[LK] ROW 17.5.10.084 원본 LK 식별: 현재 슬롯 LK를 직접 패치합니다.".into());
        return patch_row_084_lk(stock);
    }
    for (hash,label) in [
        (PRC_103_PATCHED_SHA256,"PRC 103"),
        (ROW_084_PATCHED_SHA256,"ROW 084"),
    ] {
        if let Some(stock) = known_dump_prefix(&dump, ROW_084_PATCHED_SIZE, hash) {
            on_log(format!("[LK] 현재 슬롯에서 검증된 {label} 패치 LK 확인: 중복 패치를 생략합니다."));
            return Ok(stock.to_vec());
        }
    }
    on_log(
        "[LK] 읽은 LK는 기존 패치 원본 목록에 없습니다(PRC 060, ROW 057/095/107 등 가능). \
         임의 바이트 패치는 하지 않고, SHA-256이 검증된 LPMBOX-LK 1.0.0 \
         TB375FC/TB373FU 공통 LK를 사용합니다. 모델·펌웨어별 정상 부팅은 별도 검증이 필요합니다.".into()
    );
    Ok(verified_tb375_release_lk()?.to_vec())
}

/// ROW 17.5.10.057 and later: accept recognized ROW version generations
/// only when the exact SHA-pinned, bundled LPMBOX-LK image is available.
/// A filename or version alone is never proof of secure-boot compatibility.
pub fn validate_tb373_row_release(version: Option<&str>) -> Result<()> {
    if !supported_row_version(version) {
        return Err(invalid(format!(
            "TB373FU ROW 버전 표기 오류 또는 미지원 세대: {}",
            version.unwrap_or("UNKNOWN")
        )));
    }
    verified_tb375_release_lk()?;
    Ok(())
}

/// TB373FU ZUI 17.5.10.057 and later: validate the version grammar without
/// reusing the former 084 threshold. The exact new release image is
/// independently SHA-pinned; neither the version nor the hash alone proves
/// successful vendor secure boot on every firmware generation.
pub fn supported_row_version(version: Option<&str>) -> bool {
    let Some(raw) = version else { return false; };
    let Some(rest) = raw.trim().strip_prefix("ZUI_") else { return false; };
    let Some((numbers, suffix)) = rest.split_once('_') else { return false; };
    if suffix != "ST" { return false; }
    let values: Vec<u32> = numbers.split('.')
        .map(str::parse::<u32>)
        .collect::<std::result::Result<_, _>>()
        .unwrap_or_default();
    if values.len() != 4 { return false; }
    (values[0], values[1], values[2], values[3]) >= (17, 5, 10, 57)
}

// Preserve the old >=084 live-readback entry condition, even though
// ROW057 can now use the SHA-pinned embedded release image offline.
pub fn row_live_readback_required(version: Option<&str>) -> bool {
    if !supported_row_version(version) { return false; }
    let Some(version) = version.and_then(|v| v.strip_prefix("ZUI_"))
        .and_then(|v| v.strip_suffix("_ST")) else { return false; };
    let Ok(parts) = version.split('.').map(str::parse::<u32>)
        .collect::<std::result::Result<Vec<_>,_>>() else { return false; };
    parts.len()==4 && (parts[0],parts[1],parts[2],parts[3]) >= (17,5,10,84)
}


/// PRC 16.0.x / ZUI 17.5.10.060 and earlier require no LK modification.
/// The newer PRC chain has only a verified LK patch for 1.5.10.103.
/// Unknown versions MUST NOT be rewritten by guessing patch offsets.
pub fn prc_lk_patch_required(version: Option<&str>) -> Result<bool> {
    let raw = version.ok_or_else(||invalid(
        "PRC 버전을 판별할 수 없어 순정 LK와 패치 LK 중 하나를 선택하지 못했습니다."
    ))?.trim();
    let prefix = if let Some(v) = raw.strip_prefix("ZUI_") {
        ("ZUI",v)
    } else if let Some(v) = raw.strip_prefix("ZUXOS_").or_else(||raw.strip_prefix("ZUX_")) {
        ("ZUX",v)
    } else {
        return Err(invalid(format!("지원되지 않는 PRC 버전 표기: {raw}")));
    };
    let numbers = prefix.1.strip_suffix("_ST").unwrap_or(prefix.1);
    let values: Vec<u32> = numbers.split('.')
        .map(str::parse::<u32>)
        .collect::<std::result::Result<_,_>>()
        .map_err(|_|invalid(format!("PRC 버전 비교 실패: {raw}")))?;
    if values.len() != 4 {
        return Err(invalid(format!("PRC 버전 값이 네 구간이 아닙니다: {raw}")));
    }
    let version = (values[0],values[1],values[2],values[3]);
    Ok(if prefix.0 == "ZUX" {
        version > (1,5,10,60)
    } else {
        version > (17,5,10,60)
    })
}

fn validate_prc_stock_images(image_dir: &Path) -> Result<()> {
    let lk = fs::read(image_dir.join("lk.img"))?;
    let dtbo = fs::read(image_dir.join("dtbo.img"))?;
    if lk.len() < 1024 || !lk.starts_with(&[0x88,0x16,0x88,0x58]) {
        return Err(invalid("PRC 순정 LK 헤더 또는 크기가 유효하지 않습니다."));
    }
    if dtbo.len() < 32 || dtbo.len() > 16*1024*1024
        || !dtbo.starts_with(&[0xd7,0xb7,0xab,0x1e]) {
        return Err(invalid(format!(
            "PRC 순정 DTBO 헤더/길이 검증 실패: {} ({}바이트)",
            image_dir.join("dtbo.img").display(),dtbo.len()
        )));
    }
    let header_size = u32::from_be_bytes(dtbo[4..8].try_into().unwrap()) as usize;
    if header_size < 32 || header_size > dtbo.len() {
        return Err(invalid(format!(
            "PRC DTBO 헤더에 기록된 크기 {}가 실제 길이 {}를 초과합니다.",
            header_size,dtbo.len()
        )));
    }
    Ok(())
}

/// On PRC the original DTBO is never repackaged. An LK patch is applied
/// only if the version requires it AND the original is a pinned 103 image.
pub fn prepare_prc_lk_for_version<F>(
    image_dir: &Path, version: Option<&str>, on_log: &mut F
) -> Result<bool>
where F: FnMut(String) {
    validate_prc_stock_images(image_dir)?;
    if !prc_lk_patch_required(version)? {
        on_log("[LK] PRC 순정 LK/DTBO 및 원래 scatter 파일 경로를 유지합니다. \
                다운로드·파일 교체·LK 패치가 필요하지 않습니다.".into());
        return Ok(false);
    }
    let stock = fs::read(image_dir.join("lk.img"))?;
    let stock_sha=sha256_hex(&stock);
    let (patched, verified_reference) = if stock_sha == PRC_103_STOCK_SHA256 {
        (patch_prc_103_lk(&stock)?, true)
    } else if stock_sha == PRC_103_PATCHED_SHA256 {
        (stock, true)
    } else {
        // Official 103 must retain its pinned hash: a corrupted 103 must
        // never be treated as an unknown future experimental build.
        if matches!(version, Some("ZUI_17.5.10.103_ST"
            | "ZUX_1.5.10.103_ST" | "ZUXOS_1.5.10.103_ST")) {
            return Err(invalid(format!(
                "PRC 103 순정 LK SHA-256 미일치: {}. 공식 기준값을 확인해주세요.",
                stock_sha
            )));
        }
        // Extend PRC >060 to later builds that *share* the known cert2 layout.
        // Signed boot acceptance remains unverified for any unknown input.
        match patch_prc_103_compatible_layout(&stock) {
            Ok(candidate) => {
                on_log(format!(
                    "[LK][실험] PRC {}의 LK가 103 cert2 구조와 호환되어 동일한 패치 규칙을 적용합니다. \
                     원본 SHA-256={}, 결과 SHA-256={}. 이 버전의 부팅/서명은 미검증입니다.",
                    version.unwrap_or("UNKNOWN"),stock_sha,sha256_hex(&candidate)
                ));
                (candidate,false)
            }
            Err(_) => {
                // The source still passed validate_prc_stock_images, but its
                // cert2 offsets differ. Never modify its unknown signed bytes.
                // Use only the boot-reported, SHA-pinned release image.
                let bundled=verified_tb375_release_lk()?.to_vec();
                on_log(format!(
                    "[LK] PRC {}는 기존 103 패치 구조와 달라 바이트 패치를 생략합니다. \
                     LPMBOX-LK 1.0.0 공통 이미지(SHA-256 {})를 별도로 배치합니다. \
                     이 버전의 장치별 AVB/서명 호환성은 미확인입니다.",
                    version.unwrap_or("UNKNOWN"),TB375_RELEASE_SHA256
                ));
                (bundled,true)
            }
        }
    };
    let output_sha=sha256_hex(&patched);
    write_checked(
        &image_dir.join(LPMBOX_PATCHED_LK_FILENAME),&patched,&output_sha
    )?;
    on_log(format!(
        "[LK] PRC {} LK 패치 결과 준비 완료: SHA-256={} / 공식 참조 출력 검증={} /          순정 lk.img·dtbo.img·원본 scatter.xml은 변경하지 않음.",
        version.unwrap_or("UNKNOWN"),output_sha,verified_reference
    ));
    Ok(true)
}

/// Only the four LK/DTBO source references are updated. Other scatter
/// partition flags, metadata and existing install sequence are untouched.
pub fn keep_prc_stock_image_references(
    partitions: &mut [ScatterPartition], patched_lk: bool
) {
    for part in partitions {
        let source = match part.name.to_ascii_lowercase().as_str() {
            "lk_a" | "lk_b" => if patched_lk {
                LPMBOX_PATCHED_LK_FILENAME
            } else { "lk.img" },
            "dtbo_a" | "dtbo_b" => "dtbo.img",
            _ => continue
        };
        part.file_name = Some(source.to_string());
    }
}

fn stock_084_candidates(image_dir: &Path) -> Vec<PathBuf> {
    let root = lpmbox_core::app_paths::runtime_root();
    let mut candidates = vec![
        image_dir.join("lk.img"),
        image_dir.join("ROW_084_lk.img"),
        root.join("tools/TB375FC/ROW_084_lk.img"),
    ];
    // Find a peer 084 firmware extracted beside the selected 095/107 package.
    // Some firmware ZIPs contain a second same-named folder inside the first.
    for ancestor in image_dir.ancestors().take(5) {
        let Ok(entries) = fs::read_dir(ancestor) else { continue };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let name = entry.file_name().to_string_lossy().to_ascii_uppercase();
            if !name.contains("TB373FU") || !name.contains("17.5.10.084") {
                continue;
            }
            candidates.push(path.join("image/lk.img"));
            candidates.push(path.join(entry.file_name()).join("image/lk.img"));
        }
    }
    candidates
}

fn trusted_row_084_patched(image_dir: &Path) -> Result<Vec<u8>> {
    // Prefer performing the exact 084 transformation during firmware staging
    // when the known official stock image is available. Never patch a random LK.
    for file in stock_084_candidates(image_dir) {
        if !file.is_file() { continue; }
        let data = fs::read(&file)?;
        if sha256_hex(&data) == ROW_084_STOCK_SHA256 {
            return patch_row_084_lk(&data);
        }
    }

    // A pre-patched file is a fully verified offline fallback when the
    // original 084 package is not available on the user's computer.
    let mut patched_candidates = vec![image_dir.join("ROW_084_lk_patched.img")];
    let mut roots = vec![lpmbox_core::app_paths::runtime_root()];
    if let Ok(cwd) = std::env::current_dir() {
        roots.extend(cwd.ancestors().take(5).map(Path::to_path_buf));
    }
    if let Ok(exe) = std::env::current_exe() {
        roots.extend(exe.ancestors().skip(1).take(5).map(Path::to_path_buf));
    }
    roots.sort();
    roots.dedup();
    for root in roots {
        patched_candidates.push(root.join("tools/TB375FC/ROW_084_lk_patched.img"));
    }
    for file in &patched_candidates {
        if !file.is_file() { continue; }
        let data = fs::read(file)?;
        verify_sha256(&data, ROW_084_PATCHED_SHA256, &file.display().to_string())?;
        if data.len() != ROW_084_PATCHED_SIZE {
            return Err(invalid("ROW 084 패치 LK 크기 불일치"));
        }
        return Ok(data);
    }

    Err(invalid(
        "검증된 ROW 084 LK를 찾지 못했습니다. tools/TB375FC/ROW_084_lk_patched.img \
         (SHA-256 고정 검증) 또는 공식 ROW 17.5.10.084의 image/lk.img가 필요합니다. \
         구버전 GitHub LK ZIP이나 095/107 순정 LK로 대체하지 않습니다."
    ))
}

fn write_checked(path: &Path, data: &[u8], expected_sha256: &str) -> Result<()> {
    verify_sha256(data, expected_sha256, &path.display().to_string())?;
    fs::write(path, data)?;
    let written = fs::read(path)?;
    verify_sha256(&written, expected_sha256, &path.display().to_string())
}

fn prepare_dtbo(image_dir: &Path) -> Result<Vec<u8>> {
    let path = image_dir.join("dtbo.img");
    let data = fs::read(&path).map_err(|err| {
        invalid(format!("선택한 펌웨어 자체의 dtbo.img가 필요합니다: {} / {err}", path.display()))
    })?;
    if data.len() < 32 || data.len() > 16 * 1024 * 1024
        || !data.starts_with(&[0xd7, 0xb7, 0xab, 0x1e]) {
        return Err(invalid(format!(
            "선택한 펌웨어의 DTBO 크기/헤더 검증 실패: {}", path.display()
        )));
    }
    let declared=u32::from_be_bytes(data[4..8].try_into().unwrap()) as usize;
    if declared<32 || declared>data.len() {
        return Err(invalid("선택한 펌웨어의 DTBO 내부 길이가 실제 파일보다 큽니다."));
    }
    Ok(data)
}

/// Populate only the four established LK/DTBO destination files.
/// No DA commands, network I/O, slot changes, or reboot occurs here.
pub fn prepare_local_lk_dtbo<F>(
    region: RomRegion,
    version: Option<&str>,
    image_dir: &Path,
    on_log: &mut F,
) -> Result<()>
where F: FnMut(String) {
    prepare_local_lk_dtbo_from_readback(region, version, image_dir, None, on_log)
}

pub fn prepare_local_lk_dtbo_from_readback<F>(
    region: RomRegion,
    version: Option<&str>,
    image_dir: &Path,
    active_lk_readback: Option<&Path>,
    on_log: &mut F,
) -> Result<()>
where F: FnMut(String) {
    let lk = match region {
        RomRegion::Row => {
            validate_tb373_row_release(version)?;
            if !matches!(version, Some("ZUI_17.5.10.084_ST"
                | "ZUI_17.5.10.095_ST" | "ZUI_17.5.10.107_ST"))
            {
                on_log(format!(
                    "[LK] 실험적 버전 {}: 펌웨어 차단은 해제했지만,                      084 패치 LK와 이 버전의 AVB/서명 호환성은 실기기 미검증입니다.",
                    version.unwrap_or("UNKNOWN")
                ));
            }
            on_log("[LK] LPMBOX-LK 1.0.0 사용자 제공 TB375FC/TB373FU 패치 LK를 내장 SHA-256으로 검증합니다. 기존 ROW084 패치와 다른 PRC103 계열 후보입니다.".to_string());
            if let Some(dump) = active_lk_readback {
                // Keep the original readback validation and log stages.
                // Never patch unknown stock offsets or change a device here.
                let _ = patch_active_slot_dump(dump, image_dir, on_log)?;
            }
            verified_tb375_release_lk()?.to_vec()
        },
        RomRegion::Prc => {
            let stock = fs::read(image_dir.join("lk.img")).map_err(|err| {
                invalid(format!("PRC 순정 LK를 찾지 못했습니다: {err}"))
            })?;
            if stock.len() < 1024 || !stock.starts_with(&[0x88,0x16,0x88,0x58]) {
                return Err(invalid("PRC 순정 LK 헤더/크기 검증 실패"));
            }
            on_log("[LK] PRC 자체 순정 LK를 사용합니다. 구버전 GitHub LK 다운로드를 생략합니다.".to_string());
            stock
        },
        RomRegion::Unknown => return Err(invalid("LK 패치를 적용할 PRC/ROW 지역을 확인하지 못했습니다.")),
    };
    let dtbo = prepare_dtbo(image_dir)?;
    let lk_sha = sha256_hex(&lk);
    let dtbo_sha = sha256_hex(&dtbo);
    write_checked(&image_dir.join(LPMBOX_PATCHED_LK_FILENAME), &lk, &lk_sha)?;
    for part in ["dtbo_a","dtbo_b"] {
        write_checked(&image_dir.join(part), &dtbo, &dtbo_sha)?;
    }
    on_log(format!(
        "[LK] LPMBOX_lk.img 검증 완료 (SHA-256 {}), dtbo_a/dtbo_b 선택한 펌웨어 원본 적용 (SHA-256 {})",
        lk_sha, dtbo_sha
    ));
    Ok(())
}

/// For ROW 084+ LK-shim installs, do not write device identity, slot metadata,
/// fuses, or preloader. This only tightens the selected TB373FU ROW scatter;
/// unrelated devices and dedicated country-code reset workflows are untouched.
pub fn protect_tb373_row_downloads(partitions: &mut [ScatterPartition]) -> usize {
    let mut changed = 0usize;
    for partition in partitions {
        let name = partition.name.to_ascii_lowercase();
        let protected = name.starts_with("preloader")
            || matches!(
                name.as_str(),
                "efuse" | "misc" | "para" | "boot_para" | "metadata"
            );
        if !protected { continue; }
        if partition.is_download.as_deref() != Some("false") {
            partition.is_download = Some("false".into());
            changed += 1;
        }
        if partition.is_upgradable.as_deref() != Some("false") {
            partition.is_upgradable = Some("false".into());
            changed += 1;
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn row_guard_disables_only_protected_metadata_and_preloader() {
        let names = ["preloader", "preloader_backup", "efuse", "misc", "para",
                     "boot_para", "metadata", "boot_a", "proinfo", "lk_a"];
        let mut partitions: Vec<ScatterPartition> = names.iter().map(|name| {
            ScatterPartition {
                name: name.to_string(), partition_index: None, file_name: None,
                is_download: Some("true".into()), is_upgradable: Some("true".into()),
                partition_type: None, storage: None, linear_start_addr: None,
                physical_start_addr: None, partition_size: None,
            }
        }).collect();
        assert_eq!(protect_tb373_row_downloads(&mut partitions), 14);
        for p in &partitions[..7] {
            assert_eq!(p.is_download.as_deref(), Some("false"));
            assert_eq!(p.is_upgradable.as_deref(), Some("false"));
        }
        for p in &partitions[7..] {
            assert_eq!(p.is_download.as_deref(), Some("true"));
        }
    }
    #[test]
    fn version_allowlist_is_strict() {
        assert!(supported_row_version(Some("ZUI_17.5.10.084_ST")));
        assert!(supported_row_version(Some("ZUI_17.5.10.095_ST")));
        assert!(supported_row_version(Some("ZUI_17.5.10.107_ST")));
        assert!(supported_row_version(Some("ZUI_17.5.10.057_ST")));
        assert!(supported_row_version(Some("ZUI_17.5.10.108_ST")));
        assert!(supported_row_version(Some("ZUI_17.5.10.193_ST")));
        assert!(supported_row_version(Some("ZUI_17.5.10.213_ST")));
        assert!(supported_row_version(Some("ZUI_17.5.11.001_ST")));
        assert!(supported_row_version(Some("ZUI_17.5.10.083_ST")));
        assert!(!supported_row_version(Some("ZUI_17.5.10.056_ST")));
        assert!(!supported_row_version(Some("ZUXOS_1.5.10.103_ST")));
        assert!(!supported_row_version(None));
        assert!(!row_live_readback_required(Some("ZUI_17.5.10.057_ST")));
        assert!(row_live_readback_required(Some("ZUI_17.5.10.084_ST")));
    }
    #[test]
    fn embedded_release_lk_matches_user_provided_prc103_family_sha() {
        let output=verified_tb375_release_lk().unwrap();
        assert_eq!(output.len(),TB375_RELEASE_SIZE);
        assert_eq!(sha256_hex(output),TB375_RELEASE_SHA256);
        assert_ne!(sha256_hex(output),ROW_084_PATCHED_SHA256);
    }
    #[test]
    fn row057_stages_pinned_release_without_084_or_network() {
        let dir=std::env::temp_dir().join(format!(
            "lpmbox_row057_offline_{}_{}",std::process::id(),
            std::time::SystemTime::now().duration_since(
                std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&dir).unwrap();
        let mut dtbo=vec![0u8;4096];
        dtbo[..4].copy_from_slice(&[0xd7,0xb7,0xab,0x1e]);
        dtbo[4..8].copy_from_slice(&32u32.to_be_bytes());
        fs::write(dir.join("dtbo.img"),&dtbo).unwrap();
        fs::write(dir.join("lk.img"),b"UNMODIFIED_057_STOCK").unwrap();
        let mut logs=Vec::new();
        prepare_local_lk_dtbo(RomRegion::Row,Some("ZUI_17.5.10.057_ST"),
            &dir,&mut |msg|logs.push(msg)).unwrap();
        assert_eq!(sha256_hex(&fs::read(dir.join(LPMBOX_PATCHED_LK_FILENAME)).unwrap()),
            TB375_RELEASE_SHA256);
        assert_eq!(fs::read(dir.join("lk.img")).unwrap(),b"UNMODIFIED_057_STOCK");
        assert_eq!(fs::read(dir.join("dtbo.img")).unwrap(),dtbo);
        assert!(prepare_local_lk_dtbo(RomRegion::Row,Some("ZUI_17.5.10.056_ST"),
            &dir,&mut |_|{}).is_err());
        assert!(!logs.is_empty());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn unknown_prc_post060_layout_uses_exact_release_without_changing_stock() {
        let dir=std::env::temp_dir().join(format!(
            "lpmbox_prc061_fallback_{}_{}",std::process::id(),
            std::time::SystemTime::now().duration_since(
                std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&dir).unwrap();
        let mut lk=vec![0u8;2048];
        lk[..4].copy_from_slice(&[0x88,0x16,0x88,0x58]);
        let mut dtbo=vec![0u8;4096];
        dtbo[..4].copy_from_slice(&[0xd7,0xb7,0xab,0x1e]);
        dtbo[4..8].copy_from_slice(&32u32.to_be_bytes());
        fs::write(dir.join("lk.img"),&lk).unwrap();
        fs::write(dir.join("dtbo.img"),&dtbo).unwrap();
        assert!(prepare_prc_lk_for_version(&dir,
            Some("ZUXOS_1.5.10.061_ST"),&mut |_|{}).unwrap());
        assert_eq!(sha256_hex(&fs::read(dir.join(LPMBOX_PATCHED_LK_FILENAME)).unwrap()),
            TB375_RELEASE_SHA256);
        assert_eq!(fs::read(dir.join("lk.img")).unwrap(),lk);
        assert_eq!(fs::read(dir.join("dtbo.img")).unwrap(),dtbo);
        assert!(prepare_prc_lk_for_version(&dir,
            Some("ZUXOS_1.5.10.103_ST"),&mut |_|{}).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn prc_versions_select_safe_lk_paths() {
        for ver in ["ZUI_16.0.20.091_ST", "ZUI_17.5.10.060_ST",
                    "ZUX_1.5.10.060_ST"] {
            assert!(!prc_lk_patch_required(Some(ver)).unwrap(), "{ver}");
        }
        for ver in ["ZUI_17.5.10.103_ST","ZUX_1.5.10.103_ST",
                    "ZUI_17.5.10.061_ST"] {
            assert!(prc_lk_patch_required(Some(ver)).unwrap(), "{ver}");
        }
        assert!(prc_lk_patch_required(None).is_err());
        assert!(prc_lk_patch_required(Some("bad")).is_err());
    }

    #[test]
    fn prc_stock_scatter_keeps_vendor_images() {
        let mut parts: Vec<ScatterPartition> = ["lk_a","lk_b","dtbo_a","dtbo_b","super"]
            .iter().map(|name| ScatterPartition {
                name:name.to_string(),partition_index:None,
                file_name:Some("BEFORE".into()),
                is_download:Some("true".into()),is_upgradable:Some("true".into()),
                partition_type:None,storage:None,linear_start_addr:None,
                physical_start_addr:None,partition_size:None,
            }).collect();
        keep_prc_stock_image_references(&mut parts,false);
        assert_eq!(parts[0].file_name.as_deref(),Some("lk.img"));
        assert_eq!(parts[1].file_name.as_deref(),Some("lk.img"));
        assert_eq!(parts[2].file_name.as_deref(),Some("dtbo.img"));
        assert_eq!(parts[3].file_name.as_deref(),Some("dtbo.img"));
        assert_eq!(parts[4].file_name.as_deref(),Some("BEFORE"));
        keep_prc_stock_image_references(&mut parts,true);
        assert_eq!(parts[0].file_name.as_deref(),Some(LPMBOX_PATCHED_LK_FILENAME));
        assert_eq!(parts[1].file_name.as_deref(),Some(LPMBOX_PATCHED_LK_FILENAME));
        assert_eq!(parts[2].file_name.as_deref(),Some("dtbo.img"));
        assert_eq!(parts[3].file_name.as_deref(),Some("dtbo.img"));
    }

    #[test]
    fn small_prc_dtbo_does_not_require_repacking_when_supplied() {
        let Ok(source)=std::env::var("LPMBOX_TEST_PRC091_IMAGE") else {return};
        let dir=std::env::temp_dir().join(format!("lpmbox_prc091_{}",std::process::id()));
        let _=std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for name in ["lk.img","dtbo.img"] {
            std::fs::copy(Path::new(&source).join(name),dir.join(name)).unwrap();
        }
        let stock_lk=std::fs::read(dir.join("lk.img")).unwrap();
        let stock_dtbo=std::fs::read(dir.join("dtbo.img")).unwrap();
        assert!(stock_dtbo.len() < 8*1024*1024);
        assert!(!prepare_prc_lk_for_version(
            &dir,Some("ZUI_16.0.20.091_ST"),&mut |_|{}
        ).unwrap());
        assert!(!dir.join("lk_a").exists());
        assert!(!dir.join("dtbo_a").exists());
        assert_eq!(std::fs::read(dir.join("lk.img")).unwrap(),stock_lk);
        assert_eq!(std::fs::read(dir.join("dtbo.img")).unwrap(),stock_dtbo);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn recognized_prc103_is_patched_but_vendor_lk_stays_original() {
        let Ok(source)=std::env::var("LPMBOX_TEST_PRC103_IMAGE") else {return};
        let dir=std::env::temp_dir().join(format!("lpmbox_prc103_{}",std::process::id()));
        let _=std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for name in ["lk.img","dtbo.img"] {
            std::fs::copy(Path::new(&source).join(name),dir.join(name)).unwrap();
        }
        let stock=std::fs::read(dir.join("lk.img")).unwrap();
        assert!(prepare_prc_lk_for_version(
            &dir,Some("ZUI_17.5.10.103_ST"),&mut |_|{}
        ).unwrap());
        assert_eq!(sha256_hex(&std::fs::read(dir.join(LPMBOX_PATCHED_LK_FILENAME)).unwrap()),
                       PRC_103_PATCHED_SHA256);
        assert!(!dir.join("dtbo_a").exists());
        assert_eq!(std::fs::read(dir.join("lk.img")).unwrap(),stock);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn prc_cert2_layout_candidate_matches_known_103_reference_if_supplied() {
        let Ok(path)=std::env::var("LPMBOX_TEST_PRC103_LK") else {return};
        let stock=std::fs::read(path).unwrap();
        let output=patch_prc_103_compatible_layout(&stock).unwrap();
        assert_eq!(sha256_hex(&output),PRC_103_PATCHED_SHA256);
        assert_eq!(output,patch_prc_103_lk(&stock).unwrap());
    }

    #[test]
    fn prc_cert2_unknown_structural_candidate_is_reproducible_if_supplied() {
        let Ok(path)=std::env::var("LPMBOX_TEST_PRC103_LK") else {return};
        let mut candidate=std::fs::read(path).unwrap();
        candidate[0x2000] ^= 1; // Simulate a distinct newer layout-compatible build.
        assert_ne!(sha256_hex(&candidate),PRC_103_STOCK_SHA256);
        let patched=patch_prc_103_compatible_layout(&candidate).unwrap();
        assert_eq!(patched.len(),ROW_084_PATCHED_SIZE);
        assert_eq!(patched[0x2000],candidate[0x2000]);
        assert_eq!(patched[0x1065c..0x10660],[0xc0,0x03,0x5f,0xd6]);
        assert_eq!(patched,patch_prc_103_compatible_layout(&candidate).unwrap());
        // Tamper with the cert2 container: unknown formats must be refused.
        candidate[0x1039c0] ^= 1;
        assert!(patch_prc_103_compatible_layout(&candidate).is_err());
    }

    #[test]
    fn newer_prc_candidate_stages_separate_lk_and_keeps_stock_files_if_supplied() {
        let Ok(path)=std::env::var("LPMBOX_TEST_PRC103_IMAGE") else {return};
        let dir=std::env::temp_dir().join(format!(
            "lpmbox_prc104_simulated_{}",std::process::id()
        ));
        let _=std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for f in ["lk.img","dtbo.img"] {
            std::fs::copy(Path::new(&path).join(f),dir.join(f)).unwrap();
        }
        let mut lk=std::fs::read(dir.join("lk.img")).unwrap();
        lk[0x2000] ^= 1;
        std::fs::write(dir.join("lk.img"),&lk).unwrap();
        let before_dtbo=std::fs::read(dir.join("dtbo.img")).unwrap();
        let mut logs=Vec::new();
        assert!(prepare_prc_lk_for_version(
            &dir,Some("ZUI_17.5.10.104_ST"),&mut |line|logs.push(line)
        ).unwrap());
        assert!(logs.iter().any(|line|line.contains("[실험]")));
        assert_eq!(std::fs::read(dir.join("lk.img")).unwrap(),lk);
        assert_eq!(std::fs::read(dir.join("dtbo.img")).unwrap(),before_dtbo);
        let a=std::fs::read(dir.join(LPMBOX_PATCHED_LK_FILENAME)).unwrap();
        assert_eq!(a.len(),ROW_084_PATCHED_SIZE);
        assert_ne!(sha256_hex(&a),PRC_103_PATCHED_SHA256);
        assert!(prepare_prc_lk_for_version(
            &dir,Some("ZUI_17.5.10.103_ST"),&mut |_|{}
        ).is_err()); // Known 103 must NEVER accept an altered hash.
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn prc_060_layout_must_not_receive_prc103_candidate_patch_if_supplied() {
        let Ok(path)=std::env::var("LPMBOX_TEST_PRC060_LK") else {return};
        let stock=std::fs::read(path).unwrap();
        assert!(patch_prc_103_compatible_layout(&stock).is_err());
    }

    #[test]
    fn readback_requires_known_signed_prefix_and_padding() {
        let a = vec![0u8; ROW_084_STOCK_SIZE + 12];
        assert!(known_dump_prefix(&a, ROW_084_STOCK_SIZE, PRC_103_STOCK_SHA256).is_none());
        assert!(patch_prc_103_lk(&a).is_err());
    }

    #[test]
    fn prc103_original_readback_patches_against_reference_if_supplied() {
        let Ok(path) = std::env::var("LPMBOX_TEST_PRC103_LK") else { return };
        let stock = std::fs::read(path).unwrap();
        let mut raw = stock.clone();
        raw.resize(16 * 1024 * 1024, 0);
        let prefix = known_dump_prefix(&raw, ROW_084_STOCK_SIZE, PRC_103_STOCK_SHA256).unwrap();
        let patched = patch_prc_103_lk(prefix).unwrap();
        assert_eq!(sha256_hex(&patched), PRC_103_PATCHED_SHA256);
    }

    #[test]
    fn unknown_lk_fails_closed() {
        assert!(patch_row_084_lk(&[0u8; 128]).is_err());
    }
    #[test]
    fn official_row_084_fixture_matches_reference_if_supplied() {
        let Ok(path) = std::env::var("LPMBOX_TEST_ROW084_LK") else { return };
        let original = std::fs::read(path).expect("read official ROW 084 LK");
        let original_sha = sha256_hex(&original);
        let patched = patch_row_084_lk(&original).expect("LK patch must verify");
        assert_eq!(original_sha, ROW_084_STOCK_SHA256);
        assert_eq!(sha256_hex(&patched), ROW_084_PATCHED_SHA256);
        assert_eq!(patched.len(), ROW_084_PATCHED_SIZE);
    }
    #[test]
    fn row_084_install_stage_patches_stock_before_cached_asset_if_supplied() {
        let Ok(path) = std::env::var("LPMBOX_TEST_ROW084_LK") else { return };
        let stock = std::fs::read(path).expect("official ROW 084 LK fixture");
        let dir = std::env::temp_dir()
            .join(format!("lpmbox_lk084_stock_priority_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("lk.img"), &stock).unwrap();
        std::fs::write(dir.join("ROW_084_lk_patched.img"), b"bad cache").unwrap();
        let patched = trusted_row_084_patched(&dir).expect("must patch from official stock");
        assert_eq!(sha256_hex(&patched), ROW_084_PATCHED_SHA256);
        assert_eq!(std::fs::read(dir.join("lk.img")).unwrap(), stock);
        std::fs::remove_dir_all(&dir).unwrap();
    }
    #[test]
    fn row_107_stage_preserves_stock_and_uses_verified_084_lk_if_supplied() {
        let Ok(path) = std::env::var("LPMBOX_TEST_ROW107_DTBO") else { return };
        let original = std::fs::read(&path).expect("read ROW 107 DTBO");
        let dir = std::env::temp_dir().join(format!(
            "lpmbox_tb373_lk_{}_{}", std::process::id(), "test"
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("dtbo.img"), &original).unwrap();
        std::fs::write(dir.join("lk.img"), b"STOCK-107-DO-NOT-PATCH").unwrap();
        let mut logs = Vec::new();
        prepare_local_lk_dtbo(
            RomRegion::Row, Some("ZUI_17.5.10.107_ST"), &dir,
            &mut |line| logs.push(line)
        ).expect("prepare staging from trusted 084 cache");
        let expected = verified_tb375_release_lk().unwrap();
        assert_eq!(
            std::fs::read(dir.join(LPMBOX_PATCHED_LK_FILENAME)).unwrap(), expected
        );
        for part in ["dtbo_a", "dtbo_b"] {
            assert_eq!(std::fs::read(dir.join(part)).unwrap(), original);
        }
        assert_eq!(std::fs::read(dir.join("lk.img")).unwrap(),
                   b"STOCK-107-DO-NOT-PATCH");
        assert_eq!(std::fs::read(dir.join("dtbo.img")).unwrap(), original);
        assert!(!logs.is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }
    #[test]
    fn legacy_models_are_unchanged() {
        assert!(is_tb375_family("TB375FC"));
        assert!(is_tb375_family("TB373FU"));
        assert!(!is_tb375_family("TB335FC"));
    }
}
