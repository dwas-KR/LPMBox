use thiserror::Error;

pub type Result<T> = std::result::Result<T, LpmError>;

#[derive(Debug, Error)]
pub enum LpmError {
    #[error("파일을 찾을 수 없습니다: {0}")]
    FileNotFound(String),

    #[error("지원하지 않는 모델입니다: {0}")]
    UnsupportedModel(String),

    #[error("펌웨어 폴더가 올바르지 않습니다: {0}")]
    InvalidFirmwareFolder(String),

    #[error("차단된 펌웨어 버전입니다: {0}")]
    BlockedFirmware(String),

    #[error("실험용 LK의 Secure Boot 인증이 확인되지 않았습니다: {0}")]
    ExperimentalLkUnverified(String),

    #[error("scatter 복호화에 실패했습니다: {0}")]
    ScatterDecryptFailed(String),

    #[error("XML 파싱에 실패했습니다: {0}")]
    XmlParseFailed(String),

    #[error("ADB 오류: {0}")]
    Adb(String),

    #[error("Fastboot 오류: {0}")]
    Fastboot(String),

    #[error("SP Flash Tool 오류: {0}")]
    Spft(String),

    #[error("I/O 오류: {0}")]
    Io(#[from] std::io::Error),
}
