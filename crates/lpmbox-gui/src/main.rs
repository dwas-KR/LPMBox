#![windows_subsystem = "windows"]

use chrono::Local;
use image as image_crate;
use iced::widget::{button, column, container, pick_list, row, scrollable, text as raw_iced_text, text_input};
use iced::{window, Background, Color, Element, Font, Length, Size, Subscription, Task, Theme};
use lpmbox_core::{
    BlockedFirmwareCheck, FirmwareInfo, FlashPreparedOutput, InstallMode, PatchPlan,
    PatchValidationResult, PatchedPartitionSnapshot, ProinfoLiveEvent, ProinfoReadbackResult,
    RequiredPartitionCheck, RomRegion, SpftProgress,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const APP_DISPLAY_VERSION: &str = env!("CARGO_PKG_VERSION");

const BODY_FONT: u32 = 15;
const LOG_FONT: u32 = 12;
const INTER_FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/Inter.ttf");
const APP_ICON_PNG_BYTES: &[u8] = include_bytes!("../assets/icon.png");
const LPM_FONT_FAMILY: &str = "Malgun Gothic";
const LPMBOX_RELEASES_URL: &str = "https://github.com/dwas-KR/LPMBox/releases";
const LPMBOX_RELEASES_API_URL: &str = "https://api.github.com/repos/dwas-KR/LPMBox/releases?per_page=20";
const MODEL_TB375FC_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB375FC.png");
const MODEL_TB365FC_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB365FC.png");
const MODEL_TB335FC_IMAGE_BYTES: &[u8] = include_bytes!("../assets/models/TB335FC.png");
const ROM_INSTALL_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/earth_refresh.png");
const ROM_UPDATE_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/earth_upload.png");
const FOLDER_SELECT_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/folder_select.png");
const FOLDER_CHECK_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/folder_check.png");
const TABLET_CHECK_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/tablet_check.png");
const TABLET_X_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/tablet_x.png");
const TABLET_FIX_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/tablet_fix.png");
const SLIDE_BUTTON_BYTES: &[u8] = include_bytes!("../assets/icons/slide_button.png");
const WARNING_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/waring.png");
const NAV_HOME_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/nav_home.png");
const NAV_REFRESH_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/nav_refresh.png");
const NAV_TAB_SETTINGS_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/nav_tab_settings.png");
const NAV_FIRMWARE_DOWNLOAD_ICON_BYTES: &[u8] =
    include_bytes!("../assets/icons/nav_firmware_download.png");
const NAV_QNA_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/nav_qna.png");
const NAV_SETTINGS_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/nav_settings.png");
const NAV_LOG_ICON_BYTES: &[u8] = include_bytes!("../assets/icons/nav_log.png");
const BATTERY_RING_GREEN_BYTES: &[u8] =
    include_bytes!("../assets/icons/battery_ring_green.png");
const BATTERY_RING_GRAY_BYTES: &[u8] =
    include_bytes!("../assets/icons/battery_ring_gray.png");
const LOADING_PROGRESS_FRAME_01_BYTES: &[u8] =
    include_bytes!("../assets/icons/loading_progress_frame_01.png");
const LOADING_PROGRESS_FRAME_02_BYTES: &[u8] =
    include_bytes!("../assets/icons/loading_progress_frame_02.png");
const LOADING_PROGRESS_FRAME_03_BYTES: &[u8] =
    include_bytes!("../assets/icons/loading_progress_frame_03.png");
const LOADING_PROGRESS_FRAME_04_BYTES: &[u8] =
    include_bytes!("../assets/icons/loading_progress_frame_04.png");
const LOADING_PROGRESS_FRAME_05_BYTES: &[u8] =
    include_bytes!("../assets/icons/loading_progress_frame_05.png");
const LOADING_PROGRESS_FRAME_06_BYTES: &[u8] =
    include_bytes!("../assets/icons/loading_progress_frame_06.png");
const LOADING_PROGRESS_FRAME_07_BYTES: &[u8] =
    include_bytes!("../assets/icons/loading_progress_frame_07.png");
const LOADING_PROGRESS_FRAME_08_BYTES: &[u8] =
    include_bytes!("../assets/icons/loading_progress_frame_08.png");
const LOADING_PROGRESS_FRAME_09_BYTES: &[u8] =
    include_bytes!("../assets/icons/loading_progress_frame_09.png");
const LOADING_PROGRESS_FRAME_10_BYTES: &[u8] =
    include_bytes!("../assets/icons/loading_progress_frame_10.png");
const LOADING_PROGRESS_FRAME_11_BYTES: &[u8] =
    include_bytes!("../assets/icons/loading_progress_frame_11.png");
const LOADING_PROGRESS_FRAME_12_BYTES: &[u8] =
    include_bytes!("../assets/icons/loading_progress_frame_12.png");
const LOADING_PROGRESS_FRAME_13_BYTES: &[u8] =
    include_bytes!("../assets/icons/loading_progress_frame_13.png");
const LOADING_PROGRESS_FRAME_14_BYTES: &[u8] =
    include_bytes!("../assets/icons/loading_progress_frame_14.png");
const LOADING_PROGRESS_FRAME_15_BYTES: &[u8] =
    include_bytes!("../assets/icons/loading_progress_frame_15.png");

const LOADING_PROGRESS_FRAME_BYTES: [&[u8]; 15] = [
    LOADING_PROGRESS_FRAME_01_BYTES,
    LOADING_PROGRESS_FRAME_02_BYTES,
    LOADING_PROGRESS_FRAME_03_BYTES,
    LOADING_PROGRESS_FRAME_04_BYTES,
    LOADING_PROGRESS_FRAME_05_BYTES,
    LOADING_PROGRESS_FRAME_06_BYTES,
    LOADING_PROGRESS_FRAME_07_BYTES,
    LOADING_PROGRESS_FRAME_08_BYTES,
    LOADING_PROGRESS_FRAME_09_BYTES,
    LOADING_PROGRESS_FRAME_10_BYTES,
    LOADING_PROGRESS_FRAME_11_BYTES,
    LOADING_PROGRESS_FRAME_12_BYTES,
    LOADING_PROGRESS_FRAME_13_BYTES,
    LOADING_PROGRESS_FRAME_14_BYTES,
    LOADING_PROGRESS_FRAME_15_BYTES,
];

const LOADING_PROGRESS_SIZE: u32 = 74;
const LOADING_PROGRESS_ACTIVE_ALPHA_THRESHOLD: u8 = 220;

const SIDEBAR_RAIL_WIDTH: f32 = 64.0;
const SIDEBAR_EXPANDED_WIDTH: f32 = 210.0;
const NAV_BTN_HEIGHT: f32 = 38.0;

const LOG_WRAP_CHARS: usize = 86;

const ROM_ROUTINE_CARD_WIDTH: f32 = 620.0;
const ROM_ROUTINE_CARD_HEIGHT: f32 = 86.0;
const ROM_ROUTINE_HANDLE_WIDTH: f32 = 31.0;
const ROM_ROUTINE_HANDLE_HEIGHT: f32 = 86.0;
const ROM_ROUTINE_HANDLE_RIGHT_PADDING: f32 = 0.0;
const ROM_ROUTINE_EXPAND_WIDTH: f32 = 330.0;
const ROM_ROUTINE_SLIDE_TEXT_WIDTH: f32 = 258.0;


const ROM_HOVER_OPEN_DELAY_MS: u64 = 200;
const ROM_DIM_ALPHA: f32 = 0.50;
const UI_SPINNER_FRAMES: [&str; 4] = ["│", "╱", "━", "╲"];
const INTERNAL_SPINNER_PREFIX: &str = "\u{001e}LPMBOX_SPINNER\u{001f}";


static ACTIVE_LANGUAGE_INDEX: AtomicU8 = AtomicU8::new(1);

fn lpm_ui_display_text(content: String) -> String {
    if active_language_option() == LanguageOption::Arabic && !content.trim().is_empty() {
        format!("\u{2067}{}\u{2069}", content)
    } else {
        content
    }
}

fn iced_text<'a>(content: impl Into<String>) -> iced::widget::Text<'a> {
    raw_iced_text(lpm_ui_display_text(lpm_translate_owned(content.into())))
}

fn text<'a>(content: impl Into<String>) -> iced::widget::Text<'a> {
    iced_text(content)
}

fn active_language_option() -> LanguageOption {
    LanguageOption::from_index(ACTIVE_LANGUAGE_INDEX.load(Ordering::Relaxed))
}

fn set_active_language_option(language: LanguageOption) {
    ACTIVE_LANGUAGE_INDEX.store(language.index(), Ordering::Relaxed);
}

fn lpm_language_config_path() -> PathBuf {
    lpmbox_core::app_paths::language_config_path()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InitialLanguageSource {
    SavedConfig,
    WindowsOsLocale,
    DefaultEnglish,
}

impl InitialLanguageSource {
    fn label(self) -> &'static str {
        match self {
            InitialLanguageSource::SavedConfig => "저장된 언어 설정",
            InitialLanguageSource::WindowsOsLocale => "Windows OS 언어",
            InitialLanguageSource::DefaultEnglish => "기본값 English",
        }
    }
}

fn load_saved_language_code() -> Option<String> {
    std::fs::read_to_string(lpm_language_config_path())
        .ok()
        .map(|code| code.trim().to_string())
        .filter(|code| !code.is_empty())
}


fn save_language_option(language: LanguageOption) {
    let path = lpm_language_config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, language.code());
}


fn resolve_initial_language_option(
    saved_code: Option<&str>,
    os_locale: Option<&str>,
) -> (LanguageOption, InitialLanguageSource) {
    if let Some(language) = saved_code.and_then(LanguageOption::from_code) {
        return (language, InitialLanguageSource::SavedConfig);
    }

    if let Some(language) = os_locale.and_then(LanguageOption::from_locale) {
        return (language, InitialLanguageSource::WindowsOsLocale);
    }

    (LanguageOption::English, InitialLanguageSource::DefaultEnglish)
}

fn initial_language_option_with_source() -> (LanguageOption, InitialLanguageSource) {
    let saved_code = load_saved_language_code();
    let os_locale = sys_locale::get_locale();

    resolve_initial_language_option(saved_code.as_deref(), os_locale.as_deref())
}



fn lpm_translate_en_ru_exact(lang: LanguageOption, key: &str) -> Option<&'static str> {
    let key = key.trim();
    match lang {
        LanguageOption::English => match key {
            "대기 중" | "대기중" => Some("Idle"),
            "작업 중" => Some("Working"),
            "대시보드" | "대시 보드" => Some("Dashboard"),
            "설정 언어" => Some("System language"),
            "현재 image 폴더" => Some("Current image folder"),
            "선택된 image 폴더 없음" => Some("No image folder selected"),
            "폴더 선택" => Some("Select folder"),
            "재선택" => Some("Reselect"),
            "다시 검사" => Some("Check again"),
            "드라이버 설치" => Some("Install driver"),
            "펌웨어 검사 실패" => Some("Firmware check failed"),
            "확인 필요" => Some("Needs check"),
            "확인 불가" => Some("Cannot verify"),
            "선택 불가" => Some("Unavailable"),
            "검색 결과가 없습니다." => Some("No results found."),
            "국가 코드 또는 국가명 검색" => Some("Search country code or country name"),
            "flash.xml / scatter / DA 파일이 포함된 image 폴더를 선택해주세요." => Some("Select the image folder that contains flash.xml, scatter, and DA files."),
            "펌웨어 버전, 플랫폼, 모델명, 필수 partition 유효성, MTK 드라이버 설치 유/무를 검사합니다." => Some("Check firmware version, platform, model, required partition validity, and MTK driver installation status."),
            "모델명" => Some("Model"),
            "펌웨어 버전" => Some("Firmware version"),
            "펌웨어 유형" | "image 펌웨어 유형" => Some("Firmware type"),
            "배터리 잔량" => Some("Battery level"),
            "가동 시간" => Some("Uptime"),
            "연결한 기기" => Some("Connected device"),
            "국가 코드" => Some("Country Code"),
            "와이드바인 레벨" => Some("Widevine level"),
            "기기에 원본 롬" => Some("Original ROM"),
            "기기에 설치한 롬" | "기기에 설치된 롬" => Some("Installed ROM"),
            "설정된 슬롯 값" => Some("Active slot"),
            "하드웨어 정보" => Some("Hardware info"),
            "AP 칩셋" => Some("AP chipset"),
            "플랫폼" => Some("Platform"),
            "시스템 업데이트(OTA)" => Some("System update (OTA)"),
            "시리얼 넘버" => Some("Serial number"),
            "선택 안 됨" => Some("Not selected"),
            "알 수 없음" => Some("Unknown"),
            "차단 완료" => Some("Blocked"),
            "감지 전" => Some("Not detected"),
            "선택한 국가 코드" => Some("Selected country code"),
            "감지된 국가 코드" => Some("Detected country code"),
            "image 폴더 롬" => Some("Image folder ROM"),
            "선택한 image 폴더 정보" => Some("Selected Image Folder"),
            "image 폴더 정보를 확인합니다." => Some("Check the selected image folder."),
            "폴더 재선택" => Some("Reselect folder"),
            "다음 단계로 이동" => Some("Go to next step"),
            "다음 단계로 진행해주세요." => Some("Proceed to the next step."),
            "작업 선택" => Some("Select Task"),
            "아래 작업을 선택해서 기기에 적용합니다." => Some("Select a task below and apply it to the device."),
            "이전 메뉴로 이동" => Some("Back"),
            "옵션 선택" => Some("Select Options"),
            "ROM 작업을 시작하기 전에 세부 옵션을 설정합니다." => Some("Configure detailed options before starting the ROM task."),
            "데이터 초기화" => Some("Factory reset"),
            "활성화 할 경우 기기를 초기화 합니다." => Some("If enabled, the device will be wiped."),
            "기기에 국가 코드를 변경합니다." | "기기에 설정된 국가 코드를 변경합니다." => Some("Change the country code on the device."),
            "선택" => Some("Select"),
            "계속" => Some("Continue"),
            "시작 전 확인" => Some("Before you start"),
            "PRC ↔ ROW 설치" => Some("PRC ↔ ROW Install"),
            "ROW(글로벌롬) 업데이트" | "ROW 업데이트" => Some("ROW Update"),
            "기기 복구" => Some("Device Recovery"),
            "설치 시작" => Some("Start Install"),
            "업데이트 시작" => Some("Start Update"),
            "복구 시작" => Some("Start Recovery"),
            "PRC(중국 내수롬) 또는\nROW(글로벌롬)을 설치합니다." => Some("Install PRC or ROW ROM."),
            "ROW(글로벌롬) 버전을\n업데이트 합니다." => Some("Update the ROW ROM version."),
            "설치 실패 / 무한 재부팅 / Red State 복구" => Some("Fix install failure, boot loop, or Red State."),
            "기기가 켜지지 않거나, 무한 재부팅 등 다양한 오류를 고칩니다." => Some("Fix no-boot, boot loop, and other device errors."),
            "기기 관리" => Some("Device Management"),
            "기기를 추가적으로 설정합니다." => Some("Configure additional device options."),
            "OTA(업데이트)" => Some("OTA (Update)"),
            "업데이트 기능을 활성화 또는 비활성화로 설정합니다." => Some("Enable or disable the update feature."),
            "국가 코드 재설정" => Some("Country Code Reset"),
            "재설정" => Some("Reset"),
            "활성화" => Some("Enable"),
            "비활성화" => Some("Disable"),
            _ => None,
        },
        LanguageOption::Russian => match key {
            "대기 중" | "대기중" => Some("Ожидание"),
            "작업 중" => Some("Выполняется"),
            "대시보드" | "대시 보드" => Some("Панель управления"),
            "설정 언어" => Some("Язык системы"),
            "현재 image 폴더" => Some("Текущая папка image"),
            "선택된 image 폴더 없음" => Some("Папка image не выбрана"),
            "폴더 선택" => Some("Выбрать папку"),
            "재선택" => Some("Выбрать снова"),
            "다시 검사" => Some("Проверить снова"),
            "드라이버 설치" => Some("Установить драйвер"),
            "펌웨어 검사 실패" => Some("Ошибка проверки прошивки"),
            "확인 필요" => Some("Требуется проверка"),
            "확인 불가" => Some("Невозможно проверить"),
            "선택 불가" => Some("Недоступно"),
            "검색 결과가 없습니다." => Some("Результаты не найдены."),
            "국가 코드 또는 국가명 검색" => Some("Поиск кода или названия страны"),
            "flash.xml / scatter / DA 파일이 포함된 image 폴더를 선택해주세요." => Some("Выберите папку image, содержащую flash.xml, scatter и DA-файлы."),
            "펌웨어 버전, 플랫폼, 모델명, 필수 partition 유효성, MTK 드라이버 설치 유/무를 검사합니다." => Some("Проверка версии прошивки, платформы, модели, обязательных partition и установки MTK-драйвера."),
            "모델명" => Some("Модель"),
            "펌웨어 버전" => Some("Версия прошивки"),
            "펌웨어 유형" | "image 펌웨어 유형" => Some("Тип прошивки"),
            "배터리 잔량" => Some("Заряд батареи"),
            "가동 시간" => Some("Время работы"),
            "연결한 기기" => Some("Подключённое устройство"),
            "국가 코드" => Some("Код страны"),
            "와이드바인 레벨" => Some("Уровень Widevine"),
            "기기에 원본 롬" => Some("Исходная ROM"),
            "기기에 설치한 롬" | "기기에 설치된 롬" => Some("Установленная ROM"),
            "설정된 슬롯 값" => Some("Активный slot"),
            "하드웨어 정보" => Some("Информация об оборудовании"),
            "AP 칩셋" => Some("AP чипсет"),
            "플랫폼" => Some("Платформа"),
            "시스템 업데이트(OTA)" => Some("Системное обновление (OTA)"),
            "시리얼 넘버" => Some("Серийный номер"),
            "선택 안 됨" => Some("Не выбрано"),
            "알 수 없음" => Some("Неизвестно"),
            "차단 완료" => Some("Заблокировано"),
            "감지 전" => Some("Не обнаружено"),
            "선택한 국가 코드" => Some("Выбранный код страны"),
            "감지된 국가 코드" => Some("Обнаруженный код страны"),
            "image 폴더 롬" => Some("ROM папки image"),
            "선택한 image 폴더 정보" => Some("Выбранная папка image"),
            "image 폴더 정보를 확인합니다." => Some("Проверьте выбранную папку image."),
            "폴더 재선택" => Some("Выбрать снова"),
            "다음 단계로 이동" => Some("Перейти к следующему шагу"),
            "다음 단계로 진행해주세요." => Some("Перейдите к следующему шагу."),
            "작업 선택" => Some("Выбор задачи"),
            "아래 작업을 선택해서 기기에 적용합니다." => Some("Выберите задачу ниже и примените её к устройству."),
            "이전 메뉴로 이동" => Some("Назад"),
            "옵션 선택" => Some("Выбор параметров"),
            "ROM 작업을 시작하기 전에 세부 옵션을 설정합니다." => Some("Настройте параметры перед запуском операции ROM."),
            "데이터 초기화" => Some("Сброс данных"),
            "활성화 할 경우 기기를 초기화 합니다." => Some("Если включено, устройство будет сброшено."),
            "기기에 국가 코드를 변경합니다." | "기기에 설정된 국가 코드를 변경합니다." => Some("Изменить код страны на устройстве."),
            "선택" => Some("Выбрать"),
            "계속" => Some("Продолжить"),
            "시작 전 확인" => Some("Перед началом"),
            "PRC ↔ ROW 설치" => Some("Установка PRC ↔ ROW"),
            "ROW(글로벌롬) 업데이트" | "ROW 업데이트" => Some("Обновление ROW"),
            "기기 복구" => Some("Восстановление устройства"),
            "설치 시작" => Some("Начать установку"),
            "업데이트 시작" => Some("Начать обновление"),
            "복구 시작" => Some("Начать восстановление"),
            "PRC(중국 내수롬) 또는\nROW(글로벌롬)을 설치합니다." => Some("Установить PRC или ROW ROM."),
            "ROW(글로벌롬) 버전을\n업데이트 합니다." => Some("Обновить версию ROW ROM."),
            "설치 실패 / 무한 재부팅 / Red State 복구" => Some("Исправление сбоя установки, циклической перезагрузки или Red State."),
            "기기가 켜지지 않거나, 무한 재부팅 등 다양한 오류를 고칩니다." => Some("Исправляет отсутствие запуска, циклическую перезагрузку и другие ошибки."),
            "기기 관리" => Some("Управление устройством"),
            "기기를 추가적으로 설정합니다." => Some("Настройте дополнительные параметры устройства."),
            "OTA(업데이트)" => Some("OTA (обновление)"),
            "업데이트 기능을 활성화 또는 비활성화로 설정합니다." => Some("Включить или отключить функцию обновления."),
            "국가 코드 재설정" => Some("Сброс кода страны"),
            "재설정" => Some("Сбросить"),
            "활성화" => Some("Включить"),
            "비활성화" => Some("Отключить"),
            _ => None,
        },
        _ => None,
    }
}

fn lpm_translate_en_ru_dynamic(lang: LanguageOption, content: &str) -> Option<String> {
    if !matches!(lang, LanguageOption::English | LanguageOption::Russian) {
        return None;
    }
    let raw = content.trim();
    if let Some(rest) = raw.strip_prefix("선택한 국가 코드: ") {
        let value = if rest.trim() == "선택 안 됨" || rest.trim() == "Not selected" || rest.trim() == "Не выбрано" {
            lpm_translate_en_ru_exact(lang, "선택 안 됨").unwrap_or(rest).to_string()
        } else { rest.trim().to_string() };
        return Some(match lang {
            LanguageOption::English => format!("Selected country code: {value}"),
            LanguageOption::Russian => format!("Выбранный код страны: {value}"),
            _ => unreachable!(),
        });
    }
    if let Some(rest) = raw.strip_prefix("감지된 국가 코드: ") {
        return Some(match lang {
            LanguageOption::English => format!("Detected country code: {}", rest.trim()),
            LanguageOption::Russian => format!("Обнаруженный код страны: {}", rest.trim()),
            _ => unreachable!(),
        });
    }
    if let Some(rest) = raw.strip_prefix("image 폴더 롬: ").or_else(|| raw.strip_prefix("image 폴더 롬：")) {
        return Some(match lang {
            LanguageOption::English => format!("Image folder ROM: {}", lpm_normalize_en_ru_rom(lang, rest.trim())),
            LanguageOption::Russian => format!("ROM папки image: {}", lpm_normalize_en_ru_rom(lang, rest.trim())),
            _ => unreachable!(),
        });
    }
    if let Some(rest) = raw.strip_prefix("기기에 설치된 롬: ").or_else(|| raw.strip_prefix("기기에 설치한 롬: ")) {
        return Some(match lang {
            LanguageOption::English => format!("Installed ROM: {}", lpm_normalize_en_ru_rom(lang, rest.trim())),
            LanguageOption::Russian => format!("Установленная ROM: {}", lpm_normalize_en_ru_rom(lang, rest.trim())),
            _ => unreachable!(),
        });
    }
    if raw.contains("실제 값과 다를 수") {
        let value = raw.split('(').next().unwrap_or(raw).trim();
        return Some(match lang {
            LanguageOption::English => format!("{value} (may differ from the actual value)"),
            LanguageOption::Russian => format!("{value} (может отличаться от фактического значения)"),
            _ => unreachable!(),
        });
    }
    None
}

fn lpm_normalize_en_ru_rom(lang: LanguageOption, value: &str) -> String {
    let v = value.trim();
    if v.is_empty() {
        return lpm_translate_en_ru_exact(lang, "알 수 없음").unwrap_or("Unknown").to_string();
    }
    let upper = v.to_ascii_uppercase();
    if upper.contains("ROW") {
        return match lang {
            LanguageOption::English => "ROW (Global ROM)".to_string(),
            LanguageOption::Russian => "ROW (глобальная ROM)".to_string(),
            _ => v.to_string(),
        };
    }
    if upper.contains("PRC") {
        return match lang {
            LanguageOption::English => "PRC (China ROM)".to_string(),
            LanguageOption::Russian => "PRC (китайская ROM)".to_string(),
            _ => v.to_string(),
        };
    }
    v.to_string()
}

fn lpm_translate_en_ru_cleanup(lang: LanguageOption, content: String) -> String {
    let mut out = content;
    match lang {
        LanguageOption::English => {
            let pairs = [
                ("대기중", "Idle"), ("대기 중", "Idle"), ("작업 중", "Working"), ("실제 값과 다를 수 있음", "may differ from the actual value"),
                ("대시 보드", "Dashboard"), ("대시보드", "Dashboard"), ("설정 언어", "System language"),
                ("현재 image 폴더", "Current image folder"), ("선택된 image 폴더 없음", "No image folder selected"), ("폴더 선택", "Select folder"),
                ("재선택", "Reselect"), ("다시 검사", "Check again"), ("드라이버 설치", "Install driver"), ("펌웨어 검사 실패", "Firmware check failed"),
                ("확인 필요", "Needs check"), ("확인 불가", "Cannot verify"), ("선택 불가", "Unavailable"),
                ("모델명", "Model"), ("펌웨어 유형", "Firmware type"), ("배터리 잔량", "Battery level"), ("가동 시간", "Uptime"), ("연결한 기기", "Connected device"),
                ("기기에 설치된 롬", "Installed ROM"), ("기기에 설치한 롬", "Installed ROM"), ("image 폴더 롬", "Image folder ROM"),
                ("선택 안 됨", "Not selected"), ("선택한 국가 코드", "Selected country code"), ("감지된 국가 코드", "Detected country code"),
                ("Change the country code on the devCe.", "Change the country code on the device."),
                ("Country Code를", "Country Code"), ("device에", "device"), ("설치된 롬", "Installed ROM"),
                ("ROW (Global ROM) 버전 업데이트.", "Update the ROW ROM version."), ("ROW(Global ROM) 버전 업데이트.", "Update the ROW ROM version."),
                ("ROW (Global ROM) 버전", "ROW ROM version"), ("버전 업데이트", "version update"),
                ("설치합니다", "Install"), ("업데이트 합니다", "Update"), ("합니다", ""), ("를 ", " "), ("을 ", " "),
            ];
            for (from, to) in pairs { out = out.replace(from, to); }
        }
        LanguageOption::Russian => {
            let pairs = [
                ("대기중", "Ожидание"), ("대기 중", "Ожидание"), ("작업 중", "Выполняется"), ("실제 값과 다를 수 있음", "может отличаться от фактического значения"),
                ("Dashboard", "Панель управления"), ("ROM Tasks", "Операции ROM"), ("Additional Options", "Дополнительные параметры"), ("Settings", "Настройки"),
                ("Device Management", "Управление устройством"), ("Log Management", "Управление журналами"), ("Firmware Download", "Загрузка прошивки"),
                ("대시 보드", "Панель управления"), ("대시보드", "Панель управления"), ("설정 언어", "Язык системы"),
                ("현재 image 폴더", "Текущая папка image"), ("선택된 image 폴더 없음", "Папка image не выбрана"), ("폴더 선택", "Выбрать папку"),
                ("재선택", "Выбрать снова"), ("다시 검사", "Проверить снова"), ("드라이버 설치", "Установить драйвер"), ("펌웨어 검사 실패", "Ошибка проверки прошивки"),
                ("확인 필요", "Требуется проверка"), ("확인 불가", "Невозможно проверить"), ("선택 불가", "Недоступно"),
                ("검색 결과가 없습니다.", "Результаты не найдены."), ("국가 코드 또는 국가명 검색", "Поиск кода или названия страны"),
                ("Country Code Reset", "Сброс кода страны"), ("ROM Options", "Параметры ROM"), ("OTA update", "OTA-обновление"),
                ("detection failed", "ошибка обнаружения"), ("open failed", "не удалось открыть"), ("driver installation", "установка драйвера"),
                ("port detection", "обнаружение порта"), ("task log", "журнал задачи"), ("select again", "выбрать снова"),
                ("connected device", "подключённое устройство"), ("China ROM", "китайская ROM"), ("Global ROM", "глобальная ROM"),
                ("success", "успешно"), ("failed", "ошибка"), ("completed", "завершено"), ("passed", "пройдено"), ("all", "все"),
                ("warnings", "предупреждения"), ("errors", "ошибки"), ("current", "текущий"), ("latest", "последний"), ("source", "источник"),
                ("firmware", "прошивка"), ("update", "обновление"), ("routine", "процедура"), ("version", "версия"), ("model name", "модель"),
                ("required", "обязательно"), ("validity", "валидность"), ("status", "статус"), ("check", "проверка"), ("selected", "выбрано"),
                ("select", "выбор"), ("folder", "папка"), ("file", "файл"), ("task", "задача"), ("driver", "драйвер"),
                ("detecting", "обнаружение"), ("detected", "обнаружено"), ("try again", "повторите попытку"),
                ("saving", "сохранение"), ("save", "сохранить"), ("path", "путь"), ("previous logs", "предыдущие журналы"),
                ("backup", "резервная копия"), ("valid", "корректно"), ("retry", "повторить"),
                ("charge", "зарядить"), ("or more", "или больше"),
                ("모델명", "Модель"), ("펌웨어 유형", "Тип прошивки"), ("배터리 잔량", "Заряд батареи"), ("가동 시간", "Время работы"), ("연결한 기기", "Подключённое устройство"),
                ("기기에 설치된 롬", "Установленная ROM"), ("기기에 설치한 롬", "Установленная ROM"), ("image 폴더 롬", "ROM папки image"),
                ("선택 안 됨", "Не выбрано"), ("선택한 국가 코드", "Выбранный код страны"), ("감지된 국가 코드", "Обнаруженный код страны"),
                ("ROW (глобальная ROM) 버전 업데이트.", "Обновить версию ROW ROM."), ("ROW (Global ROM) 버전 업데이트.", "Обновить версию ROW ROM."),
                ("ROW (Global ROM)", "ROW (глобальная ROM)"), ("PRC (China ROM)", "PRC (китайская ROM)"),
                ("Country Code", "Код страны"), ("Firmware version", "Версия прошивки"), ("Firmware type", "Тип прошивки"),
                ("Installed ROM", "Установленная ROM"), ("Original ROM", "Исходная ROM"), ("Active slot", "Активный slot"),
                ("Hardware info", "Информация об оборудовании"), ("System update (OTA)", "Системное обновление (OTA)"),
                ("Serial number", "Серийный номер"), ("Unknown", "Неизвестно"), ("Blocked", "Заблокировано"),
                ("버전 업데이트", "обновить версию"), ("설치합니다", "установить"), ("업데이트 합니다", "обновить"),
                ("합니다", ""), ("를 ", " "), ("을 ", " "),
            ];
            for (from, to) in pairs { out = out.replace(from, to); }
        }
        _ => {}
    }
    normalize_localized_whitespace_preserving_lines(out)
}

fn normalize_localized_whitespace_preserving_lines(text: String) -> String {
    text.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n")
}

fn detach_spinner_frame_owned(content: String) -> (String, Option<&'static str>) {
    let trimmed = content.trim_end();
    for frame in UI_SPINNER_FRAMES {
        if let Some(base) = trimmed.strip_suffix(frame) {
            return (base.trim_end().to_string(), Some(frame));
        }
    }
    for frame in ["|", "/", "-", "\\"] {
        if let Some(base) = trimmed.strip_suffix(frame) {
            return (base.trim_end().to_string(), Some(frame));
        }
    }
    (content, None)
}

fn lpm_translate_exact_stage13(lang: LanguageOption, key: &str) -> Option<&'static str> {
    if lang.is_korean() {
        return None;
    }

    let key = key.trim();
    Some(match key {
        "모델명" => lpm_lang_text(lang, "Model", "Модель", "モデル名", "型號", "Tên mẫu", "Μοντέλο", "मॉडल", "მოდელი", "Model", "الطراز", "Modelo"),
        "펌웨어 버전" => lpm_lang_text(lang, "Firmware version", "Версия прошивки", "ファームウェアバージョン", "韌體版本", "Phiên bản firmware", "Έκδοση firmware", "फर्मवेयर संस्करण", "Firmware ვერსია", "Firmwareversie", "إصدار firmware", "Versión de firmware"),
        "펌웨어 유형" | "image 펌웨어 유형" => lpm_lang_text(lang, "Firmware type", "Тип прошивки", "ファームウェア種類", "韌體類型", "Loại firmware", "Τύπος firmware", "फर्मवेयर प्रकार", "Firmware ტიპი", "Firmwaretype", "نوع firmware", "Tipo de firmware"),
        "ROM 타입" | "ROM 유형" => lpm_lang_text(lang, "ROM type", "Тип ROM", "ROMタイプ", "ROM 類型", "Loại ROM", "Τύπος ROM", "ROM प्रकार", "ROM ტიპი", "ROM-type", "نوع ROM", "Tipo de ROM"),
        "연결한 기기" => lpm_lang_text(lang, "Connected device", "Подключённое устройство", "接続した端末", "已連接裝置", "Thiết bị đã kết nối", "Συνδεδεμένη συσκευή", "कनेक्टेड डिवाइस", "დაკავშირებული მოწყობილობა", "Verbonden apparaat", "الجهاز المتصل", "Dispositivo conectado"),
        "배터리 잔량" => lpm_lang_text(lang, "Battery level", "Заряд батареи", "バッテリー残量", "電池電量", "Pin còn lại", "Υπόλοιπο μπαταρίας", "बैटरी स्तर", "ბატარეის დონე", "Batterijniveau", "مستوى البطارية", "Nivel de batería"),
        "가동 시간" => lpm_lang_text(lang, "Uptime", "Время работы", "稼働時間", "運作時間", "Thời gian hoạt động", "Χρόνος λειτουργίας", "अपटाइम", "მუშაობის დრო", "Uptime", "مدة التشغيل", "Tiempo activo"),
        "와이드바인 레벨" => lpm_lang_text(lang, "Widevine level", "Уровень Widevine", "Widevineレベル", "Widevine 等級", "Cấp Widevine", "Επίπεδο Widevine", "Widevine स्तर", "Widevine დონე", "Widevine-niveau", "مستوى Widevine", "Nivel Widevine"),
        "실제 값과 다를 수 있음" => lpm_lang_text(lang, "may differ from the actual value", "может отличаться от фактического значения", "実際の値と異なる場合があります", "可能與實際值不同", "có thể khác giá trị thực tế", "μπορεί να διαφέρει από την πραγματική τιμή", "वास्तविक मान से भिन्न हो सकता है", "შეიძლება განსხვავდებოდეს რეალური მნიშვნელობისგან", "kan afwijken van de werkelijke waarde", "قد يختلف عن القيمة الفعلية", "puede diferir del valor real"),
        "선택 안 됨" | "선택 안함" | "select 안 됨" | "Not selected" => lpm_lang_text(lang, "Not selected", "Не выбрано", "未選択", "未選擇", "Chưa chọn", "Δεν επιλέχθηκε", "चयनित नहीं", "არ არის არჩეული", "Niet geselecteerd", "غير محدد", "No seleccionado"),
        "기기에 설치한 롬" | "기기에 설치된 롬" => lpm_lang_text(lang, "Installed ROM", "Установленная ROM", "インストール済みROM", "已安裝 ROM", "ROM đã cài", "Εγκατεστημένη ROM", "इंस्टॉल ROM", "დაყენებული ROM", "Geïnstalleerde ROM", "الروم المثبت", "ROM instalada"),
        "image 폴더 롬" => lpm_lang_text(lang, "Image folder ROM", "ROM папки image", "imageフォルダーROM", "image 資料夾 ROM", "ROM thư mục image", "ROM φακέλου image", "image फ़ोल्डर ROM", "image საქაღალდის ROM", "ROM van image-map", "ROM مجلد image", "ROM de carpeta image"),
        "기기에 원본 롬" => lpm_lang_text(lang, "Original ROM", "Исходная ROM", "元のROM", "原始 ROM", "ROM gốc", "Αρχική ROM", "मूल ROM", "საწყისი ROM", "Originele ROM", "الروم الأصلي", "ROM original"),
        "설정된 슬롯 값" => lpm_lang_text(lang, "Active slot", "Активный slot", "設定スロット値", "已設定 Slot 值", "Slot đã đặt", "Ενεργό slot", "सक्रिय slot", "აქტიური slot", "Actieve slot", "الفتحة النشطة", "Slot activo"),
        "시스템 업데이트(OTA)" => lpm_lang_text(lang, "System update (OTA)", "Системное обновление (OTA)", "システム更新（OTA）", "系統更新（OTA）", "Cập nhật hệ thống (OTA)", "Ενημέρωση συστήματος (OTA)", "सिस्टम अपडेट (OTA)", "სისტემის განახლება (OTA)", "Systeemupdate (OTA)", "تحديث النظام (OTA)", "Actualización del sistema (OTA)"),
        "하드웨어 정보" => lpm_lang_text(lang, "Hardware info", "Информация об оборудовании", "ハードウェア情報", "硬體資訊", "Thông tin phần cứng", "Πληροφορίες υλικού", "हार्डवेयर जानकारी", "აპარატურის ინფორმაცია", "Hardware-info", "معلومات العتاد", "Información de hardware"),
        "시리얼 넘버" => lpm_lang_text(lang, "Serial number", "Серийный номер", "シリアル番号", "序號", "Số sê-ri", "Σειριακός αριθμός", "सीरियल नंबर", "სერიული ნომერი", "Serienummer", "الرقم التسلسلي", "Número de serie"),
        "국가 코드" => lpm_lang_text(lang, "Country Code", "Код страны", "国コード", "國家代碼", "Mã quốc gia", "Κωδικός χώρας", "देश कोड", "ქვეყნის კოდი", "Landcode", "رمز البلد", "Código de país"),
        "경로" => lpm_lang_text(lang, "Path", "Путь", "パス", "路徑", "Đường dẫn", "Διαδρομή", "पथ", "ბილიკი", "Pad", "المسار", "Ruta"),
        "선택한 국가 코드" => lpm_lang_text(lang, "Selected country code", "Выбранный код страны", "選択した国コード", "已選擇的國家代碼", "Mã quốc gia đã chọn", "Επιλεγμένος κωδικός χώρας", "चयनित देश कोड", "არჩეული ქვეყნის კოდი", "Geselecteerde landcode", "رمز البلد المحدد", "Código de país seleccionado"),
        "감지된 국가 코드" => lpm_lang_text(lang, "Detected country code", "Обнаруженный код страны", "検出された国コード", "偵測到的國家代碼", "Mã quốc gia đã phát hiện", "Εντοπισμένος κωδικός χώρας", "पहचाना गया देश कोड", "აღმოჩენილი ქვეყნის კოდი", "Gedetecteerde landcode", "رمز البلد المكتشف", "Código de país detectado"),
        "ROW(글로벌롬) 버전 업데이트." | "ROW(글로벌 ROM) 버전 업데이트." | "ROW (Global ROM) 버전 업데이트." | "ROW (글로벌 ROM) 버전 업데이트." => lpm_lang_text(lang, "Update the ROW ROM version.", "Обновить версию ROW ROM.", "ROW ROMを更新します。", "更新 ROW ROM 版本。", "Cập nhật phiên bản ROW ROM.", "Ενημέρωση έκδοσης ROW ROM.", "ROW ROM संस्करण अपडेट करें।", "განაახლეთ ROW ROM ვერსია.", "Werk de ROW-ROM bij.", "تحديث إصدار ROW ROM.", "Actualiza la ROM ROW."),
        "활성화 할 경우 기기를 초기화 합니다." => lpm_lang_text(lang, "If enabled, the device will be wiped.", "Если включено, устройство будет сброшено.", "有効にすると端末を初期化します。", "啟用時會清除裝置資料。", "Nếu bật, thiết bị sẽ bị xóa dữ liệu.", "Αν ενεργοποιηθεί, η συσκευή θα διαγραφεί.", "चालू करने पर डिवाइस मिटा दिया जाएगा।", "ჩართვისას მოწყობილობა წაიშლება.", "Als dit is ingeschakeld, wordt het apparaat gewist.", "عند التفعيل، سيتم مسح الجهاز.", "Si se activa, se borrará el dispositivo."),
        "기기에 국가 코드를 변경합니다." | "기기에 설정된 국가 코드를 변경합니다." => lpm_lang_text(lang, "Change the country code on the device.", "Изменить код страны на устройстве.", "端末の国コードを変更します。", "變更裝置上的國家代碼。", "Đổi mã quốc gia trên thiết bị.", "Αλλαγή του κωδικού χώρας στη συσκευή.", "डिवाइस पर देश कोड बदलें।", "შეცვალეთ ქვეყნის კოდი მოწყობილობაზე.", "Wijzig de landcode op het apparaat.", "تغيير رمز البلد على الجهاز.", "Cambiar el código de país del dispositivo."),
        "PRC(중국 내수롬) 또는 ROW(글로벌롬)을 설치합니다." | "PRC 또는 ROW ROM을 설치합니다." => lpm_lang_text(lang, "Install PRC or ROW ROM.", "Установить PRC или ROW ROM.", "PRCまたはROW ROMを導入します。", "安裝 PRC 或 ROW ROM。", "Cài PRC hoặc ROW ROM.", "Εγκατάσταση PRC ή ROW ROM.", "PRC या ROW ROM इंस्टॉल करें।", "დააყენეთ PRC ან ROW ROM.", "Installeer PRC- of ROW-ROM.", "تثبيت PRC أو ROW ROM.", "Instalar PRC o ROW ROM."),
        "기기가 켜지지 않거나, 무한 재부팅 등 다양한 오류를 고칩니다." => lpm_lang_text(lang, "Fixes no boot, boot loop, and other errors.", "Исправляет отсутствие запуска, циклическую перезагрузку и другие ошибки.", "起動不可や再起動ループなどを修復します。", "修復無法開機、循環重啟和其他錯誤。", "Sửa lỗi không khởi động, lặp khởi động và lỗi khác.", "Διορθώνει μη εκκίνηση, boot loop και άλλα σφάλματα.", "नो बूट, boot loop और अन्य त्रुटियाँ ठीक करता है।", "ასწორებს არ ჩართვას, boot loop-ს და სხვა შეცდომებს.", "Herstelt niet opstarten, bootloop en andere fouten.", "إصلاح عدم الإقلاع وboot loop وأخطاء أخرى.", "Corrige no arranque, boot loop y otros errores."),
        "재설치 모드는 current slot stage 없이 PreLoader/SPFlashToolV6 단계로 진행합니다." => lpm_lang_text(lang, "Reinstall mode proceeds directly to PreLoader/SPFlashToolV6 without the current slot stage.", "Режим переустановки сразу переходит к PreLoader/SPFlashToolV6 без этапа current slot.", "再インストールモードはcurrent slot段階なしでPreLoader/SPFlashToolV6段階へ進みます。", "重新安裝模式不經 current slot 階段，直接進入 PreLoader/SPFlashToolV6 階段。", "Chế độ cài lại sẽ chuyển thẳng đến bước PreLoader/SPFlashToolV6 mà không qua bước current slot.", "Η λειτουργία επανεγκατάστασης προχωρά απευθείας στο PreLoader/SPFlashToolV6 χωρίς το στάδιο current slot.", "रीइंस्टॉल मोड current slot चरण के बिना सीधे PreLoader/SPFlashToolV6 चरण पर जाता है।", "ხელახლა ინსტალაციის რეჟიმი current slot ეტაპის გარეშე პირდაპირ PreLoader/SPFlashToolV6 ეტაპზე გადადის.", "Herinstallatiemodus gaat direct naar PreLoader/SPFlashToolV6 zonder current slot-stap.", "ينتقل وضع إعادة التثبيت مباشرة إلى PreLoader/SPFlashToolV6 بدون مرحلة current slot.", "El modo de reinstalación pasa directamente a PreLoader/SPFlashToolV6 sin la etapa current slot."),
        _ => return None,
    })
}

fn lpm_translate_dynamic_stage13(lang: LanguageOption, content: &str) -> Option<String> {
    if lang.is_korean() {
        return None;
    }

    let text = content.trim();

    for prefix in ["기기에 설치된 롬: ", "On the device, 설치된 롬: ", "On the device, installed 롬: ", "Installed ROM on device: ", "Device installed ROM: "] {
        if let Some(rest) = text.strip_prefix(prefix) {
            return Some(format!("{}: {}", lpm_translate_owned("기기에 설치된 롬".to_string()), lpm_normalize_rom_label_for_lang(lang, rest)));
        }
    }

    for prefix in ["image 폴더 롬: ", "image folder ROM: ", "Image folder ROM: "] {
        if let Some(rest) = text.strip_prefix(prefix) {
            return Some(format!("{}: {}", lpm_translate_owned("image 폴더 롬".to_string()), lpm_normalize_rom_label_for_lang(lang, rest)));
        }
    }

    for prefix in ["선택한 국가 코드: ", "Selected country code: ", "선택된 국가 코드: "] {
        if let Some(rest) = text.strip_prefix(prefix) {
            return Some(format!("{}: {}", lpm_translate_owned("선택한 국가 코드".to_string()), lpm_translate_owned(rest.to_string())));
        }
    }

    for prefix in ["감지된 국가 코드: ", "Detected country code: "] {
        if let Some(rest) = text.strip_prefix(prefix) {
            return Some(format!("{}: {rest}", lpm_translate_owned("감지된 국가 코드".to_string())));
        }
    }

    if text.contains("실제 값과 다를 수")
        || text.contains("実際 値")
        || text.contains("實際 值")
        || text.contains("実際の値と異なる")
        || text.contains("可能與實際值不同")
    {
        return Some(lpm_format_widevine_value(lang, text));
    }

    if let Some(rest) = text.strip_prefix("validation warning / ") {
        let rest = lpm_translate_stage13_cleanup(lang, rest.to_string());
        return Some(format!("{} / {rest}", lpm_stage13_term(lang, "validation_warning")));
    }

    if text.contains("재설치 모드는 current slot stage 없이 PreLoader/SPFlashToolV6 단계로 진행합니다") {
        return Some(lpm_translate_exact_stage13(lang, "재설치 모드는 current slot stage 없이 PreLoader/SPFlashToolV6 단계로 진행합니다.").unwrap_or(text).to_string());
    }

    None
}

fn lpm_normalize_rom_label_for_lang(lang: LanguageOption, value: &str) -> String {
    let mut out = value.trim().to_string();
    let pairs = [
        ("ROW(글로벌롬)", "ROW (Global ROM)"),
        ("ROW(글로벌 ROM)", "ROW (Global ROM)"),
        ("ROW (글로벌 ROM)", "ROW (Global ROM)"),
        ("ROW(全球版 ROM)", "ROW (Global ROM)"),
        ("ROW(глобальная ROM)", "ROW (Global ROM)"),
        ("ROW(グローバルROM)", "ROW (Global ROM)"),
        ("ROW (グローバルROM)", "ROW (Global ROM)"),
        ("ROW (全球版 ROM)", "ROW (Global ROM)"),
        ("ROW（グローバルROM）", "ROW (Global ROM)"),
        ("ROW（全球版 ROM）", "ROW (Global ROM)"),
        ("PRC(중국 내수롬)", "PRC (China ROM)"),
        ("PRC(중국 ROM)", "PRC (China ROM)"),
        ("PRC (중국 ROM)", "PRC (China ROM)"),
        ("PRC(中国版ROM)", "PRC (China ROM)"),
        ("PRC(中國版 ROM)", "PRC (China ROM)"),
        ("PRC (中国版ROM)", "PRC (China ROM)"),
        ("PRC (中國版 ROM)", "PRC (China ROM)"),
        ("PRC（中国版ROM）", "PRC (China ROM)"),
        ("PRC（中國版 ROM）", "PRC (China ROM)"),
    ];
    for (from, to) in pairs {
        out = out.replace(from, to);
    }

    match lang {
        LanguageOption::Japanese => out.replace("Global ROM", "グローバルROM").replace("China ROM", "中国版ROM"),
        LanguageOption::TraditionalChinese => out.replace("Global ROM", "全球版 ROM").replace("China ROM", "中國版 ROM"),
        LanguageOption::Vietnamese => out.replace("Global ROM", "ROM toàn cầu").replace("China ROM", "ROM Trung Quốc"),
        LanguageOption::Greek => out.replace("Global ROM", "παγκόσμια ROM").replace("China ROM", "κινεζική ROM"),
        LanguageOption::Hindi => out.replace("Global ROM", "ग्लोबल ROM").replace("China ROM", "चीन ROM"),
        LanguageOption::Russian => out.replace("Global ROM", "глобальная ROM").replace("China ROM", "китайская ROM"),
        LanguageOption::Spanish => out.replace("Global ROM", "ROM global").replace("China ROM", "ROM china"),
        _ => out,
    }
}

fn lpm_widevine_note(lang: LanguageOption) -> &'static str {
    match lang {
        LanguageOption::English => "may differ from the actual value",
        LanguageOption::Russian => "может отличаться от фактического значения",
        LanguageOption::Japanese => "実際の値と異なる場合があります",
        LanguageOption::TraditionalChinese => "可能與實際值不同",
        LanguageOption::Vietnamese => "có thể khác giá trị thực tế",
        LanguageOption::Greek => "μπορεί να διαφέρει από την πραγματική τιμή",
        LanguageOption::Hindi => "वास्तविक मान से भिन्न हो सकता है",
        LanguageOption::Georgian => "შეიძლება განსხვავდებოდეს რეალური მნიშვნელობისგან",
        LanguageOption::Dutch => "kan afwijken van de werkelijke waarde",
        LanguageOption::Arabic => "قد يختلف عن القيمة الفعلية",
        LanguageOption::Spanish => "puede diferir del valor real",
        LanguageOption::Korean => "실제 값과 다를 수 있음",
    }
}

fn lpm_format_widevine_value(lang: LanguageOption, value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "알 수 없음" || trimmed == "감지 전" {
        return trimmed.to_string();
    }

    let base = trimmed
        .split('(')
        .next()
        .unwrap_or(trimmed)
        .split('（')
        .next()
        .unwrap_or(trimmed)
        .trim();

    match lang {
        LanguageOption::Japanese | LanguageOption::TraditionalChinese => {
            format!("{}（{}）", base, lpm_widevine_note(lang))
        }
        _ => format!("{} ({})", base, lpm_widevine_note(lang)),
    }
}

fn lpm_option_rom_line(lang: LanguageOption, image_folder: bool, value: &str) -> String {
    let label = match (lang, image_folder) {
        (LanguageOption::English, true) => "Image folder ROM",
        (LanguageOption::Russian, true) => "ROM папки image",
        (LanguageOption::Japanese, true) => "imageフォルダーROM",
        (LanguageOption::TraditionalChinese, true) => "image 資料夾 ROM",
        (LanguageOption::Vietnamese, true) => "ROM thư mục image",
        (LanguageOption::Greek, true) => "ROM φακέλου image",
        (LanguageOption::Hindi, true) => "image फ़ोल्डर ROM",
        (LanguageOption::Georgian, true) => "image საქაღალდის ROM",
        (LanguageOption::Dutch, true) => "ROM van image-map",
        (LanguageOption::Arabic, true) => "ROM مجلد image",
        (LanguageOption::Spanish, true) => "ROM de carpeta image",
        (LanguageOption::Korean, true) => "image 폴더 롬",
        (LanguageOption::English, false) => "Installed ROM",
        (LanguageOption::Russian, false) => "Установленная ROM",
        (LanguageOption::Japanese, false) => "インストール済みROM",
        (LanguageOption::TraditionalChinese, false) => "已安裝 ROM",
        (LanguageOption::Vietnamese, false) => "ROM đã cài",
        (LanguageOption::Greek, false) => "Εγκατεστημένη ROM",
        (LanguageOption::Hindi, false) => "इंस्टॉल ROM",
        (LanguageOption::Georgian, false) => "დაყენებული ROM",
        (LanguageOption::Dutch, false) => "Geïnstalleerde ROM",
        (LanguageOption::Arabic, false) => "الروم المثبت",
        (LanguageOption::Spanish, false) => "ROM instalada",
        (LanguageOption::Korean, false) => "기기에 설치된 롬",
    };

    let rom = if value.trim().is_empty() || value.trim() == "감지 전" || value.trim() == "알 수 없음" {
        lpm_translate_owned("알 수 없음".to_string())
    } else {
        lpm_normalize_rom_label_for_lang(lang, value)
    };

    format!("{label}: {rom}")
}


fn lpm_rom_option_text(lang: LanguageOption, key: &str) -> &'static str {
    match key {
        "data_wipe_title" => match lang {
            LanguageOption::English => "Factory reset",
            LanguageOption::Korean => "데이터 초기화",
            LanguageOption::Russian => "Сброс данных",
            LanguageOption::Japanese => "データ初期化",
            LanguageOption::TraditionalChinese => "恢復原廠設定",
            LanguageOption::Vietnamese => "Xóa dữ liệu",
            LanguageOption::Greek => "Επαναφορά δεδομένων",
            LanguageOption::Hindi => "डेटा रीसेट",
            LanguageOption::Georgian => "მონაცემების წაშლა",
            LanguageOption::Dutch => "Fabrieksreset",
            LanguageOption::Arabic => "إعادة ضبط البيانات",
            LanguageOption::Spanish => "Restablecer datos",
        },
        "wipe_required" => match lang {
            LanguageOption::English => "A factory reset is required for stability.",
            LanguageOption::Korean => "안정성을 위해 데이터 초기화를 필수적으로 해야합니다.",
            LanguageOption::Russian => "Для стабильности требуется сброс данных.",
            LanguageOption::Japanese => "安定性のため、データ初期化が必要です。",
            LanguageOption::TraditionalChinese => "為了穩定性，必須清除資料。",
            LanguageOption::Vietnamese => "Cần xóa dữ liệu để đảm bảo ổn định.",
            LanguageOption::Greek => "Για σταθερότητα απαιτείται επαναφορά δεδομένων.",
            LanguageOption::Hindi => "स्थिरता के लिए डेटा रीसेट आवश्यक है।",
            LanguageOption::Georgian => "სტაბილურობისთვის საჭიროა მონაცემების წაშლა.",
            LanguageOption::Dutch => "Voor stabiliteit is een fabrieksreset vereist.",
            LanguageOption::Arabic => "يلزم مسح البيانات لضمان الاستقرار.",
            LanguageOption::Spanish => "Es obligatorio restablecer los datos para garantizar la estabilidad.",
        },
        "wipe_if_enabled" => match lang {
            LanguageOption::English => "If enabled, the device will be wiped.",
            LanguageOption::Korean => "활성화 할 경우 기기를 초기화 합니다.",
            LanguageOption::Russian => "Если включено, устройство будет сброшено.",
            LanguageOption::Japanese => "有効にすると端末を初期化します。",
            LanguageOption::TraditionalChinese => "啟用時會清除裝置資料。",
            LanguageOption::Vietnamese => "Nếu bật, thiết bị sẽ bị xóa dữ liệu.",
            LanguageOption::Greek => "Αν ενεργοποιηθεί, η συσκευή θα διαγραφεί.",
            LanguageOption::Hindi => "चालू करने पर डिवाइस मिटा दिया जाएगा।",
            LanguageOption::Georgian => "ჩართვისას მოწყობილობა წაიშლება.",
            LanguageOption::Dutch => "Als dit is ingeschakeld, wordt het apparaat gewist.",
            LanguageOption::Arabic => "عند التفعيل، سيتم مسح الجهاز.",
            LanguageOption::Spanish => "Si se activa, se borrará el dispositivo.",
        },
        "prc_install_wipe_required" => match lang {
            LanguageOption::English => "Installing PRC (China ROM) requires a factory reset.",
            LanguageOption::Korean => "PRC(중국 내수롬) 설치는 데이터 초기화가 필수입니다.",
            LanguageOption::Russian => "Установка PRC (китайская ROM) требует сброса данных.",
            LanguageOption::Japanese => "PRC（中国版ROM）のインストールにはデータ初期化が必要です。",
            LanguageOption::TraditionalChinese => "安裝 PRC（中國版 ROM）時必須清除資料。",
            LanguageOption::Vietnamese => "Cài PRC (ROM Trung Quốc) bắt buộc phải xóa dữ liệu.",
            LanguageOption::Greek => "Η εγκατάσταση PRC (κινεζική ROM) απαιτεί επαναφορά δεδομένων.",
            LanguageOption::Hindi => "PRC (चीन ROM) इंस्टॉल करने के लिए डेटा रीसेट आवश्यक है।",
            LanguageOption::Georgian => "PRC (ჩინური ROM)-ის ინსტალაციისთვის საჭიროა მონაცემების წაშლა.",
            LanguageOption::Dutch => "Voor installatie van PRC (China-ROM) is een fabrieksreset vereist.",
            LanguageOption::Arabic => "يتطلب تثبيت PRC (روم الصين) مسح البيانات.",
            LanguageOption::Spanish => "Instalar PRC (ROM china) requiere restablecer los datos.",
        },
        "country_title" => match lang {
            LanguageOption::English => "Country Code",
            LanguageOption::Korean => "국가 코드",
            LanguageOption::Russian => "Код страны",
            LanguageOption::Japanese => "国コード",
            LanguageOption::TraditionalChinese => "國家代碼",
            LanguageOption::Vietnamese => "Mã quốc gia",
            LanguageOption::Greek => "Κωδικός χώρας",
            LanguageOption::Hindi => "देश कोड",
            LanguageOption::Georgian => "ქვეყნის კოდი",
            LanguageOption::Dutch => "Landcode",
            LanguageOption::Arabic => "رمز البلد",
            LanguageOption::Spanish => "Código de país",
        },
        "country_locked" => match lang {
            LanguageOption::English => "Country code cannot be changed for PRC (China ROM).",
            LanguageOption::Korean => "PRC(중국 내수롬)은 국가 코드를 변경할 수 없습니다.",
            LanguageOption::Russian => "Для PRC (китайская ROM) код страны изменить нельзя.",
            LanguageOption::Japanese => "PRC（中国版ROM）では国コードを変更できません。",
            LanguageOption::TraditionalChinese => "PRC（中國版 ROM）無法變更國家代碼。",
            LanguageOption::Vietnamese => "Không thể đổi mã quốc gia cho PRC (ROM Trung Quốc).",
            LanguageOption::Greek => "Για PRC (κινεζική ROM) δεν μπορεί να αλλάξει ο κωδικός χώρας.",
            LanguageOption::Hindi => "PRC (चीन ROM) में देश कोड बदला नहीं जा सकता।",
            LanguageOption::Georgian => "PRC (ჩინური ROM)-ზე ქვეყნის კოდის შეცვლა შეუძლებელია.",
            LanguageOption::Dutch => "De landcode kan niet worden gewijzigd voor PRC (China-ROM).",
            LanguageOption::Arabic => "لا يمكن تغيير رمز البلد في PRC (روم الصين).",
            LanguageOption::Spanish => "No se puede cambiar el código de país en PRC (ROM china).",
        },
        "country_change" => match lang {
            LanguageOption::English => "Change the country code on the device.",
            LanguageOption::Korean => "기기에 국가 코드를 변경합니다.",
            LanguageOption::Russian => "Изменить код страны на устройстве.",
            LanguageOption::Japanese => "端末の国コードを変更します。",
            LanguageOption::TraditionalChinese => "變更裝置上的國家代碼。",
            LanguageOption::Vietnamese => "Đổi mã quốc gia trên thiết bị.",
            LanguageOption::Greek => "Αλλαγή του κωδικού χώρας στη συσκευή.",
            LanguageOption::Hindi => "डिवाइस पर देश कोड बदलें।",
            LanguageOption::Georgian => "შეცვალეთ ქვეყნის კოდი მოწყობილობაზე.",
            LanguageOption::Dutch => "Wijzig de landcode op het apparaat.",
            LanguageOption::Arabic => "تغيير رمز البلد على الجهاز.",
            LanguageOption::Spanish => "Cambiar el código de país del dispositivo.",
        },
        "detected_country" => match lang {
            LanguageOption::English => "Detected country code",
            LanguageOption::Korean => "감지된 국가 코드",
            LanguageOption::Russian => "Обнаруженный код страны",
            LanguageOption::Japanese => "検出された国コード",
            LanguageOption::TraditionalChinese => "偵測到的國家代碼",
            LanguageOption::Vietnamese => "Mã quốc gia đã phát hiện",
            LanguageOption::Greek => "Εντοπισμένος κωδικός χώρας",
            LanguageOption::Hindi => "पहचाना गया देश कोड",
            LanguageOption::Georgian => "აღმოჩენილი ქვეყნის კოდი",
            LanguageOption::Dutch => "Gedetecteerde landcode",
            LanguageOption::Arabic => "رمز البلد المكتشف",
            LanguageOption::Spanish => "Código de país detectado",
        },
        "selected_country" => match lang {
            LanguageOption::English => "Selected country code",
            LanguageOption::Korean => "선택한 국가 코드",
            LanguageOption::Russian => "Выбранный код страны",
            LanguageOption::Japanese => "選択した国コード",
            LanguageOption::TraditionalChinese => "已選擇的國家代碼",
            LanguageOption::Vietnamese => "Mã quốc gia đã chọn",
            LanguageOption::Greek => "Επιλεγμένος κωδικός χώρας",
            LanguageOption::Hindi => "चयनित देश कोड",
            LanguageOption::Georgian => "არჩეული ქვეყნის კოდი",
            LanguageOption::Dutch => "Geselecteerde landcode",
            LanguageOption::Arabic => "رمز البلد المحدد",
            LanguageOption::Spanish => "Código de país seleccionado",
        },
        "unavailable" => match lang {
            LanguageOption::English => "Unavailable",
            LanguageOption::Korean => "선택 불가",
            LanguageOption::Russian => "Недоступно",
            LanguageOption::Japanese => "選択不可",
            LanguageOption::TraditionalChinese => "無法選擇",
            LanguageOption::Vietnamese => "Không khả dụng",
            LanguageOption::Greek => "Μη διαθέσιμο",
            LanguageOption::Hindi => "उपलब्ध नहीं",
            LanguageOption::Georgian => "მიუწვდომელია",
            LanguageOption::Dutch => "Niet beschikbaar",
            LanguageOption::Arabic => "غير متاح",
            LanguageOption::Spanish => "No disponible",
        },
        "not_selected" => match lang {
            LanguageOption::English => "Not selected",
            LanguageOption::Korean => "선택 안 됨",
            LanguageOption::Russian => "Не выбрано",
            LanguageOption::Japanese => "未選択",
            LanguageOption::TraditionalChinese => "未選擇",
            LanguageOption::Vietnamese => "Chưa chọn",
            LanguageOption::Greek => "Δεν επιλέχθηκε",
            LanguageOption::Hindi => "चयनित नहीं",
            LanguageOption::Georgian => "არ არის არჩეული",
            LanguageOption::Dutch => "Niet geselecteerd",
            LanguageOption::Arabic => "غير محدد",
            LanguageOption::Spanish => "No seleccionado",
        },
        "unknown" => match lang {
            LanguageOption::English => "Unknown",
            LanguageOption::Korean => "알 수 없음",
            LanguageOption::Russian => "Неизвестно",
            LanguageOption::Japanese => "不明",
            LanguageOption::TraditionalChinese => "未知",
            LanguageOption::Vietnamese => "Không xác định",
            LanguageOption::Greek => "Άγνωστο",
            LanguageOption::Hindi => "अज्ञात",
            LanguageOption::Georgian => "უცნობია",
            LanguageOption::Dutch => "Onbekend",
            LanguageOption::Arabic => "غير معروف",
            LanguageOption::Spanish => "Desconocido",
        },
        "select" => match lang {
            LanguageOption::English => "Select",
            LanguageOption::Korean => "선택",
            LanguageOption::Russian => "Выбрать",
            LanguageOption::Japanese => "選択",
            LanguageOption::TraditionalChinese => "選擇",
            LanguageOption::Vietnamese => "Chọn",
            LanguageOption::Greek => "Επιλογή",
            LanguageOption::Hindi => "चुनें",
            LanguageOption::Georgian => "არჩევა",
            LanguageOption::Dutch => "Selecteren",
            LanguageOption::Arabic => "اختيار",
            LanguageOption::Spanish => "Seleccionar",
        },
        "back" => match lang {
            LanguageOption::English => "Back",
            LanguageOption::Korean => "이전 메뉴로 이동",
            LanguageOption::Russian => "Назад",
            LanguageOption::Japanese => "前のメニューへ",
            LanguageOption::TraditionalChinese => "返回",
            LanguageOption::Vietnamese => "Quay lại",
            LanguageOption::Greek => "Πίσω",
            LanguageOption::Hindi => "वापस",
            LanguageOption::Georgian => "უკან",
            LanguageOption::Dutch => "Terug",
            LanguageOption::Arabic => "رجوع",
            LanguageOption::Spanish => "Atrás",
        },
        "continue" => match lang {
            LanguageOption::English => "Continue",
            LanguageOption::Korean => "계속",
            LanguageOption::Russian => "Продолжить",
            LanguageOption::Japanese => "続行",
            LanguageOption::TraditionalChinese => "繼續",
            LanguageOption::Vietnamese => "Tiếp tục",
            LanguageOption::Greek => "Συνέχεια",
            LanguageOption::Hindi => "जारी रखें",
            LanguageOption::Georgian => "გაგრძელება",
            LanguageOption::Dutch => "Doorgaan",
            LanguageOption::Arabic => "متابعة",
            LanguageOption::Spanish => "Continuar",
        },
        "before_start" => match lang {
            LanguageOption::English => "Before you start",
            LanguageOption::Korean => "시작 전 확인",
            LanguageOption::Russian => "Перед началом",
            LanguageOption::Japanese => "開始前の確認",
            LanguageOption::TraditionalChinese => "開始前確認",
            LanguageOption::Vietnamese => "Kiểm tra trước khi bắt đầu",
            LanguageOption::Greek => "Πριν ξεκινήσετε",
            LanguageOption::Hindi => "शुरू करने से पहले",
            LanguageOption::Georgian => "დაწყებამდე",
            LanguageOption::Dutch => "Vóór starten",
            LanguageOption::Arabic => "قبل البدء",
            LanguageOption::Spanish => "Antes de empezar",
        },
        _ => "",
    }
}


fn lpm_stage13_term(lang: LanguageOption, key: &str) -> &'static str {
    match key {
        "validation_warning" => lpm_lang_text(lang, "validation warning", "предупреждение проверки", "検証警告", "驗證警告", "cảnh báo xác thực", "προειδοποίηση ελέγχου", "मान्यकरण चेतावनी", "ვალიდაციის გაფრთხილება", "validatiewaarschuwing", "تحذير التحقق", "advertencia de validación"),
        _ => "",
    }
}

fn lpm_translate_stage13_cleanup(lang: LanguageOption, content: String) -> String {
    if lang.is_korean() {
        return content;
    }

    let mut out = content;
    let universal = [
        ("재설치 모드는 current slot stage 없이 PreLoader/SPFlashToolV6 단계로 진행합니다.", lpm_translate_exact_stage13(lang, "재설치 모드는 current slot stage 없이 PreLoader/SPFlashToolV6 단계로 진행합니다.").unwrap_or("")),
        ("재설치 모드는 current slot stage 없이 PreLoader/SPFlashToolV6 단계로 진행합니다", lpm_translate_exact_stage13(lang, "재설치 모드는 current slot stage 없이 PreLoader/SPFlashToolV6 단계로 진행합니다.").unwrap_or("")),
        ("모드는 current slot stage 없이 PreLoader/SPFlashToolV6 단계로 진행합니다.", lpm_translate_exact_stage13(lang, "재설치 모드는 current slot stage 없이 PreLoader/SPFlashToolV6 단계로 진행합니다.").unwrap_or("")),
        ("current slot stage 없이 PreLoader/SPFlashToolV6 단계로 진행합니다.", lpm_translate_exact_stage13(lang, "재설치 모드는 current slot stage 없이 PreLoader/SPFlashToolV6 단계로 진행합니다.").unwrap_or("")),
        ("ROW(글로벌롬)", "ROW (Global ROM)"),
        ("ROW(글로벌 ROM)", "ROW (Global ROM)"),
        ("PRC(중국 내수롬)", "PRC (China ROM)"),
    ];
    for (from, to) in universal {
        if !to.is_empty() {
            out = out.replace(from, to);
        }
    }

    for (from, to) in lpm_stage13_cleanup_pairs(lang) {
        out = out.replace(from, to);
    }

    out
}

fn lpm_stage13_cleanup_pairs(lang: LanguageOption) -> &'static [(&'static str, &'static str)] {
    use LanguageOption::*;
    match lang {
        English => &[
            ("모델명", "Model"), ("펌웨어 유형", "Firmware type"), ("배터리 잔량", "Battery level"), ("가동 시간", "Uptime"), ("연결한 기기", "Connected device"),
            ("ROW (Global ROM) 버전 업데이트.", "Update the ROW ROM version."), ("ROW (Global ROM) 버전 업데이트", "Update the ROW ROM version"),
            ("Country Code를 changes.", "Change the country code on the device."), ("Country Code를", "Country Code"), ("installed 롬", "installed ROM"), ("설치된 롬", "installed ROM"),
            ("select 안 됨", "Not selected"), ("선택 안 됨", "Not selected"), ("데이터 초기화", "Factory reset"),
        ],
        Russian => &[
            ("모델명", "Модель"), ("펌웨어 유형", "Тип прошивки"), ("배터리 잔량", "Заряд батареи"), ("가동 시간", "Время работы"), ("연결한", "Подключённое"),
            ("버전 업데이트.", "обновление версии."), ("설치된 롬", "установленная ROM"), ("선택 안 됨", "Не выбрано"), ("데이터 초기화", "Сброс данных"),
        ],
        Japanese => &[
            ("모델명", "モデル名"), ("펌웨어 유형", "ファームウェア種類"), ("배터리 잔량", "バッテリー残量"), ("가동 시간", "稼働時間"), ("연결한", "接続した"),
            ("버전 업데이트.", "バージョンを更新します。"), ("설치된 롬", "インストール済みROM"), ("선택 안 됨", "未選択"), ("데이터 초기화", "データ初期化"),
            ("를", "を"), ("을", "を"), ("에", "に"),
        ],
        TraditionalChinese => &[
            ("모델명", "型號"), ("펌웨어 유형", "韌體類型"), ("배터리 잔량", "電池電量"), ("가동 시간", "運作時間"), ("연결한", "已連接"),
            ("버전 업데이트.", "版本更新。"), ("설치된 롬", "已安裝 ROM"), ("선택 안 됨", "未選擇"), ("데이터 초기화", "清除資料"),
            ("사용 가능", "可使用"), ("적용됨", "已套用"), ("성공", "成功"), ("통과", "通過"), ("경고", "警告"), ("개", "個"),
        ],
        Vietnamese => &[
            ("모델명", "Tên mẫu"), ("펌웨어 유형", "Loại firmware"), ("배터리 잔량", "Pin còn lại"), ("가동 시간", "Thời gian hoạt động"), ("연결한", "Đã kết nối"),
            ("버전 업데이트.", "cập nhật phiên bản."), ("설치된 롬", "ROM đã cài"), ("선택 안 됨", "Chưa chọn"), ("데이터 초기화", "Xóa dữ liệu"),
        ],
        Greek => &[
            ("모델명", "Μοντέλο"), ("펌웨어 유형", "Τύπος firmware"), ("배터리 잔량", "Υπόλοιπο μπαταρίας"), ("가동 시간", "Χρόνος λειτουργίας"), ("연결한", "Συνδεδεμένη"),
            ("버전 업데이트.", "ενημέρωση έκδοσης."), ("설치된 롬", "εγκατεστημένη ROM"), ("선택 안 됨", "Δεν επιλέχθηκε"), ("데이터 초기화", "Επαναφορά δεδομένων"),
        ],
        Hindi => &[
            ("모델명", "मॉडल"), ("펌웨어 유형", "फर्मवेयर प्रकार"), ("배터리 잔량", "बैटरी स्तर"), ("가동 시간", "अपटाइम"), ("연결한", "कनेक्टेड"),
            ("버전 업데이트.", "संस्करण अपडेट।"), ("설치된 롬", "इंस्टॉल ROM"), ("선택 안 됨", "चयनित नहीं"), ("데이터 초기화", "डेटा रीसेट"),
            ("성공", "सफल"), ("통과", "उत्तीर्ण"), ("경고", "चेतावनी"), ("개", " आइटम"), ("있음", "मौजूद"), ("사용 가능", "उपलब्ध"), ("적용됨", "लागू"),
        ],
        Georgian => &[("모델명", "მოდელი"), ("펌웨어 유형", "Firmware ტიპი"), ("배터리 잔량", "ბატარეის დონე"), ("가동 시간", "მუშაობის დრო"), ("선택 안 됨", "არ არის არჩეული")],
        Dutch => &[("모델명", "Model"), ("펌웨어 유형", "Firmwaretype"), ("배터리 잔량", "Batterijniveau"), ("가동 시간", "Uptime"), ("선택 안 됨", "Niet geselecteerd")],
        Arabic => &[("모델명", "الطراز"), ("펌웨어 유형", "نوع firmware"), ("배터리 잔량", "مستوى البطارية"), ("가동 시간", "مدة التشغيل"), ("선택 안 됨", "غير محدد")],
        Spanish => &[("모델명", "Modelo"), ("펌웨어 유형", "Tipo de firmware"), ("배터리 잔량", "Nivel de batería"), ("가동 시간", "Tiempo activo"), ("선택 안 됨", "No seleccionado")],
        Korean => &[],
    }
}

fn lpm_translate_duration_text(lang: LanguageOption, content: &str) -> Option<String> {
    if lang.is_korean() {
        return None;
    }

    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }

    if !trimmed.chars().any(|c| matches!(c, '일' | '시' | '간' | '분' | '초')) {
        return None;
    }

    if !trimmed.chars().all(|c| c.is_ascii_digit() || c.is_ascii_whitespace() || matches!(c, '일' | '시' | '간' | '분' | '초')) {
        return None;
    }

    let mut out = trimmed.to_string();
    match lang {
        LanguageOption::English => {
            out = out.replace("시간", "h").replace("일", "d").replace("분", "m").replace("초", "s");
        }
        LanguageOption::Russian => {
            out = out.replace("시간", " ч").replace("일", " дн.").replace("분", " мин").replace("초", " с");
        }
        LanguageOption::Japanese => {
            out = out.replace("시간", "時間").replace("일", "日").replace("분", "分").replace("초", "秒");
        }
        LanguageOption::TraditionalChinese => {
            out = out.replace("시간", "小時").replace("일", "天").replace("분", "分鐘").replace("초", "秒");
        }
        LanguageOption::Vietnamese => {
            out = out.replace("시간", " giờ").replace("일", " ngày").replace("분", " phút").replace("초", " giây");
        }
        LanguageOption::Greek => {
            out = out.replace("시간", " ώρες").replace("일", " ημ.").replace("분", " λεπτά").replace("초", " δευτ.");
        }
        LanguageOption::Hindi => {
            out = out.replace("시간", " घंटे").replace("일", " दिन").replace("분", " मिनट").replace("초", " सेकंड");
        }
        LanguageOption::Georgian => {
            out = out.replace("시간", " სთ").replace("일", " დღე").replace("분", " წთ").replace("초", " წმ");
        }
        LanguageOption::Dutch => {
            out = out.replace("시간", " u").replace("일", " d").replace("분", " min").replace("초", " sec");
        }
        LanguageOption::Arabic => {
            out = out.replace("시간", " ساعة").replace("일", " يوم").replace("분", " دقيقة").replace("초", " ثانية");
        }
        LanguageOption::Spanish => {
            out = out.replace("시간", " h").replace("일", " d").replace("분", " min").replace("초", " s");
        }
        LanguageOption::Korean => {}
    }

    Some(out)
}

fn lpm_stage11_label(lang: LanguageOption, key: &str) -> &'static str {
    match key {
        "selected_image_folder" => lpm_lang_text(lang, "Selected image folder", "Выбранная папка image", "選択したimageフォルダー", "已選擇的 image 資料夾", "Thư mục image đã chọn", "Επιλεγμένος φάκελος image", "चयनित image फ़ोल्डर", "არჩეული image საქაღალდე", "Geselecteerde image-map", "مجلد image المحدد", "Carpeta image seleccionada"),
        "firmware_check_start" => lpm_lang_text(lang, "Starting firmware information and installation environment check.", "Запуск проверки информации о прошивке и среды установки.", "ファームウェア情報とインストール環境の確認を開始します。", "開始檢查韌體資訊與安裝環境。", "Bắt đầu kiểm tra thông tin firmware và môi trường cài đặt.", "Έναρξη ελέγχου πληροφοριών firmware και περιβάλλοντος εγκατάστασης.", "फर्मवेयर जानकारी और इंस्टॉलेशन वातावरण की जाँच शुरू।", "Firmware-ის ინფორმაციისა და ინსტალაციის გარემოს შემოწმება იწყება.", "Controle van firmware-informatie en installatieomgeving wordt gestart.", "بدء فحص معلومات firmware وبيئة التثبيت.", "Iniciando comprobación de información del firmware y entorno de instalación."),
        "firmware_check_complete" => lpm_lang_text(lang, "Firmware check completed", "Проверка прошивки завершена", "ファームウェア確認完了", "韌體檢查完成", "Kiểm tra firmware hoàn tất", "Ο έλεγχος firmware ολοκληρώθηκε", "फर्मवेयर जाँच पूर्ण", "Firmware-ის შემოწმება დასრულდა", "Firmwarecontrole voltooid", "اكتمل فحص firmware", "Comprobación de firmware completada"),
        "model" => lpm_lang_text(lang, "Model", "Модель", "モデル", "型號", "Model", "Μοντέλο", "मॉडल", "მოდელი", "Model", "الطراز", "Modelo"),
        "version" => lpm_lang_text(lang, "Version", "Версия", "バージョン", "版本", "Phiên bản", "Έκδοση", "संस्करण", "ვერსია", "Versie", "الإصدار", "Versión"),
        "rom_type" => lpm_lang_text(lang, "ROM type", "Тип ROM", "ROM種類", "ROM 類型", "Loại ROM", "Τύπος ROM", "ROM प्रकार", "ROM ტიპი", "ROM-type", "نوع ROM", "Tipo de ROM"),
        "platform" => lpm_lang_text(lang, "Platform", "Платформа", "プラットフォーム", "平台", "Nền tảng", "Πλατφόρμα", "प्लेटफ़ॉर्म", "პლატფორმა", "Platform", "المنصة", "Plataforma"),
        "success" => lpm_lang_text(lang, "success", "успешно", "成功", "成功", "thành công", "επιτυχία", "सफल", "წარმატება", "succes", "نجاح", "correcto"),
        "failed" => lpm_lang_text(lang, "failed", "ошибка", "失敗", "失敗", "thất bại", "αποτυχία", "विफल", "ვერ შესრულდა", "mislukt", "فشل", "fallido"),
        "available" => lpm_lang_text(lang, "available", "доступно", "使用可能", "可使用", "khả dụng", "διαθέσιμο", "उपलब्ध", "ხელმისაწვდომია", "beschikbaar", "متاح", "disponible"),
        "applied" => lpm_lang_text(lang, "applied", "применено", "適用済み", "已套用", "đã áp dụng", "εφαρμόστηκε", "लागू", "გამოყენებულია", "toegepast", "تم التطبيق", "aplicado"),
        "passed" => lpm_lang_text(lang, "passed", "пройдено", "合格", "通過", "đạt", "πέρασε", "उत्तीर्ण", "გაიარა", "geslaagd", "تم الاجتياز", "superado"),
        "warning" => lpm_lang_text(lang, "warning", "предупреждение", "警告", "警告", "cảnh báo", "προειδοποίηση", "चेतावनी", "გაფრთხილება", "waarschuwing", "تحذير", "advertencia"),
        "text_file_saved" => lpm_lang_text(lang, "Saved text file", "Текстовый файл сохранён", "テキストファイルを保存しました", "已儲存文字檔", "Đã lưu tệp văn bản", "Το αρχείο κειμένου αποθηκεύτηκε", "टेक्स्ट फ़ाइल सहेजी गई", "ტექსტური ფაილი შენახულია", "Tekstbestand opgeslagen", "تم حفظ الملف النصي", "Archivo de texto guardado"),
        "selected_program_language" => lpm_lang_text(lang, "Selected program language", "Выбран язык программы", "プログラム言語を選択しました", "已選擇程式語言", "Đã chọn ngôn ngữ chương trình", "Επιλέχθηκε γλώσσα προγράμματος", "प्रोग्राम भाषा चुनी गई", "პროგრამის ენა არჩეულია", "Programmataal geselecteerd", "تم اختيار لغة البرنامج", "Idioma del programa seleccionado"),
        "language_file_saved" => lpm_lang_text(lang, "Saved to language setting file", "Сохранено в файл настройки языка", "言語設定ファイルに保存しました", "已儲存到語言設定檔", "Đã lưu vào tệp cài đặt ngôn ngữ", "Αποθηκεύτηκε στο αρχείο ρύθμισης γλώσσας", "भाषा सेटिंग फ़ाइल में सहेजा गया", "შენახულია ენის პარამეტრის ფაილში", "Opgeslagen in taalinstellingenbestand", "تم الحفظ في ملف إعداد اللغة", "Guardado en el archivo de idioma"),
        "not_selected" => lpm_lang_text(lang, "Not selected", "Не выбрано", "未選択", "未選擇", "Chưa chọn", "Δεν επιλέχθηκε", "चयनित नहीं", "არ არის არჩეული", "Niet geselecteerd", "غير محدد", "No seleccionado"),
        "scatter_xml" => lpm_lang_text(lang, "Scatter XML", "Scatter XML", "Scatter XML", "Scatter XML", "Scatter XML", "Scatter XML", "Scatter XML", "Scatter XML", "Scatter XML", "Scatter XML", "Scatter XML"),
        "partition_list" => lpm_lang_text(lang, "Partition list", "Список разделов", "パーティション一覧", "分割區清單", "Danh sách phân vùng", "Λίστα διαμερισμάτων", "पार्टिशन सूची", "დანაყოფების სია", "Partitielijst", "قائمة الأقسام", "Lista de particiones"),
        "partition_details" => lpm_lang_text(lang, "Partition details", "Сведения о разделах", "パーティション詳細", "分割區詳細資訊", "Chi tiết phân vùng", "Λεπτομέρειες διαμερισμάτων", "पार्टिशन विवरण", "დანაყოფების დეტალები", "Partitiedetails", "تفاصيل الأقسام", "Detalles de particiones"),
        "required_partition_check" => lpm_lang_text(lang, "Required partition check", "Проверка обязательных разделов", "必須パーティション確認", "必要分割區檢查", "Kiểm tra phân vùng bắt buộc", "Έλεγχος απαιτούμενων διαμερισμάτων", "आवश्यक पार्टिशन जाँच", "აუცილებელი დანაყოფების შემოწმება", "Controle van vereiste partities", "فحص الأقسام المطلوبة", "Comprobación de particiones obligatorias"),
        "required_partition_details" => lpm_lang_text(lang, "Required partition details", "Сведения об обязательных разделах", "必須パーティション詳細", "必要分割區詳細資訊", "Chi tiết phân vùng bắt buộc", "Λεπτομέρειες απαιτούμενων διαμερισμάτων", "आवश्यक पार्टिशन विवरण", "აუცილებელი დანაყოფების დეტალები", "Details van vereiste partities", "تفاصيل الأقسام المطلوبة", "Detalles de particiones obligatorias"),
        "patch_plan_creation" => lpm_lang_text(lang, "Patch plan creation", "Создание плана исправлений", "パッチプラン作成", "建立修補計畫", "Tạo kế hoạch vá", "Δημιουργία σχεδίου ενημέρωσης", "पैच योजना बनाना", "ცვლილებების გეგმის შექმნა", "Patchplan maken", "إنشاء خطة التعديل", "Creación del plan de parches"),
        "patch_plan_preview" => lpm_lang_text(lang, "Patch plan preview", "Предварительный просмотр плана исправлений", "パッチプランのプレビュー", "修補計畫預覽", "Xem trước kế hoạch vá", "Προεπισκόπηση σχεδίου ενημέρωσης", "पैच योजना पूर्वावलोकन", "ცვლილებების გეგმის წინასწარი ნახვა", "Voorbeeld van patchplan", "معاينة خطة التعديل", "Vista previa del plan de parches"),
        "patch_plan_application" => lpm_lang_text(lang, "Patch plan application", "Применение плана исправлений", "パッチプラン適用", "套用修補計畫", "Áp dụng kế hoạch vá", "Εφαρμογή σχεδίου ενημέρωσης", "पैच योजना लागू करना", "ცვლილებების გეგმის გამოყენება", "Patchplan toepassen", "تطبيق خطة التعديل", "Aplicación del plan de parches"),
        "patch_application_preview" => lpm_lang_text(lang, "Patch application preview", "Предварительный просмотр применения исправлений", "パッチ適用プレビュー", "修補套用預覽", "Xem trước việc áp dụng bản vá", "Προεπισκόπηση εφαρμογής ενημέρωσης", "पैच लागू करने का पूर्वावलोकन", "ცვლილებების გამოყენების წინასწარი ნახვა", "Voorbeeld van patchtoepassing", "معاينة تطبيق التعديل", "Vista previa de aplicación del parche"),
        "patch_result_recheck" => lpm_lang_text(lang, "Patch result verification", "Повторная проверка результатов исправлений", "パッチ結果の再確認", "重新檢查修補結果", "Kiểm tra lại kết quả vá", "Επανέλεγχος αποτελεσμάτων ενημέρωσης", "पैच परिणाम की दोबारा जाँच", "ცვლილებების შედეგის ხელახალი შემოწმება", "Patchresultaat opnieuw controleren", "إعادة التحقق من نتيجة التعديل", "Nueva comprobación del resultado del parche"),
        "patch_result_preview" => lpm_lang_text(lang, "Patch verification preview", "Сводка повторной проверки исправлений", "パッチ再確認の概要", "修補重新檢查摘要", "Tóm tắt kiểm tra lại bản vá", "Σύνοψη επανελέγχου ενημέρωσης", "पैच पुनः जाँच सारांश", "ცვლილებების ხელახალი შემოწმების შეჯამება", "Overzicht van patchcontrole", "ملخص إعادة التحقق من التعديل", "Resumen de la nueva comprobación del parche"),
        "validation_warning" => lpm_lang_text(lang, "Validation warning", "Предупреждение проверки", "検証警告", "驗證警告", "Cảnh báo xác thực", "Προειδοποίηση επικύρωσης", "सत्यापन चेतावनी", "ვალიდაციის გაფრთხილება", "Validatiewaarschuwing", "تحذير التحقق", "Advertencia de validación"),
        "settings_tag" => lpm_lang_text(lang, "Settings", "Настройки", "設定", "設定", "Cài đặt", "Ρυθμίσεις", "सेटिंग्स", "პარამეტრები", "Instellingen", "الإعدادات", "Configuración"),
        "initial_language" => lpm_lang_text(lang, "Initial language", "Начальный язык", "初期言語", "初始語言", "Ngôn ngữ ban đầu", "Αρχική γλώσσα", "प्रारंभिक भाषा", "საწყისი ენა", "Begintaal", "اللغة الأولية", "Idioma inicial"),
        "source" => lpm_lang_text(lang, "Source", "Источник", "基準", "依據", "Nguồn", "Πηγή", "स्रोत", "წყარო", "Bron", "المصدر", "Origen"),
        "language_file_path" => lpm_lang_text(lang, "Language setting file path", "Путь к файлу настройки языка", "言語設定ファイルのパス", "語言設定檔路徑", "Đường dẫn tệp cài đặt ngôn ngữ", "Διαδρομή αρχείου ρύθμισης γλώσσας", "भाषा सेटिंग फ़ाइल पथ", "ენის პარამეტრის ფაილის ბილიკი", "Pad van taalinstellingenbestand", "مسار ملف إعداد اللغة", "Ruta del archivo de idioma"),
        _ => "",
    }
}

fn lpm_localized_language_display_name(lang: LanguageOption, value: &str) -> String {
    match value.trim() {
        "한국어 (ko)" => lpm_lang_text(
            lang,
            "Korean (ko)",
            "Корейский (ko)",
            "韓国語 (ko)",
            "韓文 (ko)",
            "Tiếng Hàn (ko)",
            "Κορεατικά (ko)",
            "कोरियाई (ko)",
            "კორეული (ko)",
            "Koreaans (ko)",
            "الكورية (ko)",
            "Coreano (ko)",
        )
        .to_string(),
        other => other.to_string(),
    }
}

fn lpm_translate_dynamic_stage11(lang: LanguageOption, content: &str) -> Option<String> {
    if lang.is_korean() {
        return None;
    }

    let text = content.trim();

    if let Some(rest) = text.strip_prefix("image 폴더 선택됨: ") {
        return Some(format!("{}: {rest}", lpm_stage11_label(lang, "selected_image_folder")));
    }
    if text == "펌웨어 정보 및 설치 환경 검사를 시작합니다." {
        return Some(lpm_stage11_label(lang, "firmware_check_start").to_string());
    }
    if text == "펌웨어 검사 완료" {
        return Some(lpm_stage11_label(lang, "firmware_check_complete").to_string());
    }
    if let Some(rest) = text.strip_prefix("[Image] 모델명: ") {
        return Some(format!("[Image] {}: {rest}", lpm_stage11_label(lang, "model")));
    }
    if let Some(rest) = text.strip_prefix("[Image] 버전: ") {
        return Some(format!("[Image] {}: {rest}", lpm_stage11_label(lang, "version")));
    }
    if let Some(rest) = text.strip_prefix("[Image] ROM 타입: ") {
        return Some(format!("[Image] {}: {}", lpm_stage11_label(lang, "rom_type"), lpm_translate_owned(rest.to_string())));
    }
    if let Some(rest) = text.strip_prefix("[Image] 플랫폼: ") {
        return Some(format!("[Image] {}: {rest}", lpm_stage11_label(lang, "platform")));
    }
    if let Some(rest) = text.strip_prefix("온라인 차단 펌웨어 규칙 검사: ") {
        let converted = lpm_translate_stage11_final_cleanup(lang, rest.to_string());
        let label = lpm_lang_text(
            lang,
            "Online blocked-firmware rules",
            "Онлайн-правила блокировки прошивок",
            "オンラインのファームウェア遮断ルール",
            "線上韌體封鎖規則",
            "Quy tắc chặn firmware trực tuyến",
            "Διαδικτυακοί κανόνες αποκλεισμού firmware",
            "ऑनलाइन फर्मवेयर अवरोध नियम",
            "Firmware-ის ონლაინ დაბლოკვის წესები",
            "Online regels voor geblokkeerde firmware",
            "قواعد حظر البرامج الثابتة عبر الإنترنت",
            "Reglas en línea de bloqueo de firmware",
        );
        return Some(format!("{label}: {converted}"));
    }
    if let Some(rest) = text.strip_prefix("scatter XML 파싱: ") {
        return Some(format!("{}: {}", lpm_stage11_label(lang, "scatter_xml"), lpm_translate_stage11_final_cleanup(lang, rest.to_string())));
    }
    if let Some(rest) = text.strip_prefix("partition 목록 읽기: ") {
        return Some(format!("{}: {}", lpm_stage11_label(lang, "partition_list"), lpm_translate_stage11_final_cleanup(lang, rest.to_string())));
    }
    if let Some(rest) = text.strip_prefix("partition 상세 정보 읽기: ") {
        return Some(format!("{}: {}", lpm_stage11_label(lang, "partition_details"), lpm_translate_stage11_final_cleanup(lang, rest.to_string())));
    }
    if let Some(rest) = text.strip_prefix("필수 partition 검사: ") {
        return Some(format!("{}: {}", lpm_stage11_label(lang, "required_partition_check"), lpm_translate_stage11_final_cleanup(lang, rest.to_string())));
    }
    if let Some(rest) = text.strip_prefix("필수 partition 상세: ") {
        return Some(format!("{}: {}", lpm_stage11_label(lang, "required_partition_details"), lpm_translate_stage11_final_cleanup(lang, rest.to_string())));
    }
    if let Some(rest) = text.strip_prefix("patch plan 생성: ") {
        return Some(format!("{}: {}", lpm_stage11_label(lang, "patch_plan_creation"), lpm_translate_stage11_final_cleanup(lang, rest.to_string())));
    }
    if let Some(rest) = text.strip_prefix("patch plan 미리보기: ") {
        return Some(format!("{}: {}", lpm_stage11_label(lang, "patch_plan_preview"), lpm_translate_stage11_final_cleanup(lang, rest.to_string())));
    }
    if let Some(rest) = text.strip_prefix("patch plan 적용: ") {
        return Some(format!("{}: {}", lpm_stage11_label(lang, "patch_plan_application"), lpm_translate_stage11_final_cleanup(lang, rest.to_string())));
    }
    if let Some(rest) = text.strip_prefix("patch 적용 미리보기: ") {
        return Some(format!("{}: {}", lpm_stage11_label(lang, "patch_application_preview"), lpm_translate_stage11_final_cleanup(lang, rest.to_string())));
    }
    if let Some(rest) = text.strip_prefix("patch 결과 재검증: ") {
        return Some(format!("{}: {}", lpm_stage11_label(lang, "patch_result_recheck"), lpm_translate_stage11_final_cleanup(lang, rest.to_string())));
    }
    if let Some(rest) = text.strip_prefix("patch 결과 재검증 미리보기: ") {
        return Some(format!("{}: {}", lpm_stage11_label(lang, "patch_result_preview"), lpm_translate_stage11_final_cleanup(lang, rest.to_string())));
    }
    if let Some(rest) = text.strip_prefix("검증 경고 / ") {
        return Some(format!("{} / {}", lpm_stage11_label(lang, "validation_warning"), lpm_translate_stage11_final_cleanup(lang, rest.to_string())));
    }
    if let Some(rest) = text.strip_prefix("[Log] 텍스트 파일을 ") {
        if let Some(path) = rest.strip_suffix("에 저장합니다.") {
            return Some(format!("[Log] {}: {path}", lpm_stage11_label(lang, "text_file_saved")));
        }
    }
    if let Some(rest) = text.strip_prefix("[설정] 초기 언어 설정: ") {
        if let Some((language, source)) = rest.split_once(" / 기준: ") {
            let language = lpm_localized_language_display_name(lang, language);
            let source = lpm_translate_exact_stage8(lang, source).unwrap_or(source);
            return Some(format!(
                "[{}] {}: {} / {}: {}",
                lpm_stage11_label(lang, "settings_tag"),
                lpm_stage11_label(lang, "initial_language"),
                language,
                lpm_stage11_label(lang, "source"),
                source
            ));
        }
    }
    if let Some(rest) = text.strip_prefix("[설정] 언어 설정 파일 경로: ") {
        return Some(format!(
            "[{}] {}: {rest}",
            lpm_stage11_label(lang, "settings_tag"),
            lpm_stage11_label(lang, "language_file_path")
        ));
    }
    if let Some(rest) = text.strip_prefix("[설정] 프로그램 언어를 선택했습니다: ") {
        let language = lpm_localized_language_display_name(lang, rest);
        return Some(format!(
            "[{}] {}: {}",
            lpm_stage11_label(lang, "settings_tag"),
            lpm_stage11_label(lang, "selected_program_language"),
            language
        ));
    }
    if let Some(rest) = text.strip_prefix("[설정] 언어 설정 파일에 저장했습니다: ") {
        return Some(format!(
            "[{}] {}: {rest}",
            lpm_stage11_label(lang, "settings_tag"),
            lpm_stage11_label(lang, "language_file_saved")
        ));
    }
    if let Some(rest) = text.strip_prefix("선택한 국가 코드: ") {
        return Some(format!("{}: {}", lpm_translate_owned("선택한 국가 코드".to_string()), lpm_translate_owned(rest.to_string())));
    }

    None
}

fn lpm_translate_stage11_final_cleanup(lang: LanguageOption, content: String) -> String {
    if lang.is_korean() {
        return content;
    }

    let mut out = content;
    for (from, to) in lpm_stage11_cleanup_pairs(lang) {
        out = out.replace(from, to);
    }
    out
}

fn lpm_stage11_cleanup_pairs(lang: LanguageOption) -> &'static [(&'static str, &'static str)] {
    use LanguageOption::*;
    match lang {
        English => &[
            ("선택 안 됨", "Not selected"), ("select 안 됨", "Not selected"), ("실제 값과 다를 수 있음", "may differ from the actual value"),
            ("image 펌웨어 유형", "image firmware type"), ("펌웨어 유형", "Firmware type"), ("배터리 잔량", "Battery level"), ("가동 시간", "Uptime"), ("연결한 기기", "Connected device"),
            ("ROM 타입", "ROM type"), ("모델의 설치 금지 버전이 등록되어 있지 않습니다", "model has no blocked firmware versions registered"),
            ("성공", "success"), ("실패", "failed"), ("완료", "completed"), ("통과", "passed"), ("모두", "all"), ("경고", "warning"), ("개", " items"),
            ("일반 설치", "normal install"), ("재설치", "reinstall"), ("데이터 초기화", "factory reset"), ("데이터 유지", "keep data"), ("사용 가능", "available"), ("적용됨", "applied"), ("변경", "changes"),
            ("검증", "validation"), ("모드는", "mode"), ("없이", "without"), ("단계로 진행합니다", "stage is used"), ("정보", "information"), ("목록", "list"), ("읽기", "read"), ("상세", "details"), ("필수", "required"), ("검사", "check"),
        ],
        Russian => &[
            ("선택 안 됨", "Не выбрано"), ("select 안 됨", "Не выбрано"), ("실제 값과 다를 수 있음", "может отличаться от фактического значения"),
            ("연결한 Устройство", "Подключённое устройство"), ("배터리 잔량", "Заряд батареи"), ("가동 시간", "Время работы"), ("펌웨어 유형", "Тип прошивки"), ("прошивка 유형", "Тип прошивки"),
            ("Устройство에", "На устройстве"), ("국가 코드를", "код страны"), ("변경합니다", "изменяется"), ("활성화 할 경우", "Если включено"), ("сброс 합니다", "будет сброшено"),
            ("설치된 롬", "установленная ROM"), ("image 폴더 롬", "ROM папки image"), ("선택한 국가 코드", "Выбранный код страны"),
            ("ROM 타입", "Тип ROM"), ("모델의 설치 금지 버전이 등록되어 있지 않습니다", "для модели нет зарегистрированных запрещённых версий"),
            ("성공", "успешно"), ("실패", "ошибка"), ("완료", "завершено"), ("통과", "пройдено"), ("모두", "все"), ("경고", "предупреждение"), ("개", " шт."),
            ("일반 설치", "обычная установка"), ("재설치", "переустановка"), ("데이터 초기화", "сброс данных"), ("데이터 유지", "сохранить данные"), ("사용 가능", "доступно"), ("적용됨", "применено"), ("변경", "изменения"),
        ],
        Japanese => &[
            ("선택 안 됨", "未選択"), ("select 안 됨", "未選択"), ("실제 값과 다를 수 있음", "実際の値と異なる場合があります"),
            ("연결한 端末", "接続した端末"), ("端末에", "端末の"), ("배터리 잔량", "バッテリー残量"), ("가동 시간", "稼働時間"), ("펌웨어 유형", "ファームウェア種類"),
            ("image 폴더 롬", "imageフォルダーROM"), ("기기에 설치된 롬", "端末にインストール済みのROM"), ("선택한 국가 코드", "選択した国コード"),
            ("ROM 타입", "ROM種類"), ("모델의 설치 금지 버전이 등록되어 있지 않습니다", "モデルにはインストール禁止バージョンが登録されていません"),
            ("성공", "成功"), ("실패", "失敗"), ("완료", "完了"), ("통과", "合格"), ("모두", "すべて"), ("경고", "警告"), ("개", "件"),
            ("일반 설치", "通常インストール"), ("재설치", "再インストール"), ("데이터 초기화", "データ初期化"), ("데이터 유지", "データ保持"), ("사용 가능", "使用可能"), ("적용됨", "適用済み"), ("변경", "変更"),
            ("활성화 할 경우", "有効にすると"), ("합니다", "します"), ("입니다", "です"),
        ],
        TraditionalChinese => &[
            ("선택 안 됨", "未選擇"), ("select 안 됨", "未選擇"), ("실제 값과 다를 수 있음", "可能與實際值不同"),
            ("image 資料夾 選擇됨", "已選擇 image 資料夾"), ("韌體 資訊 및 安裝 환경 檢查를 開始", "開始檢查韌體資訊與安裝環境"), ("韌體 檢查 完成", "韌體檢查完成"),
            ("ROM 타입", "ROM 類型"), ("모델의 安裝 금지 版本이 등록되어 있지 않습니다", "型號沒有登錄禁止安裝版本"),
            ("scatter XML 파싱", "scatter XML 解析"), ("partition 목록 읽기", "讀取 partition 清單"), ("partition 상세 資訊 읽기", "讀取 partition 詳細資訊"),
            ("必要 partition 상세", "必要 partition 詳細"), ("patch plan 생성", "建立 patch plan"), ("patch plan 미리보기", "patch plan 預覽"), ("patch plan 적용", "套用 patch plan"),
            ("patch 적용 미리보기", "patch 套用預覽"), ("patch 결과 재검증", "重新驗證 patch 結果"), ("검증 警告", "驗證警告"),
            ("일반 安裝", "一般安裝"), ("재安裝", "重新安裝"), ("데이터 초기화", "清除資料"), ("데이터 유지", "保留資料"), ("사용 가능", "可使用"), ("적용됨", "已套用"), ("변경", "變更"),
            ("모두 通過", "全部通過"), ("通過", "通過"), ("成功", "成功"), ("警告", "警告"), ("개", "個"), ("個", "個"),
            ("文字 檔案을", "文字檔"), ("에 儲存합니다", "已儲存"), ("프日誌램 언어를 選擇했습니다", "已選擇程式語言"), ("언어를 選擇했습니다", "已選擇語言"),
            ("연결한 裝置", "已連接裝置"), ("배터리 잔량", "電池電量"), ("가동 시간", "運作時間"), ("펌웨어 유형", "韌體類型"), ("선택한 국가 코드", "已選擇的國家代碼"),
            ("활성화 할 경우", "啟用時"), ("변경합니다", "變更"),
        ],
        Spanish => &[
            ("선택 안 됨", "No seleccionado"), ("select 안 됨", "No seleccionado"), ("실제 값과 다를 수 있음", "puede diferir del valor real"),
            ("연결한 기기", "Dispositivo conectado"), ("배터리 잔량", "Batería restante"), ("가동 시간", "Tiempo activo"), ("펌웨어 유형", "Tipo de firmware"), ("선택한 국가 코드", "Código de país seleccionado"),
            ("ROM 타입", "Tipo de ROM"), ("성공", "correcto"), ("실패", "fallido"), ("완료", "completado"), ("통과", "superado"), ("경고", "advertencia"), ("개", " elementos"),
            ("일반 설치", "instalación normal"), ("재설치", "reinstalación"), ("데이터 초기화", "restablecer datos"), ("데이터 유지", "conservar datos"), ("사용 가능", "disponible"), ("적용됨", "aplicado"),
        ],
        Vietnamese => &[("선택 안 됨", "Chưa chọn"), ("실제 값과 다를 수 있음", "có thể khác giá trị thực tế"), ("연결한 기기", "Thiết bị đã kết nối"), ("배터리 잔량", "Pin còn lại"), ("가동 시간", "Thời gian hoạt động"), ("펌웨어 유형", "Loại firmware")],
        Greek => &[("선택 안 됨", "Δεν επιλέχθηκε"), ("실제 값과 다를 수 있음", "μπορεί να διαφέρει από την πραγματική τιμή"), ("연결한 기기", "Συνδεδεμένη συσκευή"), ("배터리 잔량", "Υπόλοιπο μπαταρίας"), ("가동 시간", "Χρόνος λειτουργίας"), ("펌웨어 유형", "Τύπος firmware")],
        Hindi => &[("선택 안 됨", "चयनित नहीं"), ("실제 값과 다를 수 있음", "वास्तविक मान से भिन्न हो सकता है"), ("연결한 기기", "कनेक्टेड डिवाइस"), ("배터리 잔량", "बैटरी स्तर"), ("가동 시간", "अपटाइम"), ("펌웨어 유형", "फर्मवेयर प्रकार")],
        Georgian => &[("선택 안 됨", "არ არის არჩეული"), ("실제 값과 다를 수 있음", "შეიძლება განსხვავდებოდეს რეალური მნიშვნელობისგან"), ("연결한 기기", "დაკავშირებული მოწყობილობა"), ("배터리 잔량", "ბატარეის დონე"), ("가동 시간", "მუშაობის დრო"), ("펌웨어 유형", "Firmware ტიპი")],
        Dutch => &[("선택 안 됨", "Niet geselecteerd"), ("실제 값과 다를 수 있음", "kan afwijken van de werkelijke waarde"), ("연결한 기기", "Verbonden apparaat"), ("배터리 잔량", "Batterijniveau"), ("가동 시간", "Uptime"), ("펌웨어 유형", "Firmwaretype")],
        Arabic => &[("선택 안 됨", "غير محدد"), ("실제 값과 다를 수 있음", "قد يختلف عن القيمة الفعلية"), ("연결한 기기", "الجهاز المتصل"), ("배터리 잔량", "مستوى البطارية"), ("가동 시간", "مدة التشغيل"), ("펌웨어 유형", "نوع firmware")],
        Korean => &[],
    }
}

fn lpm_translate_v312_runtime_message(lang: LanguageOption, text: &str) -> Option<String> {
    if let Some(rest) = text.strip_prefix("[Image] 온라인 차단 펌웨어 규칙을 확인합니다: ") {
        return Some(format!(
            "[Image] {}: {rest}",
            lpm_lang_text(
                lang,
                "Checking online blocked-firmware rules",
                "Проверка онлайн-правил блокировки прошивок",
                "オンラインのファームウェア遮断ルールを確認しています",
                "正在檢查線上韌體封鎖規則",
                "Đang kiểm tra quy tắc chặn firmware trực tuyến",
                "Έλεγχος διαδικτυακών κανόνων αποκλεισμού firmware",
                "ऑनलाइन फर्मवेयर अवरोध नियमों की जाँच",
                "Firmware-ის ონლაინ დაბლოკვის წესების შემოწმება",
                "Online regels voor geblokkeerde firmware controleren",
                "جارٍ التحقق من قواعد حظر البرامج الثابتة عبر الإنترنت",
                "Comprobando las reglas en línea de bloqueo de firmware",
            )
        ));
    }

    if let Some(rest) = text.strip_prefix("[Image] 온라인 차단 펌웨어 규칙 확인 완료: ") {
        return Some(format!(
            "[Image] {}: {rest}",
            lpm_lang_text(
                lang,
                "Online blocked-firmware rules checked",
                "Онлайн-правила блокировки прошивок проверены",
                "オンラインのファームウェア遮断ルールを確認しました",
                "線上韌體封鎖規則檢查完成",
                "Đã kiểm tra quy tắc chặn firmware trực tuyến",
                "Ο έλεγχος των διαδικτυακών κανόνων αποκλεισμού firmware ολοκληρώθηκε",
                "ऑनलाइन फर्मवेयर अवरोध नियमों की जाँच पूरी हुई",
                "Firmware-ის ონლაინ დაბლოკვის წესები შემოწმებულია",
                "Online regels voor geblokkeerde firmware gecontroleerd",
                "اكتمل التحقق من قواعد حظر البرامج الثابتة عبر الإنترنت",
                "Reglas en línea de bloqueo de firmware comprobadas",
            )
        ));
    }

    if let Some(rest) = text.strip_prefix("온라인 차단 펌웨어 규칙 확인 실패: ") {
        return Some(format!(
            "{}: {}",
            lpm_lang_text(
                lang,
                "Failed to check online blocked-firmware rules",
                "Не удалось проверить онлайн-правила блокировки прошивок",
                "オンラインのファームウェア遮断ルールを確認できませんでした",
                "線上韌體封鎖規則檢查失敗",
                "Không thể kiểm tra quy tắc chặn firmware trực tuyến",
                "Αποτυχία ελέγχου των διαδικτυακών κανόνων αποκλεισμού firmware",
                "ऑनलाइन फर्मवेयर अवरोध नियमों की जाँच विफल रही",
                "Firmware-ის ონლაინ დაბლოკვის წესების შემოწმება ვერ მოხერხდა",
                "Controleren van online regels voor geblokkeerde firmware mislukt",
                "فشل التحقق من قواعد حظر البرامج الثابتة عبر الإنترنت",
                "No se pudieron comprobar las reglas en línea de bloqueo de firmware",
            ),
            lpm_translate_v312_rule_error_detail(lang, rest)
        ));
    }

    let translated = match text {
        "[ADB] adb reboot 1회 실행 중..." => lpm_lang_text(
            lang,
            "[ADB] Running adb reboot once...",
            "[ADB] Однократный запуск adb reboot...",
            "[ADB] adb reboot を1回実行しています...",
            "[ADB] 正在執行一次 adb reboot...",
            "[ADB] Đang chạy adb reboot một lần...",
            "[ADB] Εκτέλεση adb reboot μία φορά...",
            "[ADB] adb reboot एक बार चलाया जा रहा है...",
            "[ADB] adb reboot ერთხელ სრულდება...",
            "[ADB] adb reboot één keer uitvoeren...",
            "[ADB] جارٍ تنفيذ adb reboot مرة واحدة...",
            "[ADB] Ejecutando adb reboot una vez...",
        ),
        "[ADB] adb reboot 1회 실행 완료" => lpm_lang_text(
            lang,
            "[ADB] adb reboot completed once",
            "[ADB] adb reboot выполнен один раз",
            "[ADB] adb reboot を1回実行しました",
            "[ADB] adb reboot 已執行一次",
            "[ADB] Đã chạy adb reboot một lần",
            "[ADB] Το adb reboot εκτελέστηκε μία φορά",
            "[ADB] adb reboot एक बार पूरा हुआ",
            "[ADB] adb reboot ერთხელ დასრულდა",
            "[ADB] adb reboot één keer voltooid",
            "[ADB] اكتمل تنفيذ adb reboot مرة واحدة",
            "[ADB] adb reboot se ejecutó una vez",
        ),
        "[ADB] adb reboot 1회 실행 실패, Fastboot 명령을 계속 시도합니다." => lpm_lang_text(
            lang,
            "[ADB] adb reboot failed; continuing with the Fastboot command.",
            "[ADB] adb reboot не выполнен; продолжается попытка команды Fastboot.",
            "[ADB] adb reboot に失敗したため、Fastboot コマンドを続けて試します。",
            "[ADB] adb reboot 執行失敗，繼續嘗試 Fastboot 指令。",
            "[ADB] adb reboot thất bại; tiếp tục thử lệnh Fastboot.",
            "[ADB] Το adb reboot απέτυχε· συνεχίζεται η προσπάθεια με την εντολή Fastboot.",
            "[ADB] adb reboot विफल रहा; Fastboot कमांड का प्रयास जारी है।",
            "[ADB] adb reboot ვერ შესრულდა; Fastboot ბრძანების მცდელობა გრძელდება.",
            "[ADB] adb reboot mislukt; de Fastboot-opdracht wordt verder geprobeerd.",
            "[ADB] فشل adb reboot؛ ستستمر محاولة أمر Fastboot.",
            "[ADB] adb reboot falló; se continuará con el comando Fastboot.",
        ),
        "[Fastboot] fastboot reboot 1회 실행 중..." => lpm_lang_text(
            lang,
            "[Fastboot] Running fastboot reboot once...",
            "[Fastboot] Однократный запуск fastboot reboot...",
            "[Fastboot] fastboot reboot を1回実行しています...",
            "[Fastboot] 正在執行一次 fastboot reboot...",
            "[Fastboot] Đang chạy fastboot reboot một lần...",
            "[Fastboot] Εκτέλεση fastboot reboot μία φορά...",
            "[Fastboot] fastboot reboot एक बार चलाया जा रहा है...",
            "[Fastboot] fastboot reboot ერთხელ სრულდება...",
            "[Fastboot] fastboot reboot één keer uitvoeren...",
            "[Fastboot] جارٍ تنفيذ fastboot reboot مرة واحدة...",
            "[Fastboot] Ejecutando fastboot reboot una vez...",
        ),
        "[Fastboot] fastboot reboot 1회 실행 완료" => lpm_lang_text(
            lang,
            "[Fastboot] fastboot reboot completed once",
            "[Fastboot] fastboot reboot выполнен один раз",
            "[Fastboot] fastboot reboot を1回実行しました",
            "[Fastboot] fastboot reboot 已執行一次",
            "[Fastboot] Đã chạy fastboot reboot một lần",
            "[Fastboot] Το fastboot reboot εκτελέστηκε μία φορά",
            "[Fastboot] fastboot reboot एक बार पूरा हुआ",
            "[Fastboot] fastboot reboot ერთხელ დასრულდა",
            "[Fastboot] fastboot reboot één keer voltooid",
            "[Fastboot] اكتمل تنفيذ fastboot reboot مرة واحدة",
            "[Fastboot] fastboot reboot se ejecutó una vez",
        ),
        "[Fastboot] fastboot reboot 1회 실행 실패, PreLoader 감지를 계속합니다." => lpm_lang_text(
            lang,
            "[Fastboot] fastboot reboot failed; continuing with PreLoader detection.",
            "[Fastboot] fastboot reboot не выполнен; продолжается обнаружение PreLoader.",
            "[Fastboot] fastboot reboot に失敗しました。PreLoader の検出を続行します。",
            "[Fastboot] fastboot reboot 執行失敗，繼續偵測 PreLoader。",
            "[Fastboot] fastboot reboot thất bại; tiếp tục phát hiện PreLoader.",
            "[Fastboot] Το fastboot reboot απέτυχε· συνεχίζεται ο εντοπισμός PreLoader.",
            "[Fastboot] fastboot reboot विफल रहा; PreLoader पहचान जारी है।",
            "[Fastboot] fastboot reboot ვერ შესრულდა; PreLoader-ის აღმოჩენა გრძელდება.",
            "[Fastboot] fastboot reboot mislukt; PreLoader-detectie gaat door.",
            "[Fastboot] فشل fastboot reboot؛ سيستمر اكتشاف PreLoader.",
            "[Fastboot] fastboot reboot falló; continuará la detección de PreLoader.",
        ),
        _ => return None,
    };

    Some(translated.to_string())
}

fn lpm_translate_v312_rule_error_detail(lang: LanguageOption, detail: &str) -> String {
    let detail = detail
        .strip_prefix("파일을 찾을 수 없습니다: ")
        .unwrap_or(detail);

    let mappings = [
        (
            "차단 펌웨어 규칙 확인 실패: ",
            lpm_lang_text(lang, "Request failed", "Ошибка запроса", "要求に失敗しました", "要求失敗", "Yêu cầu thất bại", "Αποτυχία αιτήματος", "अनुरोध विफल", "მოთხოვნა ვერ შესრულდა", "Verzoek mislukt", "فشل الطلب", "La solicitud falló"),
        ),
        (
            "차단 펌웨어 규칙 HTTP 상태 코드 오류: ",
            lpm_lang_text(lang, "HTTP status error", "Ошибка статуса HTTP", "HTTP ステータスエラー", "HTTP 狀態錯誤", "Lỗi trạng thái HTTP", "Σφάλμα κατάστασης HTTP", "HTTP स्थिति त्रुटि", "HTTP სტატუსის შეცდომა", "HTTP-statusfout", "خطأ حالة HTTP", "Error de estado HTTP"),
        ),
        (
            "차단 펌웨어 규칙 응답 읽기 실패: ",
            lpm_lang_text(lang, "Failed to read the response", "Не удалось прочитать ответ", "応答を読み取れませんでした", "無法讀取回應", "Không thể đọc phản hồi", "Αποτυχία ανάγνωσης της απόκρισης", "प्रतिक्रिया पढ़ने में विफल", "პასუხის წაკითხვა ვერ მოხერხდა", "Lezen van antwoord mislukt", "فشل قراءة الاستجابة", "No se pudo leer la respuesta"),
        ),
    ];

    for (prefix, label) in mappings {
        if let Some(rest) = detail.strip_prefix(prefix) {
            return format!("{label}: {rest}");
        }
    }

    match detail {
        "차단 펌웨어 규칙 응답이 비어 있습니다." => lpm_lang_text(lang, "The response is empty.", "Ответ пуст.", "応答が空です。", "回應內容為空。", "Phản hồi trống.", "Η απόκριση είναι κενή.", "प्रतिक्रिया खाली है।", "პასუხი ცარიელია.", "Het antwoord is leeg.", "الاستجابة فارغة.", "La respuesta está vacía.").to_string(),
        "차단 펌웨어 규칙 대신 HTML 응답을 받았습니다." => lpm_lang_text(lang, "An HTML response was received instead of the rules.", "Вместо правил получен HTML-ответ.", "ルールの代わりに HTML 応答を受信しました。", "收到 HTML 回應，而非規則內容。", "Đã nhận phản hồi HTML thay vì nội dung quy tắc.", "Λήφθηκε απόκριση HTML αντί για τους κανόνες.", "नियमों के बजाय HTML प्रतिक्रिया मिली।", "წესების ნაცვლად მიღებულია HTML პასუხი.", "Er is een HTML-antwoord ontvangen in plaats van de regels.", "تم تلقي استجابة HTML بدلاً من القواعد.", "Se recibió una respuesta HTML en lugar de las reglas.").to_string(),
        "차단 펌웨어 규칙에서 지원 모델 항목을 찾지 못했습니다." => lpm_lang_text(lang, "No supported-model entry was found in the rules.", "В правилах не найдена запись поддерживаемой модели.", "ルール内に対応モデルの項目が見つかりません。", "規則中找不到支援型號項目。", "Không tìm thấy mục model được hỗ trợ trong quy tắc.", "Δεν βρέθηκε καταχώριση υποστηριζόμενου μοντέλου στους κανόνες.", "नियमों में समर्थित मॉडल प्रविष्टि नहीं मिली।", "წესებში მხარდაჭერილი მოდელის ჩანაწერი ვერ მოიძებნა.", "Er is geen vermelding voor een ondersteund model gevonden.", "لم يتم العثور على إدخال لطراز مدعوم في القواعد.", "No se encontró una entrada de modelo compatible en las reglas.").to_string(),
        "차단 펌웨어 규칙 메모리 잠금 실패" => lpm_lang_text(lang, "Failed to lock the in-memory rules.", "Не удалось заблокировать правила в памяти.", "メモリ内ルールのロックに失敗しました。", "無法鎖定記憶體中的規則。", "Không thể khóa quy tắc trong bộ nhớ.", "Αποτυχία κλειδώματος των κανόνων στη μνήμη.", "मेमोरी में नियम लॉक करने में विफल।", "მეხსიერებაში წესების დაბლოკვა ვერ მოხერხდა.", "Vergrendelen van de regels in het geheugen mislukt.", "فشل قفل القواعد في الذاكرة.", "No se pudieron bloquear las reglas en memoria.").to_string(),
        _ => enforce_selected_language_output(lang, detail, detail.to_string()),
    }
}

fn lpm_translate_known_runtime_log(lang: LanguageOption, text: &str) -> Option<String> {
    if lang.is_korean() {
        return None;
    }

    let exact = match text {
        "선택한 image 폴더 정보 및 설치 환경 검사를 시작합니다." => Some(lpm_lang_text(lang, "Starting the selected image folder and installation environment check.", "Запуск проверки выбранной папки image и среды установки.", "選択したimageフォルダーとインストール環境の確認を開始します。", "開始檢查已選擇的 image 資料夾與安裝環境。", "Bắt đầu kiểm tra thư mục image đã chọn và môi trường cài đặt.", "Έναρξη ελέγχου του επιλεγμένου φακέλου image και του περιβάλλοντος εγκατάστασης.", "चयनित image फ़ोल्डर और इंस्टॉलेशन वातावरण की जाँच शुरू की जा रही है।", "არჩეული image საქაღალდისა და ინსტალაციის გარემოს შემოწმება იწყება.", "De controle van de geselecteerde image-map en installatieomgeving wordt gestart.", "بدء فحص مجلد image المحدد وبيئة التثبيت.", "Iniciando la comprobación de la carpeta image seleccionada y del entorno de instalación.")),
        "펌웨어 검사 완료" => Some(lpm_lang_text(lang, "Firmware check completed", "Проверка прошивки завершена", "ファームウェアの確認が完了しました", "韌體檢查完成", "Đã kiểm tra firmware", "Ο έλεγχος firmware ολοκληρώθηκε", "फर्मवेयर जाँच पूरी हुई", "Firmware-ის შემოწმება დასრულდა", "Firmwarecontrole voltooid", "اكتمل فحص البرنامج الثابت", "Comprobación de firmware completada")),
        "image 폴더 선택을 취소했습니다." => Some(lpm_lang_text(lang, "Image folder selection was cancelled.", "Выбор папки image отменён.", "imageフォルダーの選択をキャンセルしました。", "已取消選擇 image 資料夾。", "Đã hủy chọn thư mục image.", "Η επιλογή φακέλου image ακυρώθηκε.", "image फ़ोल्डर चयन रद्द कर दिया गया।", "image საქაღალდის არჩევა გაუქმდა.", "Selectie van de image-map is geannuleerd.", "تم إلغاء اختيار مجلد image.", "Se canceló la selección de la carpeta image.")),
        "1번 옵션을 시작합니다: PRC/ROW 펌웨어 설치 [데이터 초기화]" => Some(lpm_lang_text(lang, "Starting option 1: PRC/ROW firmware installation [factory reset]", "Запуск варианта 1: установка прошивки PRC/ROW [сброс данных]", "オプション1を開始します：PRC/ROWファームウェアのインストール［データ初期化］", "開始選項 1：安裝 PRC/ROW 韌體［清除資料］", "Bắt đầu tùy chọn 1: cài firmware PRC/ROW [xóa dữ liệu]", "Έναρξη επιλογής 1: εγκατάσταση firmware PRC/ROW [επαναφορά δεδομένων]", "विकल्प 1 शुरू: PRC/ROW फर्मवेयर इंस्टॉल [डेटा रीसेट]", "იწყება ვარიანტი 1: PRC/ROW firmware-ის დაყენება [მონაცემების წაშლა]", "Optie 1 starten: PRC/ROW-firmware installeren [fabrieksreset]", "بدء الخيار 1: تثبيت برنامج PRC/ROW الثابت [إعادة ضبط البيانات]", "Iniciando la opción 1: instalación de firmware PRC/ROW [restablecer datos]")),
        "2번 옵션을 시작합니다: ROW(글로벌) 펌웨어 업데이트 [데이터 유지]" => Some(lpm_lang_text(lang, "Starting option 2: ROW (Global ROM) firmware update [keep data]", "Запуск варианта 2: обновление прошивки ROW (глобальная ROM) [сохранение данных]", "オプション2を開始します：ROW（グローバルROM）ファームウェア更新［データ保持］", "開始選項 2：更新 ROW（全球版 ROM）韌體［保留資料］", "Bắt đầu tùy chọn 2: cập nhật firmware ROW (ROM toàn cầu) [giữ dữ liệu]", "Έναρξη επιλογής 2: ενημέρωση firmware ROW (Global ROM) [διατήρηση δεδομένων]", "विकल्प 2 शुरू: ROW (ग्लोबल ROM) फर्मवेयर अपडेट [डेटा रखें]", "იწყება ვარიანტი 2: ROW (გლობალური ROM) firmware-ის განახლება [მონაცემების შენარჩუნება]", "Optie 2 starten: ROW-firmware (Global ROM) bijwerken [gegevens behouden]", "بدء الخيار 2: تحديث برنامج ROW الثابت (الروم العالمي) [الاحتفاظ بالبيانات]", "Iniciando la opción 2: actualización de firmware ROW (ROM global) [conservar datos]")),
        "3번 옵션을 시작합니다: 기기 복구 [데이터 초기화]" => Some(lpm_lang_text(lang, "Starting option 3: device recovery [factory reset]", "Запуск варианта 3: восстановление устройства [сброс данных]", "オプション3を開始します：デバイス復旧［データ初期化］", "開始選項 3：裝置修復［清除資料］", "Bắt đầu tùy chọn 3: khôi phục thiết bị [xóa dữ liệu]", "Έναρξη επιλογής 3: ανάκτηση συσκευής [επαναφορά δεδομένων]", "विकल्प 3 शुरू: डिवाइस रिकवरी [डेटा रीसेट]", "იწყება ვარიანტი 3: მოწყობილობის აღდგენა [მონაცემების წაშლა]", "Optie 3 starten: apparaatherstel [fabrieksreset]", "بدء الخيار 3: استرداد الجهاز [إعادة ضبط البيانات]", "Iniciando la opción 3: recuperación del dispositivo [restablecer datos]")),
        "[1단계] ADB 기기 감지 및 기기 정보 확인" => Some(lpm_lang_text(lang, "[Step 1] Detecting the ADB device and checking device information", "[Шаг 1] Обнаружение устройства ADB и проверка сведений об устройстве", "[ステップ1] ADBデバイスを検出し、デバイス情報を確認", "[步驟 1] 偵測 ADB 裝置並檢查裝置資訊", "[Bước 1] Phát hiện thiết bị ADB và kiểm tra thông tin thiết bị", "[Βήμα 1] Εντοπισμός συσκευής ADB και έλεγχος πληροφοριών συσκευής", "[चरण 1] ADB डिवाइस पहचान और डिवाइस जानकारी की जाँच", "[ნაბიჯი 1] ADB მოწყობილობის აღმოჩენა და ინფორმაციის შემოწმება", "[Stap 1] ADB-apparaat detecteren en apparaatgegevens controleren", "[الخطوة 1] اكتشاف جهاز ADB والتحقق من معلومات الجهاز", "[Paso 1] Detectar el dispositivo ADB y comprobar la información del dispositivo")),
        "[2단계] image 폴더 검사 및 플래싱 준비" => Some(lpm_lang_text(lang, "[Step 2] Checking the image folder and preparing to flash", "[Шаг 2] Проверка папки image и подготовка к прошивке", "[ステップ2] imageフォルダーを確認し、フラッシュの準備", "[步驟 2] 檢查 image 資料夾並準備刷寫", "[Bước 2] Kiểm tra thư mục image và chuẩn bị flash", "[Βήμα 2] Έλεγχος φακέλου image και προετοιμασία flash", "[चरण 2] image फ़ोल्डर जाँच और फ्लैश की तैयारी", "[ნაბიჯი 2] image საქაღალდის შემოწმება და ჩაწერის მომზადება", "[Stap 2] Image-map controleren en flash voorbereiden", "[الخطوة 2] فحص مجلد image والتحضير للتفليش", "[Paso 2] Comprobar la carpeta image y preparar el flasheo")),
        "[2단계] image 폴더 기본 검사" => Some(lpm_lang_text(lang, "[Step 2] Basic image folder check", "[Шаг 2] Базовая проверка папки image", "[ステップ2] imageフォルダーの基本確認", "[步驟 2] image 資料夾基本檢查", "[Bước 2] Kiểm tra cơ bản thư mục image", "[Βήμα 2] Βασικός έλεγχος φακέλου image", "[चरण 2] image फ़ोल्डर की मूल जाँच", "[ნაბიჯი 2] image საქაღალდის ძირითადი შემოწმება", "[Stap 2] Basiscontrole van de image-map", "[الخطوة 2] الفحص الأساسي لمجلد image", "[Paso 2] Comprobación básica de la carpeta image")),
        "[3단계] Flash Plan 준비 및 작업용 scatter/xml 생성" => Some(lpm_lang_text(lang, "[Step 3] Preparing the Flash Plan and generating the working scatter/XML files", "[Шаг 3] Подготовка Flash Plan и создание рабочих файлов scatter/XML", "[ステップ3] Flash Planを準備し、作業用scatter/XMLを生成", "[步驟 3] 準備 Flash Plan 並產生工作用 scatter/XML", "[Bước 3] Chuẩn bị Flash Plan và tạo scatter/XML làm việc", "[Βήμα 3] Προετοιμασία Flash Plan και δημιουργία αρχείων scatter/XML εργασίας", "[चरण 3] Flash Plan तैयार करना और कार्य scatter/XML बनाना", "[ნაბიჯი 3] Flash Plan-ის მომზადება და სამუშაო scatter/XML-ის შექმნა", "[Stap 3] Flash Plan voorbereiden en werk-scatter/XML genereren", "[الخطوة 3] إعداد Flash Plan وإنشاء ملفات scatter/XML للعمل", "[Paso 3] Preparar el Flash Plan y generar scatter/XML de trabajo")),
        "[4단계] current slot A 설정" => Some(lpm_lang_text(lang, "[Step 4] Setting current slot A", "[Шаг 4] Настройка текущего слота A", "[ステップ4] current slotをAに設定", "[步驟 4] 將目前 slot 設為 A", "[Bước 4] Đặt current slot thành A", "[Βήμα 4] Ρύθμιση του current slot σε A", "[चरण 4] current slot को A पर सेट करना", "[ნაბიჯი 4] current slot-ის A-ზე დაყენება", "[Stap 4] Current slot instellen op A", "[الخطوة 4] تعيين current slot إلى A", "[Paso 4] Establecer current slot en A")),
        "[5단계] ROM 설치" => Some(lpm_lang_text(lang, "[Step 5] Installing the ROM", "[Шаг 5] Установка ROM", "[ステップ5] ROMをインストール", "[步驟 5] 安裝 ROM", "[Bước 5] Cài đặt ROM", "[Βήμα 5] Εγκατάσταση ROM", "[चरण 5] ROM इंस्टॉल", "[ნაბიჯი 5] ROM-ის დაყენება", "[Stap 5] ROM installeren", "[الخطوة 5] تثبيت ROM", "[Paso 5] Instalar la ROM")),
        "[6단계] MediaTek PreLoader 포트 감지" => Some(lpm_lang_text(lang, "[Step 6] Detecting the MediaTek PreLoader port", "[Шаг 6] Обнаружение порта MediaTek PreLoader", "[ステップ6] MediaTek PreLoaderポートを検出", "[步驟 6] 偵測 MediaTek PreLoader 連接埠", "[Bước 6] Phát hiện cổng MediaTek PreLoader", "[Βήμα 6] Εντοπισμός θύρας MediaTek PreLoader", "[चरण 6] MediaTek PreLoader पोर्ट पहचान", "[ნაბიჯი 6] MediaTek PreLoader პორტის აღმოჩენა", "[Stap 6] MediaTek PreLoader-poort detecteren", "[الخطوة 6] اكتشاف منفذ MediaTek PreLoader", "[Paso 6] Detectar el puerto MediaTek PreLoader")),
        "[7단계] SPFlashToolV6 ROM 설치" => Some(lpm_lang_text(lang, "[Step 7] Installing the ROM with SPFlashToolV6", "[Шаг 7] Установка ROM с помощью SPFlashToolV6", "[ステップ7] SPFlashToolV6でROMをインストール", "[步驟 7] 使用 SPFlashToolV6 安裝 ROM", "[Bước 7] Cài ROM bằng SPFlashToolV6", "[Βήμα 7] Εγκατάσταση ROM με το SPFlashToolV6", "[चरण 7] SPFlashToolV6 से ROM इंस्टॉल", "[ნაბიჯი 7] ROM-ის დაყენება SPFlashToolV6-ით", "[Stap 7] ROM installeren met SPFlashToolV6", "[الخطوة 7] تثبيت ROM باستخدام SPFlashToolV6", "[Paso 7] Instalar la ROM con SPFlashToolV6")),
        "[완료] 1번 옵션 PRC/ROW 펌웨어 설치 작업이 완료되었습니다." => Some(lpm_lang_text(lang, "[Completed] Option 1 PRC/ROW firmware installation is complete.", "[Завершено] Установка прошивки PRC/ROW по варианту 1 завершена.", "[完了] オプション1のPRC/ROWファームウェアインストールが完了しました。", "[完成] 選項 1 的 PRC/ROW 韌體安裝已完成。", "[Hoàn tất] Đã hoàn tất cài firmware PRC/ROW theo tùy chọn 1.", "[Ολοκληρώθηκε] Η εγκατάσταση firmware PRC/ROW της επιλογής 1 ολοκληρώθηκε.", "[पूर्ण] विकल्प 1 का PRC/ROW फर्मवेयर इंस्टॉल पूरा हुआ।", "[დასრულდა] ვარიანტი 1-ის PRC/ROW firmware-ის დაყენება დასრულდა.", "[Voltooid] Optie 1 PRC/ROW-firmware-installatie is voltooid.", "[اكتمل] اكتمل تثبيت برنامج PRC/ROW الثابت للخيار 1.", "[Completado] Finalizó la instalación de firmware PRC/ROW de la opción 1.")),
        "[완료] 2번 옵션 ROW(글로벌) 펌웨어 업데이트 [데이터 유지] 작업이 완료되었습니다." => Some(lpm_lang_text(lang, "[Completed] Option 2 ROW firmware update [keep data] is complete.", "[Завершено] Обновление прошивки ROW по варианту 2 [сохранение данных] завершено.", "[完了] オプション2のROWファームウェア更新［データ保持］が完了しました。", "[完成] 選項 2 的 ROW 韌體更新［保留資料］已完成。", "[Hoàn tất] Đã hoàn tất cập nhật firmware ROW theo tùy chọn 2 [giữ dữ liệu].", "[Ολοκληρώθηκε] Η ενημέρωση firmware ROW της επιλογής 2 [διατήρηση δεδομένων] ολοκληρώθηκε.", "[पूर्ण] विकल्प 2 का ROW फर्मवेयर अपडेट [डेटा रखें] पूरा हुआ।", "[დასრულდა] ვარიანტი 2-ის ROW firmware-ის განახლება [მონაცემების შენარჩუნება] დასრულდა.", "[Voltooid] Optie 2 ROW-firmware-update [gegevens behouden] is voltooid.", "[اكتمل] اكتمل تحديث برنامج ROW الثابت للخيار 2 [الاحتفاظ بالبيانات].", "[Completado] Finalizó la actualización de firmware ROW de la opción 2 [conservar datos].")),
        "[완료] 기기 복구 [데이터 초기화] 작업이 완료되었습니다." => Some(lpm_lang_text(lang, "[Completed] Device recovery [factory reset] is complete.", "[Завершено] Восстановление устройства [сброс данных] завершено.", "[完了] デバイス復旧［データ初期化］が完了しました。", "[完成] 裝置修復［清除資料］已完成。", "[Hoàn tất] Đã hoàn tất khôi phục thiết bị [xóa dữ liệu].", "[Ολοκληρώθηκε] Η ανάκτηση συσκευής [επαναφορά δεδομένων] ολοκληρώθηκε.", "[पूर्ण] डिवाइस रिकवरी [डेटा रीसेट] पूरी हुई।", "[დასრულდა] მოწყობილობის აღდგენა [მონაცემების წაშლა] დასრულდა.", "[Voltooid] Apparaatherstel [fabrieksreset] is voltooid.", "[اكتمل] اكتمل استرداد الجهاز [إعادة ضبط البيانات].", "[Completado] Finalizó la recuperación del dispositivo [restablecer datos].")),
        "재설치 모드는 current slot 단계 없이 PreLoader/SPFlashToolV6 단계로 바로 진행합니다." => Some(lpm_lang_text(lang, "Recovery mode proceeds directly to PreLoader/SPFlashToolV6 without the current-slot stage.", "Режим восстановления сразу переходит к PreLoader/SPFlashToolV6 без этапа current slot.", "復旧モードではcurrent slotの手順を省略し、PreLoader/SPFlashToolV6へ直接進みます。", "修復模式會略過 current slot 步驟，直接進入 PreLoader/SPFlashToolV6。", "Chế độ khôi phục chuyển thẳng đến PreLoader/SPFlashToolV6 mà không qua bước current slot.", "Η λειτουργία ανάκτησης προχωρά απευθείας στο PreLoader/SPFlashToolV6 χωρίς το στάδιο current slot.", "रिकवरी मोड current slot चरण के बिना सीधे PreLoader/SPFlashToolV6 पर जाता है।", "აღდგენის რეჟიმი current slot ეტაპის გარეშე პირდაპირ PreLoader/SPFlashToolV6-ზე გადადის.", "De herstelmodus gaat zonder current-slotstap rechtstreeks naar PreLoader/SPFlashToolV6.", "ينتقل وضع الاسترداد مباشرةً إلى PreLoader/SPFlashToolV6 دون مرحلة current slot.", "El modo de recuperación pasa directamente a PreLoader/SPFlashToolV6 sin la etapa current slot.")),
        "[ADB] 기기 감지" | "[ADB] 기기를 감지하고 있습니다..." => Some(lpm_lang_text(lang, "[ADB] Detecting the device...", "[ADB] Обнаружение устройства...", "[ADB] デバイスを検出しています...", "[ADB] 正在偵測裝置...", "[ADB] Đang phát hiện thiết bị...", "[ADB] Εντοπισμός συσκευής...", "[ADB] डिवाइस पहचाना जा रहा है...", "[ADB] მოწყობილობის აღმოჩენა...", "[ADB] Apparaat detecteren...", "[ADB] جارٍ اكتشاف الجهاز...", "[ADB] Detectando el dispositivo...")),
        "[ADB] 기기 감지 완료" => Some(lpm_lang_text(lang, "[ADB] Device detected", "[ADB] Устройство обнаружено", "[ADB] デバイスを検出しました", "[ADB] 已偵測到裝置", "[ADB] Đã phát hiện thiết bị", "[ADB] Η συσκευή εντοπίστηκε", "[ADB] डिवाइस मिल गया", "[ADB] მოწყობილობა აღმოჩენილია", "[ADB] Apparaat gedetecteerd", "[ADB] تم اكتشاف الجهاز", "[ADB] Dispositivo detectado")),
        "[ADB] ADB 명령어로 Slot 설정 중..." => Some(lpm_lang_text(lang, "[ADB] Setting the slot with an ADB command...", "[ADB] Настройка slot командой ADB...", "[ADB] ADBコマンドでslotを設定しています...", "[ADB] 正在使用 ADB 指令設定 slot...", "[ADB] Đang đặt slot bằng lệnh ADB...", "[ADB] Ρύθμιση slot με εντολή ADB...", "[ADB] ADB कमांड से slot सेट किया जा रहा है...", "[ADB] slot-ის დაყენება ADB ბრძანებით...", "[ADB] Slot instellen met een ADB-opdracht...", "[ADB] جارٍ تعيين slot باستخدام أمر ADB...", "[ADB] Configurando el slot con un comando ADB...")),
        "[ADB] ADB 명령어로 Slot 설정 완료" => Some(lpm_lang_text(lang, "[ADB] Slot setting with ADB completed", "[ADB] Настройка slot через ADB завершена", "[ADB] ADBによるslot設定が完了しました", "[ADB] 已完成使用 ADB 設定 slot", "[ADB] Đã đặt slot bằng ADB", "[ADB] Η ρύθμιση slot μέσω ADB ολοκληρώθηκε", "[ADB] ADB से slot सेट हो गया", "[ADB] slot-ის ADB-ით დაყენება დასრულდა", "[ADB] Slot instellen via ADB voltooid", "[ADB] اكتمل تعيين slot عبر ADB", "[ADB] Configuración del slot mediante ADB completada")),
        "[ADB] bootloader 모드 설정" => Some(lpm_lang_text(lang, "[ADB] Switching to bootloader mode", "[ADB] Переход в режим bootloader", "[ADB] bootloaderモードへ切り替え", "[ADB] 切換至 bootloader 模式", "[ADB] Chuyển sang chế độ bootloader", "[ADB] Μετάβαση σε λειτουργία bootloader", "[ADB] bootloader मोड पर स्विच", "[ADB] bootloader რეჟიმზე გადასვლა", "[ADB] Overschakelen naar bootloader-modus", "[ADB] التبديل إلى وضع bootloader", "[ADB] Cambiando al modo bootloader")),
        "[Fastboot] 기기 감지중..." => Some(lpm_lang_text(lang, "[Fastboot] Detecting the device...", "[Fastboot] Обнаружение устройства...", "[Fastboot] デバイスを検出しています...", "[Fastboot] 正在偵測裝置...", "[Fastboot] Đang phát hiện thiết bị...", "[Fastboot] Εντοπισμός συσκευής...", "[Fastboot] डिवाइस पहचाना जा रहा है...", "[Fastboot] მოწყობილობის აღმოჩენა...", "[Fastboot] Apparaat detecteren...", "[Fastboot] جارٍ اكتشاف الجهاز...", "[Fastboot] Detectando el dispositivo...")),
        "[Fastboot] 기기 감지 완료" => Some(lpm_lang_text(lang, "[Fastboot] Device detected", "[Fastboot] Устройство обнаружено", "[Fastboot] デバイスを検出しました", "[Fastboot] 已偵測到裝置", "[Fastboot] Đã phát hiện thiết bị", "[Fastboot] Η συσκευή εντοπίστηκε", "[Fastboot] डिवाइस मिल गया", "[Fastboot] მოწყობილობა აღმოჩენილია", "[Fastboot] Apparaat gedetecteerd", "[Fastboot] تم اكتشاف الجهاز", "[Fastboot] Dispositivo detectado")),
        "[Fastboot] bootloader 재진입 후 Fastboot 재감지 중..." => Some(lpm_lang_text(lang, "[Fastboot] Re-entering bootloader and detecting Fastboot again...", "[Fastboot] Повторный вход в bootloader и обнаружение Fastboot...", "[Fastboot] bootloaderへ再移行し、Fastbootを再検出しています...", "[Fastboot] 重新進入 bootloader 並再次偵測 Fastboot...", "[Fastboot] Vào lại bootloader và phát hiện lại Fastboot...", "[Fastboot] Επανείσοδος στο bootloader και νέος εντοπισμός Fastboot...", "[Fastboot] bootloader में फिर प्रवेश करके Fastboot दोबारा पहचाना जा रहा है...", "[Fastboot] bootloader-ში ხელახლა შესვლა და Fastboot-ის ხელახალი აღმოჩენა...", "[Fastboot] Bootloader opnieuw openen en Fastboot opnieuw detecteren...", "[Fastboot] إعادة الدخول إلى bootloader واكتشاف Fastboot مجددًا...", "[Fastboot] Volviendo a entrar en bootloader y detectando Fastboot de nuevo...")),
        "[Fastboot] bootloader 재진입 후 Fastboot 재감지 완료" => Some(lpm_lang_text(lang, "[Fastboot] Fastboot detected after re-entering bootloader", "[Fastboot] Fastboot обнаружен после повторного входа в bootloader", "[Fastboot] bootloader再移行後のFastboot検出が完了しました", "[Fastboot] 重新進入 bootloader 後已偵測到 Fastboot", "[Fastboot] Đã phát hiện Fastboot sau khi vào lại bootloader", "[Fastboot] Το Fastboot εντοπίστηκε μετά την επανείσοδο στο bootloader", "[Fastboot] bootloader में फिर प्रवेश के बाद Fastboot मिल गया", "[Fastboot] bootloader-ში ხელახლა შესვლის შემდეგ Fastboot აღმოჩენილია", "[Fastboot] Fastboot gedetecteerd na opnieuw openen van bootloader", "[Fastboot] تم اكتشاف Fastboot بعد إعادة الدخول إلى bootloader", "[Fastboot] Fastboot detectado tras volver a entrar en bootloader")),
        "[Fastboot] 확인 완료" => Some(lpm_lang_text(lang, "[Fastboot] Verification completed", "[Fastboot] Проверка завершена", "[Fastboot] 確認が完了しました", "[Fastboot] 驗證完成", "[Fastboot] Đã xác minh", "[Fastboot] Η επαλήθευση ολοκληρώθηκε", "[Fastboot] सत्यापन पूरा हुआ", "[Fastboot] შემოწმება დასრულდა", "[Fastboot] Verificatie voltooid", "[Fastboot] اكتمل التحقق", "[Fastboot] Verificación completada")),
        "[Fastboot] 안정화를 위해 5초 대기합니다..." => Some(lpm_lang_text(lang, "[Fastboot] Waiting 5 seconds for stabilization...", "[Fastboot] Ожидание 5 секунд для стабилизации...", "[Fastboot] 安定化のため5秒待機しています...", "[Fastboot] 等待 5 秒以穩定連線...", "[Fastboot] Chờ 5 giây để ổn định...", "[Fastboot] Αναμονή 5 δευτερολέπτων για σταθεροποίηση...", "[Fastboot] स्थिरता के लिए 5 सेकंड प्रतीक्षा...", "[Fastboot] სტაბილიზაციისთვის 5 წამით მოცდა...", "[Fastboot] 5 seconden wachten op stabilisatie...", "[Fastboot] الانتظار 5 ثوانٍ للاستقرار...", "[Fastboot] Esperando 5 segundos para estabilizar...")),
        "[Fastboot] 안정화 대기 완료" => Some(lpm_lang_text(lang, "[Fastboot] Stabilization wait completed", "[Fastboot] Ожидание стабилизации завершено", "[Fastboot] 安定化待機が完了しました", "[Fastboot] 穩定等待完成", "[Fastboot] Đã chờ ổn định", "[Fastboot] Η αναμονή σταθεροποίησης ολοκληρώθηκε", "[Fastboot] स्थिरता प्रतीक्षा पूरी हुई", "[Fastboot] სტაბილიზაციის მოლოდინი დასრულდა", "[Fastboot] Wachten op stabilisatie voltooid", "[Fastboot] اكتمل انتظار الاستقرار", "[Fastboot] Espera de estabilización completada")),
        "[Fastboot] slot A 설정 중..." => Some(lpm_lang_text(lang, "[Fastboot] Setting slot A...", "[Fastboot] Настройка slot A...", "[Fastboot] slot Aを設定しています...", "[Fastboot] 正在設定 slot A...", "[Fastboot] Đang đặt slot A...", "[Fastboot] Ρύθμιση slot A...", "[Fastboot] slot A सेट किया जा रहा है...", "[Fastboot] slot A-ის დაყენება...", "[Fastboot] Slot A instellen...", "[Fastboot] جارٍ تعيين slot A...", "[Fastboot] Configurando slot A...")),
        "[Fastboot] slot A 설정 완료" => Some(lpm_lang_text(lang, "[Fastboot] Slot A setting completed", "[Fastboot] Настройка slot A завершена", "[Fastboot] slot Aの設定が完了しました", "[Fastboot] slot A 設定完成", "[Fastboot] Đã đặt slot A", "[Fastboot] Η ρύθμιση slot A ολοκληρώθηκε", "[Fastboot] slot A सेट हो गया", "[Fastboot] slot A-ის დაყენება დასრულდა", "[Fastboot] Slot A instellen voltooid", "[Fastboot] اكتمل تعيين slot A", "[Fastboot] Configuración de slot A completada")),
        "[Port] PreLoader 포트 감지 중..." => Some(lpm_lang_text(lang, "[Port] Detecting the PreLoader port...", "[Порт] Обнаружение порта PreLoader...", "[ポート] PreLoaderポートを検出しています...", "[連接埠] 正在偵測 PreLoader 連接埠...", "[Cổng] Đang phát hiện cổng PreLoader...", "[Θύρα] Εντοπισμός θύρας PreLoader...", "[पोर्ट] PreLoader पोर्ट पहचाना जा रहा है...", "[პორტი] PreLoader პორტის აღმოჩენა...", "[Poort] PreLoader-poort detecteren...", "[المنفذ] جارٍ اكتشاف منفذ PreLoader...", "[Puerto] Detectando el puerto PreLoader...")),
        "[Port] PreLoader 포트 감지 완료" => Some(lpm_lang_text(lang, "[Port] PreLoader port detected", "[Порт] Порт PreLoader обнаружен", "[ポート] PreLoaderポートを検出しました", "[連接埠] 已偵測到 PreLoader 連接埠", "[Cổng] Đã phát hiện cổng PreLoader", "[Θύρα] Η θύρα PreLoader εντοπίστηκε", "[पोर्ट] PreLoader पोर्ट मिल गया", "[პორტი] PreLoader პორტი აღმოჩენილია", "[Poort] PreLoader-poort gedetecteerd", "[المنفذ] تم اكتشاف منفذ PreLoader", "[Puerto] Puerto PreLoader detectado")),
        "[안내] PC(노트북)와 연결한 태블릿에 잠금 해제 → 메세지 창 왼쪽 중간 체크 박스 체크 → 오른쪽 하단 Allow(허용)를 터치해주세요." => Some(lpm_lang_text(lang, "[Guide] Unlock the tablet connected to the PC, select the checkbox in the authorization dialog, and tap Allow.", "[Инструкция] Разблокируйте планшет, подключённый к ПК, установите флажок в окне авторизации и нажмите Allow.", "[案内] PCに接続したタブレットのロックを解除し、認証画面のチェックボックスを選択して［Allow］をタップしてください。", "[說明] 解鎖已連接電腦的平板，在授權視窗勾選核取方塊，然後點選 Allow。", "[Hướng dẫn] Mở khóa máy tính bảng đã kết nối với PC, chọn ô trong hộp thoại cấp quyền rồi nhấn Allow.", "[Οδηγίες] Ξεκλειδώστε το tablet που είναι συνδεδεμένο στον υπολογιστή, επιλέξτε το πλαίσιο στο παράθυρο εξουσιοδότησης και πατήστε Allow.", "[निर्देश] PC से जुड़े टैबलेट को अनलॉक करें, अनुमति संवाद में चेकबॉक्स चुनें और Allow दबाएँ।", "[მითითება] განბლოკეთ PC-სთან დაკავშირებული ტაბლეტი, ავტორიზაციის ფანჯარაში მონიშნეთ ველი და დააჭირეთ Allow-ს.", "[Instructie] Ontgrendel de tablet die met de pc is verbonden, vink het selectievakje in het toestemmingsvenster aan en tik op Allow.", "[إرشاد] افتح قفل الجهاز اللوحي المتصل بالكمبيوتر، وحدد مربع الاختيار في نافذة التفويض، ثم اضغط على Allow.", "[Guía] Desbloquee la tablet conectada al PC, marque la casilla del cuadro de autorización y pulse Allow.")),
        _ => None,
    };

    if let Some(value) = exact {
        return Some(value.to_string());
    }

    if text == "!" {
        return Some("!".to_string());
    }

    if let Some(rest) = text.strip_prefix("차단 버전 목록 / ") {
        return Some(format!(
            "{}{rest}",
            lpm_lang_text(
                lang,
                "Blocked firmware versions / ",
                "Заблокированные версии прошивки / ",
                "インストール禁止ファームウェア一覧 / ",
                "封鎖韌體版本 / ",
                "Các phiên bản firmware bị chặn / ",
                "Αποκλεισμένες εκδόσεις firmware / ",
                "ब्लॉक किए गए फर्मवेयर संस्करण / ",
                "დაბლოკილი firmware ვერსიები / ",
                "Geblokkeerde firmwareversies / ",
                "إصدارات البرامج الثابتة المحظورة / ",
                "Versiones de firmware bloqueadas / ",
            )
        ));
    }

    let blocked_suffixes = [
        (
            " 버전은 설치 금지 목록에 포함되어 있습니다.",
            lpm_lang_text(
                lang,
                " is included in the blocked firmware list.",
                " входит в список запрещённых версий прошивки.",
                " はインストール禁止リストに含まれています。",
                " 已列入封鎖韌體清單。",
                " nằm trong danh sách firmware bị chặn.",
                " περιλαμβάνεται στη λίστα αποκλεισμένων firmware.",
                " ब्लॉक की गई फर्मवेयर सूची में शामिल है।",
                " შეტანილია დაბლოკილი firmware-ის სიაში.",
                " staat in de lijst met geblokkeerde firmware.",
                " موجود في قائمة البرامج الثابتة المحظورة.",
                " está incluida en la lista de firmware bloqueado.",
            ),
        ),
        (
            " 버전은 설치 금지 목록에 포함되어 있지 않습니다.",
            lpm_lang_text(
                lang,
                " is not included in the blocked firmware list.",
                " не входит в список запрещённых версий прошивки.",
                " はインストール禁止リストに含まれていません。",
                " 未列入封鎖韌體清單。",
                " không nằm trong danh sách firmware bị chặn.",
                " δεν περιλαμβάνεται στη λίστα αποκλεισμένων firmware.",
                " ब्लॉक की गई फर्मवेयर सूची में शामिल नहीं है।",
                " არ არის შეტანილი დაბლოკილი firmware-ის სიაში.",
                " staat niet in de lijst met geblokkeerde firmware.",
                " غير موجود في قائمة البرامج الثابتة المحظورة.",
                " no está incluida en la lista de firmware bloqueado.",
            ),
        ),
        (
            " 모델의 설치 금지 버전이 등록되어 있지 않습니다.",
            lpm_lang_text(
                lang,
                " has no registered blocked firmware versions.",
                " не имеет зарегистрированных запрещённых версий прошивки.",
                " には登録済みのインストール禁止バージョンがありません。",
                " 尚未登錄任何封鎖韌體版本。",
                " không có phiên bản firmware bị chặn nào được đăng ký.",
                " δεν έχει καταχωρισμένες αποκλεισμένες εκδόσεις firmware.",
                " के लिए कोई ब्लॉक किया गया फर्मवेयर संस्करण पंजीकृत नहीं है।",
                " მოდელისთვის დაბლოკილი firmware ვერსიები რეგისტრირებული არ არის.",
                " heeft geen geregistreerde geblokkeerde firmwareversies.",
                " لا توجد له إصدارات برامج ثابتة محظورة مسجلة.",
                " no tiene versiones de firmware bloqueadas registradas.",
            ),
        ),
        (
            " 차단 펌웨어 규칙을 온라인에서 확인하지 못했습니다.",
            lpm_lang_text(
                lang,
                " firmware blocking rules could not be checked online.",
                " — не удалось проверить правила блокировки прошивки в сети.",
                " のファームウェア遮断ルールをオンラインで確認できませんでした。",
                " 的韌體封鎖規則無法在線上確認。",
                " không thể kiểm tra quy tắc chặn firmware trực tuyến.",
                " — δεν ήταν δυνατός ο διαδικτυακός έλεγχος των κανόνων αποκλεισμού firmware.",
                " के फर्मवेयर ब्लॉक नियम ऑनलाइन जाँचे नहीं जा सके।",
                " firmware-ის დაბლოკვის წესები ონლაინ ვერ შემოწმდა.",
                " — de firmwareblokkeringsregels konden niet online worden gecontroleerd.",
                " تعذّر التحقق من قواعد حظر البرامج الثابتة عبر الإنترنت.",
                " no pudo comprobar en línea las reglas de bloqueo de firmware.",
            ),
        ),
    ];

    for (suffix, translated_suffix) in blocked_suffixes {
        if let Some(value) = text.strip_suffix(suffix) {
            return Some(format!("{value}{translated_suffix}"));
        }
    }

    if let Some(rest) = text.strip_prefix("[Image] image 폴더에 ") {
        if let Some(files) = rest.strip_suffix(" 파일을 복사합니다.") {
            return Some(format!(
                "{}{files}",
                lpm_lang_text(
                    lang,
                    "[Image] Copying files to the image folder: ",
                    "[Image] Копирование файлов в папку image: ",
                    "[Image] imageフォルダーへファイルをコピー：",
                    "[Image] 複製檔案至 image 資料夾：",
                    "[Image] Sao chép tệp vào thư mục image: ",
                    "[Image] Αντιγραφή αρχείων στον φάκελο image: ",
                    "[Image] image फ़ोल्डर में फ़ाइलें कॉपी की जा रही हैं: ",
                    "[Image] ფაილების image საქაღალდეში კოპირება: ",
                    "[Image] Bestanden kopiëren naar de image-map: ",
                    "[Image] نسخ الملفات إلى مجلد image: ",
                    "[Image] Copiando archivos a la carpeta image: ",
                )
            ));
        }
    }

    if let Some(rest) = text.strip_prefix("[Image] ") {
        if let Some(package) = rest.strip_suffix(" lk, dtbo 파일을 다운로드 합니다.") {
            return Some(format!(
                "{}{package} lk/dtbo",
                lpm_lang_text(
                    lang,
                    "[Image] Downloading model-specific files: ",
                    "[Image] Загрузка файлов для модели: ",
                    "[Image] モデル専用ファイルをダウンロード：",
                    "[Image] 下載型號專用檔案：",
                    "[Image] Đang tải tệp dành cho model: ",
                    "[Image] Λήψη αρχείων για το μοντέλο: ",
                    "[Image] मॉडल-विशिष्ट फ़ाइलें डाउनलोड हो रही हैं: ",
                    "[Image] მოდელის ფაილების ჩამოტვირთვა: ",
                    "[Image] Modelspecifieke bestanden downloaden: ",
                    "[Image] تنزيل ملفات خاصة بالطراز: ",
                    "[Image] Descargando archivos específicos del modelo: ",
                )
            ));
        }
    }

    if let Some(rest) = text.strip_prefix("[Plan] 작업 scatter XML 재파싱: 성공 / root: ") {
        let rest = rest.replace(
            "개",
            lpm_lang_text(
                lang,
                " items",
                " шт.",
                " 件",
                " 個",
                " mục",
                " στοιχεία",
                " आइटम",
                " ერთეული",
                " items",
                " عناصر",
                " elementos",
            ),
        );
        return Some(format!(
            "{}{rest}",
            lpm_lang_text(
                lang,
                "[Plan] Working scatter XML parsed again successfully / root: ",
                "[Plan] Рабочий scatter XML успешно повторно разобран / root: ",
                "[Plan] 作業用scatter XMLの再解析に成功 / root：",
                "[Plan] 工作用 scatter XML 重新解析成功 / root：",
                "[Plan] Đã phân tích lại scatter XML làm việc thành công / root: ",
                "[Plan] Επιτυχής επανεπεξεργασία του scatter XML εργασίας / root: ",
                "[Plan] कार्य scatter XML दोबारा सफलतापूर्वक पार्स हुआ / root: ",
                "[Plan] სამუშაო scatter XML წარმატებით ხელახლა დამუშავდა / root: ",
                "[Plan] Werk-scatter XML opnieuw geparseerd / root: ",
                "[Plan] تمت إعادة تحليل scatter XML للعمل بنجاح / root: ",
                "[Plan] Scatter XML de trabajo analizado de nuevo correctamente / root: ",
            )
        ));
    }

    if let Some(rest) = text.strip_prefix("[Log] ") {
        if let Some((flow, remainder)) = rest.split_once(" 작업 로그를 ") {
            if let Some(path) = remainder.strip_suffix("에 저장합니다.") {
                return Some(format!(
                    "{}{flow}: {path}",
                    lpm_lang_text(
                        lang,
                        "[Log] Saving task log for ",
                        "[Log] Сохранение журнала задачи ",
                        "[Log] 作業ログを保存：",
                        "[Log] 儲存工作日誌：",
                        "[Log] Lưu nhật ký tác vụ ",
                        "[Log] Αποθήκευση αρχείου καταγραφής εργασίας ",
                        "[Log] कार्य लॉग सहेजा जा रहा है ",
                        "[Log] სამუშაო ჟურნალის შენახვა ",
                        "[Log] Taaklog opslaan voor ",
                        "[Log] حفظ سجل المهمة ",
                        "[Log] Guardando el registro de la tarea ",
                    )
                ));
            }
        }
    }

    if let Some(rest) = text.strip_prefix("[Log] 텍스트 파일 저장 완료: ") {
        return Some(format!(
            "{}{rest}",
            lpm_lang_text(
                lang,
                "[Log] Text file saved: ",
                "[Log] Текстовый файл сохранён: ",
                "[Log] テキストファイルを保存しました：",
                "[Log] 文字檔已儲存：",
                "[Log] Đã lưu tệp văn bản: ",
                "[Log] Το αρχείο κειμένου αποθηκεύτηκε: ",
                "[Log] टेक्स्ट फ़ाइल सहेजी गई: ",
                "[Log] ტექსტური ფაილი შენახულია: ",
                "[Log] Tekstbestand opgeslagen: ",
                "[Log] تم حفظ الملف النصي: ",
                "[Log] Archivo de texto guardado: ",
            )
        ));
    }

    if let Some(rest) = text.strip_prefix("[Image] 기기에 ") {
        if let Some(region) = rest.strip_suffix("을 설치합니다.") {
            return Some(format!(
                "{}{region}",
                lpm_lang_text(
                    lang,
                    "[Image] ROM to install on the device: ",
                    "[Image] ROM для установки на устройство: ",
                    "[Image] デバイスにインストールするROM：",
                    "[Image] 將安裝至裝置的 ROM：",
                    "[Image] ROM sẽ cài trên thiết bị: ",
                    "[Image] ROM που θα εγκατασταθεί στη συσκευή: ",
                    "[Image] डिवाइस पर इंस्टॉल होने वाला ROM: ",
                    "[Image] მოწყობილობაზე დასაყენებელი ROM: ",
                    "[Image] ROM die op het apparaat wordt geïnstalleerd: ",
                    "[Image] الروم الذي سيتم تثبيته على الجهاز: ",
                    "[Image] ROM que se instalará en el dispositivo: ",
                )
            ));
        }
    }

    let prefixes = [
        ("[설정] 초기 언어 설정: ", lpm_lang_text(lang, "[Settings] Initial language: ", "[Настройки] Начальный язык: ", "[設定] 初期言語：", "[設定] 初始語言：", "[Cài đặt] Ngôn ngữ ban đầu: ", "[Ρυθμίσεις] Αρχική γλώσσα: ", "[सेटिंग्स] प्रारंभिक भाषा: ", "[პარამეტრები] საწყისი ენა: ", "[Instellingen] Begintaal: ", "[الإعدادات] اللغة الأولية: ", "[Ajustes] Idioma inicial: ")),
        ("[설정] 언어 설정 파일 경로: ", lpm_lang_text(lang, "[Settings] Language settings file: ", "[Настройки] Файл настроек языка: ", "[設定] 言語設定ファイル：", "[設定] 語言設定檔：", "[Cài đặt] Tệp cài đặt ngôn ngữ: ", "[Ρυθμίσεις] Αρχείο ρυθμίσεων γλώσσας: ", "[सेटिंग्स] भाषा सेटिंग फ़ाइल: ", "[პარამეტრები] ენის პარამეტრების ფაილი: ", "[Instellingen] Taalinstellingenbestand: ", "[الإعدادات] ملف إعدادات اللغة: ", "[Ajustes] Archivo de configuración de idioma: ")),
        ("[설정] 프로그램 언어를 선택했습니다: ", lpm_lang_text(lang, "[Settings] Program language selected: ", "[Настройки] Выбран язык программы: ", "[設定] プログラム言語を選択しました：", "[設定] 已選擇程式語言：", "[Cài đặt] Đã chọn ngôn ngữ chương trình: ", "[Ρυθμίσεις] Επιλέχθηκε γλώσσα προγράμματος: ", "[सेटिंग्स] प्रोग्राम भाषा चुनी गई: ", "[პარამეტრები] არჩეული პროგრამის ენა: ", "[Instellingen] Programmataal geselecteerd: ", "[الإعدادات] تم اختيار لغة البرنامج: ", "[Ajustes] Idioma del programa seleccionado: ")),
        ("[설정] 언어 설정 파일에 저장했습니다: ", lpm_lang_text(lang, "[Settings] Saved to the language settings file: ", "[Настройки] Сохранено в файл настроек языка: ", "[設定] 言語設定ファイルに保存しました：", "[設定] 已儲存至語言設定檔：", "[Cài đặt] Đã lưu vào tệp cài đặt ngôn ngữ: ", "[Ρυθμίσεις] Αποθηκεύτηκε στο αρχείο ρυθμίσεων γλώσσας: ", "[सेटिंग्स] भाषा सेटिंग फ़ाइल में सहेजा गया: ", "[პარამეტრები] შენახულია ენის პარამეტრების ფაილში: ", "[Instellingen] Opgeslagen in het taalinstellingenbestand: ", "[الإعدادات] تم الحفظ في ملف إعدادات اللغة: ", "[Ajustes] Guardado en el archivo de configuración de idioma: ")),
        ("선택한 image 폴더: ", lpm_lang_text(lang, "Selected image folder: ", "Выбранная папка image: ", "選択したimageフォルダー：", "已選擇的 image 資料夾：", "Thư mục image đã chọn: ", "Επιλεγμένος φάκελος image: ", "चयनित image फ़ोल्डर: ", "არჩეული image საქაღალდე: ", "Geselecteerde image-map: ", "مجلد image المحدد: ", "Carpeta image seleccionada: ")),
        ("[Image] 모델명: ", lpm_lang_text(lang, "[Image] Model: ", "[Image] Модель: ", "[Image] モデル：", "[Image] 型號：", "[Image] Model: ", "[Image] Μοντέλο: ", "[Image] मॉडल: ", "[Image] მოდელი: ", "[Image] Model: ", "[Image] الطراز: ", "[Image] Modelo: ")),
        ("[Image] 버전: ", lpm_lang_text(lang, "[Image] Version: ", "[Image] Версия: ", "[Image] バージョン：", "[Image] 版本：", "[Image] Phiên bản: ", "[Image] Έκδοση: ", "[Image] संस्करण: ", "[Image] ვერსია: ", "[Image] Versie: ", "[Image] الإصدار: ", "[Image] Versión: ")),
        ("[Image] ROM 타입: ", lpm_lang_text(lang, "[Image] ROM type: ", "[Image] Тип ROM: ", "[Image] ROMタイプ：", "[Image] ROM 類型：", "[Image] Loại ROM: ", "[Image] Τύπος ROM: ", "[Image] ROM प्रकार: ", "[Image] ROM ტიპი: ", "[Image] ROM-type: ", "[Image] نوع ROM: ", "[Image] Tipo de ROM: ")),
        ("[Image] 플랫폼: ", lpm_lang_text(lang, "[Image] Platform: ", "[Image] Платформа: ", "[Image] プラットフォーム：", "[Image] 平台：", "[Image] Nền tảng: ", "[Image] Πλατφόρμα: ", "[Image] प्लेटफ़ॉर्म: ", "[Image] პლატფორმა: ", "[Image] Platform: ", "[Image] المنصة: ", "[Image] Plataforma: ")),
        ("[ADB] Android 버전: ", lpm_lang_text(lang, "[ADB] Android version: ", "[ADB] Версия Android: ", "[ADB] Androidバージョン：", "[ADB] Android 版本：", "[ADB] Phiên bản Android: ", "[ADB] Έκδοση Android: ", "[ADB] Android संस्करण: ", "[ADB] Android ვერსია: ", "[ADB] Android-versie: ", "[ADB] إصدار Android: ", "[ADB] Versión de Android: ")),
        ("[ADB] 플랫폼: ", lpm_lang_text(lang, "[ADB] Platform: ", "[ADB] Платформа: ", "[ADB] プラットフォーム：", "[ADB] 平台：", "[ADB] Nền tảng: ", "[ADB] Πλατφόρμα: ", "[ADB] प्लेटफ़ॉर्म: ", "[ADB] პლატფორმა: ", "[ADB] Platform: ", "[ADB] المنصة: ", "[ADB] Plataforma: ")),
        ("[ADB] 모델: ", lpm_lang_text(lang, "[ADB] Model: ", "[ADB] Модель: ", "[ADB] モデル：", "[ADB] 型號：", "[ADB] Model: ", "[ADB] Μοντέλο: ", "[ADB] मॉडल: ", "[ADB] მოდელი: ", "[ADB] Model: ", "[ADB] الطراز: ", "[ADB] Modelo: ")),
        ("[ADB] 지역: ", lpm_lang_text(lang, "[ADB] Region: ", "[ADB] Регион: ", "[ADB] リージョン：", "[ADB] 區域：", "[ADB] Khu vực: ", "[ADB] Περιοχή: ", "[ADB] क्षेत्र: ", "[ADB] რეგიონი: ", "[ADB] Regio: ", "[ADB] المنطقة: ", "[ADB] Región: ")),
        ("[Fastboot] current-slot: ", "[Fastboot] current-slot: "),
        ("[Fastboot] current-slot 재확인: ", lpm_lang_text(lang, "[Fastboot] current-slot recheck: ", "[Fastboot] Повторная проверка current-slot: ", "[Fastboot] current-slot再確認：", "[Fastboot] 再次檢查 current-slot：", "[Fastboot] Kiểm tra lại current-slot: ", "[Fastboot] Επανέλεγχος current-slot: ", "[Fastboot] current-slot पुनः जाँच: ", "[Fastboot] current-slot-ის ხელახალი შემოწმება: ", "[Fastboot] current-slot opnieuw controleren: ", "[Fastboot] إعادة التحقق من current-slot: ", "[Fastboot] Nueva comprobación de current-slot: ")),
        ("[Plan] patch 변경 수: ", lpm_lang_text(lang, "[Plan] Patch changes: ", "[Plan] Изменений patch: ", "[Plan] patch変更数：", "[Plan] patch 變更數：", "[Plan] Số thay đổi patch: ", "[Plan] Αλλαγές patch: ", "[Plan] patch बदलाव: ", "[Plan] patch ცვლილებები: ", "[Plan] Patchwijzigingen: ", "[Plan] تغييرات patch: ", "[Plan] Cambios de patch: ")),
        ("[Plan] 데이터 유지 여부: ", lpm_lang_text(lang, "[Plan] Keep data: ", "[Plan] Сохранение данных: ", "[Plan] データ保持：", "[Plan] 保留資料：", "[Plan] Giữ dữ liệu: ", "[Plan] Διατήρηση δεδομένων: ", "[Plan] डेटा रखें: ", "[Plan] მონაცემების შენარჩუნება: ", "[Plan] Gegevens behouden: ", "[Plan] الاحتفاظ بالبيانات: ", "[Plan] Conservar datos: ")),
        ("[Plan] current slot stage 필요 여부: ", lpm_lang_text(lang, "[Plan] Current-slot stage required: ", "[Plan] Требуется этап current slot: ", "[Plan] current slot手順の要否：", "[Plan] 是否需要 current slot 步驟：", "[Plan] Cần bước current slot: ", "[Plan] Απαιτείται στάδιο current slot: ", "[Plan] current slot चरण आवश्यक: ", "[Plan] საჭიროა current slot ეტაპი: ", "[Plan] Current-slotstap vereist: ", "[Plan] مرحلة current slot مطلوبة: ", "[Plan] Etapa current slot necesaria: ")),
        ("[Plan] 기기 ADB 단계 필요 여부: ", lpm_lang_text(lang, "[Plan] Device ADB stage required: ", "[Plan] Требуется этап ADB устройства: ", "[Plan] デバイスADB手順の要否：", "[Plan] 是否需要裝置 ADB 步驟：", "[Plan] Cần bước ADB thiết bị: ", "[Plan] Απαιτείται στάδιο ADB συσκευής: ", "[Plan] डिवाइस ADB चरण आवश्यक: ", "[Plan] საჭიროა მოწყობილობის ADB ეტაპი: ", "[Plan] ADB-stap voor apparaat vereist: ", "[Plan] مرحلة ADB للجهاز مطلوبة: ", "[Plan] Etapa ADB del dispositivo necesaria: ")),
        ("[Plan] proinfo 패치 필요 여부: ", lpm_lang_text(lang, "[Plan] Proinfo patch required: ", "[Plan] Требуется patch proinfo: ", "[Plan] proinfo patchの要否：", "[Plan] 是否需要 proinfo patch：", "[Plan] Cần patch proinfo: ", "[Plan] Απαιτείται patch proinfo: ", "[Plan] proinfo patch आवश्यक: ", "[Plan] საჭიროა proinfo patch: ", "[Plan] Proinfo-patch vereist: ", "[Plan] patch proinfo مطلوب: ", "[Plan] Patch de proinfo necesario: ")),
        ("[ADB] platform: ", lpm_lang_text(lang, "[ADB] Platform: ", "[ADB] Платформа: ", "[ADB] プラットフォーム：", "[ADB] 平台：", "[ADB] Nền tảng: ", "[ADB] Πλατφόρμα: ", "[ADB] प्लेटफ़ॉर्म: ", "[ADB] პლატფორმა: ", "[ADB] Platform: ", "[ADB] المنصة: ", "[ADB] Plataforma: ")),
        ("[ADB] model: ", lpm_lang_text(lang, "[ADB] Model: ", "[ADB] Модель: ", "[ADB] モデル：", "[ADB] 型號：", "[ADB] Model: ", "[ADB] Μοντέλο: ", "[ADB] मॉडल: ", "[ADB] მოდელი: ", "[ADB] Model: ", "[ADB] الطراز: ", "[ADB] Modelo: ")),
        ("[ADB] region: ", lpm_lang_text(lang, "[ADB] Region: ", "[ADB] Регион: ", "[ADB] リージョン：", "[ADB] 區域：", "[ADB] Khu vực: ", "[ADB] Περιοχή: ", "[ADB] क्षेत्र: ", "[ADB] რეგიონი: ", "[ADB] Regio: ", "[ADB] المنطقة: ", "[ADB] Región: ")),
        ("[Plan] 작업 scatter XML 재파싱: 성공 / root: ", lpm_lang_text(lang, "[Plan] Working scatter XML parsed again successfully / root: ", "[Plan] Рабочий scatter XML успешно повторно разобран / root: ", "[Plan] 作業用scatter XMLの再解析に成功 / root：", "[Plan] 工作用 scatter XML 重新解析成功 / root：", "[Plan] Đã phân tích lại scatter XML làm việc thành công / root: ", "[Plan] Επιτυχής επανεπεξεργασία του scatter XML εργασίας / root: ", "[Plan] कार्य scatter XML दोबारा सफलतापूर्वक पार्स हुआ / root: ", "[Plan] სამუშაო scatter XML წარმატებით ხელახლა დამუშავდა / root: ", "[Plan] Werk-scatter XML opnieuw geparseerd / root: ", "[Plan] تمت إعادة تحليل scatter XML للعمل بنجاح / root: ", "[Plan] Scatter XML de trabajo analizado de nuevo correctamente / root: ")),
        ("[Image] image 폴더에 ", lpm_lang_text(lang, "[Image] Copying to the image folder: ", "[Image] Копирование в папку image: ", "[Image] imageフォルダーへコピー：", "[Image] 複製至 image 資料夾：", "[Image] Sao chép vào thư mục image: ", "[Image] Αντιγραφή στον φάκελο image: ", "[Image] image फ़ोल्डर में कॉपी: ", "[Image] image საქაღალდეში კოპირება: ", "[Image] Kopiëren naar de image-map: ", "[Image] النسخ إلى مجلد image: ", "[Image] Copiando a la carpeta image: ")),
        ("[SPFT] DA 설정", lpm_lang_text(lang, "[SPFT] DA setup", "[SPFT] Настройка DA", "[SPFT] DA設定", "[SPFT] DA 設定", "[SPFT] Thiết lập DA", "[SPFT] Ρύθμιση DA", "[SPFT] DA सेटअप", "[SPFT] DA დაყენება", "[SPFT] DA-instelling", "[SPFT] إعداد DA", "[SPFT] Configuración de DA")),
    ];

    for (prefix, translated_prefix) in prefixes {
        if let Some(rest) = text.strip_prefix(prefix) {
            return Some(format!("{translated_prefix}{rest}"));
        }
    }

    None
}

fn lpm_replace_numeric_korean_count_suffix(text: String, suffix: &str) -> String {
    let mut output = String::with_capacity(text.len() + suffix.len());
    let mut previous_was_digit = false;

    for ch in text.chars() {
        if ch == '개' && previous_was_digit {
            output.push_str(suffix);
            previous_was_digit = false;
            continue;
        }

        output.push(ch);
        previous_was_digit = ch.is_ascii_digit();
    }

    output
}

fn lpm_localize_runtime_fragments(lang: LanguageOption, content: String) -> String {
    if lang.is_korean() || !contains_hangul(&content) {
        return content;
    }

    let count_suffix = lpm_lang_text(
        lang,
        " items",
        " шт.",
        " 件",
        " 個",
        " mục",
        " στοιχεία",
        " आइटम",
        " ერთეული",
        " items",
        " عناصر",
        " elementos",
    );
    let mut out = lpm_replace_numeric_korean_count_suffix(content, count_suffix);
    let pairs = [
        ("일반 설치 [데이터 초기화]", lpm_lang_text(lang, "Standard installation [factory reset]", "Обычная установка [сброс данных]", "通常インストール［データ初期化］", "一般安裝［清除資料］", "Cài đặt thông thường [xóa dữ liệu]", "Τυπική εγκατάσταση [επαναφορά δεδομένων]", "सामान्य इंस्टॉल [डेटा रीसेट]", "ჩვეულებრივი დაყენება [მონაცემების წაშლა]", "Standaardinstallatie [fabrieksreset]", "تثبيت عادي [إعادة ضبط البيانات]", "Instalación normal [restablecer datos]")),
        ("ROW 업데이트 [데이터 유지]", lpm_lang_text(lang, "ROW update [keep data]", "Обновление ROW [сохранение данных]", "ROW更新［データ保持］", "ROW 更新［保留資料］", "Cập nhật ROW [giữ dữ liệu]", "Ενημέρωση ROW [διατήρηση δεδομένων]", "ROW अपडेट [डेटा रखें]", "ROW განახლება [მონაცემების შენარჩუნება]", "ROW-update [gegevens behouden]", "تحديث ROW [الاحتفاظ بالبيانات]", "Actualización ROW [conservar datos]")),
        ("재설치 [데이터 초기화]", lpm_lang_text(lang, "Recovery [factory reset]", "Восстановление [сброс данных]", "復旧［データ初期化］", "修復［清除資料］", "Khôi phục [xóa dữ liệu]", "Ανάκτηση [επαναφορά δεδομένων]", "रिकवरी [डेटा रीसेट]", "აღდგენა [მონაცემების წაშლა]", "Herstel [fabrieksreset]", "استرداد [إعادة ضبط البيانات]", "Recuperación [restablecer datos]")),
        ("국가 코드 재설정 [proinfo only]", lpm_lang_text(lang, "Country-code reset [proinfo only]", "Сброс кода страны [только proinfo]", "国コード再設定［proinfoのみ］", "重設國家代碼［僅 proinfo］", "Đặt lại mã quốc gia [chỉ proinfo]", "Επαναφορά κωδικού χώρας [μόνο proinfo]", "देश कोड रीसेट [केवल proinfo]", "ქვეყნის კოდის აღდგენა [მხოლოდ proinfo]", "Landcode opnieuw instellen [alleen proinfo]", "إعادة تعيين رمز البلد [proinfo فقط]", "Restablecer código de país [solo proinfo]")),
        ("필수 partition 상세", lpm_lang_text(lang, "Required partition details", "Сведения об обязательных partition", "必須partitionの詳細", "必要 partition 詳細", "Chi tiết partition bắt buộc", "Λεπτομέρειες υποχρεωτικών partition", "आवश्यक partition विवरण", "აუცილებელი partition-ის დეტალები", "Details van vereiste partition", "تفاصيل partition المطلوبة", "Detalles de partition obligatorias")),
        ("patch plan 생성", lpm_lang_text(lang, "Patch plan creation", "Создание patch plan", "patch plan作成", "建立 patch plan", "Tạo patch plan", "Δημιουργία patch plan", "patch plan बनाना", "patch plan-ის შექმნა", "Patch plan maken", "إنشاء patch plan", "Creación del patch plan")),
        ("patch plan 적용", lpm_lang_text(lang, "Patch plan application", "Применение patch plan", "patch plan適用", "套用 patch plan", "Áp dụng patch plan", "Εφαρμογή patch plan", "patch plan लागू करना", "patch plan-ის გამოყენება", "Patch plan toepassen", "تطبيق patch plan", "Aplicación del patch plan")),
        ("patch 결과 재검증", lpm_lang_text(lang, "Patch result recheck", "Повторная проверка результата patch", "patch結果の再確認", "重新檢查 patch 結果", "Kiểm tra lại kết quả patch", "Επανέλεγχος αποτελέσματος patch", "patch परिणाम पुनः जाँच", "patch შედეგის ხელახალი შემოწმება", "Patchresultaat opnieuw controleren", "إعادة التحقق من نتيجة patch", "Nueva comprobación del resultado del patch")),
        ("사용 가능", lpm_lang_text(lang, "available", "доступно", "使用可能", "可使用", "khả dụng", "διαθέσιμο", "उपलब्ध", "ხელმისაწვდომია", "beschikbaar", "متاح", "disponible")),
        ("사용 불가", lpm_lang_text(lang, "unavailable", "недоступно", "使用不可", "不可使用", "không khả dụng", "μη διαθέσιμο", "अनुपलब्ध", "მიუწვდომელია", "niet beschikbaar", "غير متاح", "no disponible")),
        ("적용됨", lpm_lang_text(lang, "applied", "применено", "適用済み", "已套用", "đã áp dụng", "εφαρμόστηκε", "लागू", "გამოყენებულია", "toegepast", "تم التطبيق", "aplicado")),
        ("미적용", lpm_lang_text(lang, "not applied", "не применено", "未適用", "未套用", "chưa áp dụng", "δεν εφαρμόστηκε", "लागू नहीं", "არ არის გამოყენებული", "niet toegepast", "لم يتم التطبيق", "no aplicado")),
        ("모두 통과", lpm_lang_text(lang, "all passed", "все проверки пройдены", "すべて合格", "全部通過", "tất cả đều đạt", "όλα πέρασαν", "सभी पास", "ყველა შემოწმება გავლილია", "alles geslaagd", "اجتازت جميعها", "todas superadas")),
        ("통과", lpm_lang_text(lang, "passed", "пройдено", "合格", "通過", "đạt", "πέρασε", "पास", "გავლილია", "geslaagd", "تم الاجتياز", "superado")),
        ("실패", lpm_lang_text(lang, "failed", "ошибка", "失敗", "失敗", "thất bại", "αποτυχία", "विफल", "ვერ შესრულდა", "mislukt", "فشل", "falló")),
        ("오류", lpm_lang_text(lang, "errors", "ошибки", "エラー", "錯誤", "lỗi", "σφάλματα", "त्रुटियाँ", "შეცდომები", "fouten", "أخطاء", "errores")),
        ("경고", lpm_lang_text(lang, "warnings", "предупреждения", "警告", "警告", "cảnh báo", "προειδοποιήσεις", "चेतावनियाँ", "გაფრთხილებები", "waarschuwingen", "تحذيرات", "advertencias")),
        ("변경", lpm_lang_text(lang, "changes", "изменений", "変更", "變更", "thay đổi", "αλλαγές", "बदलाव", "ცვლილებები", "wijzigingen", "تغييرات", "cambios")),
        ("있음", lpm_lang_text(lang, "yes", "да", "あり", "有", "có", "ναι", "हाँ", "კი", "ja", "نعم", "sí")),
        ("없음", lpm_lang_text(lang, "no", "нет", "なし", "無", "không", "όχι", "नहीं", "არა", "nee", "لا", "no")),
        ("누락", lpm_lang_text(lang, "missing", "отсутствует", "不足", "缺少", "thiếu", "λείπει", "अनुपलब्ध", "აკლია", "ontbreekt", "مفقود", "falta")),
        ("초기화", lpm_lang_text(lang, "factory reset", "сброс", "初期化", "清除資料", "xóa dữ liệu", "επαναφορά", "रीसेट", "წაშლა", "fabrieksreset", "إعادة ضبط", "restablecer")),
        ("유지", lpm_lang_text(lang, "keep", "сохранить", "保持", "保留", "giữ", "διατήρηση", "रखें", "შენარჩუნება", "behouden", "الاحتفاظ", "conservar")),
        ("완료", lpm_lang_text(lang, "completed", "завершено", "完了", "完成", "hoàn tất", "ολοκληρώθηκε", "पूर्ण", "დასრულდა", "voltooid", "اكتمل", "completado")),
        ("성공", lpm_lang_text(lang, "success", "успешно", "成功", "成功", "thành công", "επιτυχία", "सफल", "წარმატებით", "geslaagd", "نجاح", "correcto")),
    ];

    for (from, to) in pairs {
        out = out.replace(from, to);
    }

    out
}

fn lpm_translate_owned(content: String) -> String {
    let lang = active_language_option();

    if lang.is_korean() {
        return content;
    }

    let (content, spinner_frame) = detach_spinner_frame_owned(content);

    let translated = if let Some(translated) =
        lpm_translate_known_runtime_log(lang, content.as_str())
    {
        translated
    } else if let Some(translated) =
        lpm_translate_v312_runtime_message(lang, content.as_str())
    {
        translated
    } else if content.contains("실제 값과 다를 수")
        || content.contains("実際 値")
        || content.contains("實際 值")
        || content.contains("実際の値と異なる")
        || content.contains("可能與實際值不同")
    {
        lpm_format_widevine_value(lang, content.as_str())
    } else if let Some(translated) = lpm_translate_duration_text(lang, content.as_str()) {
        translated
    } else if let Some(translated) = lpm_translate_en_ru_dynamic(lang, content.as_str()) {
        translated
    } else if let Some(translated) = lpm_translate_en_ru_exact(lang, content.as_str()) {
        translated.to_string()
    } else if let Some(translated) = lpm_translate_dynamic_stage13(lang, content.as_str()) {
        lpm_translate_en_ru_cleanup(lang, translated)
    } else if let Some(translated) = lpm_translate_dynamic_stage11(lang, content.as_str()) {
        lpm_translate_stage13_cleanup(lang, translated)
    } else if let Some(translated) = lpm_translate_exact_stage13(lang, content.as_str()) {
        translated.to_string()
    } else if let Some(translated) = lpm_translate_exact_stage10(lang, content.as_str()) {
        translated.to_string()
    } else if let Some(translated) = lpm_translate_exact_stage8(lang, content.as_str()) {
        translated.to_string()
    } else if let Some(translated) = lpm_translate_exact_stage7(lang, content.as_str()) {
        translated.to_string()
    } else if let Some(translated) = lpm_translate_exact_stage6(lang, content.as_str()) {
        translated.to_string()
    } else if let Some(translated) = lpm_translate_exact_stage5(lang, content.as_str()) {
        translated.to_string()
    } else if let Some(translated) = lpm_translate_exact_stage4(lang, content.as_str()) {
        translated.to_string()
    } else if let Some(translated) = lpm_translate_exact_stage3(lang, content.as_str()) {
        translated.to_string()
    } else if let Some(translated) = lpm_translate_exact(lang, content.as_str()) {
        translated.to_string()
    } else {
        let translated = lpm_translate_stage8_phrasewise(lang, content.clone());
        let translated = lpm_translate_stage7_phrasewise(lang, translated);
        let translated = lpm_translate_phrasewise(lang, translated);
        let translated = lpm_translate_stage3_phrasewise(lang, translated);
        let translated = lpm_translate_stage4_phrasewise(lang, translated);
        let translated = lpm_translate_stage5_phrasewise(lang, translated);
        let translated = lpm_translate_stage6_phrasewise(lang, translated);
        let translated = lpm_translate_stage8_phrasewise(lang, translated);
        let translated = lpm_translate_stage10_cleanup(lang, translated);
        let translated = lpm_translate_stage11_final_cleanup(lang, translated);
        let translated = lpm_translate_stage13_cleanup(lang, translated);
        lpm_translate_en_ru_cleanup(lang, translated)
    };

    let translated = lpm_localize_runtime_fragments(lang, translated);
    let localized = enforce_selected_language_output(lang, &content, translated);

    if let Some(frame) = spinner_frame {
        format!("{} {}", localized.trim_end(), frame)
    } else {
        localized
    }
}

fn enforce_selected_language_output(
    lang: LanguageOption,
    original: &str,
    translated: String,
) -> String {
    if !contains_hangul(&translated) {
        return translated;
    }

    let korean_language_name = lpm_lang_text(
        lang,
        "Korean (ko)",
        "Корейский (ko)",
        "韓国語 (ko)",
        "韓文 (ko)",
        "Tiếng Hàn (ko)",
        "Κορεατικά (ko)",
        "कोरियाई (ko)",
        "კორეული (ko)",
        "Koreaans (ko)",
        "الكورية (ko)",
        "Coreano (ko)",
    );

    let mut localized = translated.replace("한국어 (ko)", korean_language_name);

    for _ in 0..2 {
        if !contains_hangul(&localized) {
            return localized;
        }

        localized = lpm_translate_stage8_phrasewise(lang, localized);
        localized = lpm_translate_stage7_phrasewise(lang, localized);
        localized = lpm_translate_phrasewise(lang, localized);
        localized = lpm_translate_stage3_phrasewise(lang, localized);
        localized = lpm_translate_stage4_phrasewise(lang, localized);
        localized = lpm_translate_stage5_phrasewise(lang, localized);
        localized = lpm_translate_stage6_phrasewise(lang, localized);
        localized = lpm_translate_stage10_cleanup(lang, localized);
        localized = lpm_translate_stage11_final_cleanup(lang, localized);
        localized = lpm_translate_stage13_cleanup(lang, localized);
    }

    if !contains_hangul(&localized) {
        return localized;
    }

    let sanitized = remove_remaining_hangul(&localized);
    if !sanitized.trim().is_empty() {
        return sanitized;
    }

    let technical = remove_remaining_hangul(original);
    if !technical.trim().is_empty() {
        return technical;
    }

    lpm_lang_text(
        lang,
        "Operation status updated.",
        "Состояние операции обновлено.",
        "処理状態を更新しました。",
        "作業狀態已更新。",
        "Trạng thái tác vụ đã được cập nhật.",
        "Η κατάσταση της εργασίας ενημερώθηκε.",
        "कार्य की स्थिति अपडेट की गई।",
        "ოპერაციის სტატუსი განახლდა.",
        "De taakstatus is bijgewerkt.",
        "تم تحديث حالة العملية.",
        "Se actualizó el estado de la operación.",
    )
    .to_string()
}

fn remove_remaining_hangul(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut previous_space = false;

    for ch in text.chars() {
        let is_hangul = matches!(
            ch,
            '\u{1100}'..='\u{11ff}'
                | '\u{3130}'..='\u{318f}'
                | '\u{a960}'..='\u{a97f}'
                | '\u{ac00}'..='\u{d7af}'
                | '\u{d7b0}'..='\u{d7ff}'
        );

        if is_hangul {
            if !previous_space && !output.is_empty() {
                output.push(' ');
            }
            previous_space = true;
            continue;
        }

        if ch == '\n' || ch == '\r' {
            while output.ends_with(' ') {
                output.pop();
            }
            output.push(ch);
            previous_space = false;
            continue;
        }

        if ch.is_whitespace() {
            if !previous_space && !output.is_empty() {
                output.push(' ');
            }
            previous_space = true;
        } else {
            output.push(ch);
            previous_space = false;
        }
    }

    output
        .lines()
        .map(|line| {
            line.trim()
                .trim_matches(|ch: char| matches!(ch, ':' | '/' | ',' | '-' | '|'))
                .trim()
                .to_string()
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn contains_hangul(text: &str) -> bool {
    text.chars().any(|ch| matches!(ch, '\u{1100}'..='\u{11ff}' | '\u{3130}'..='\u{318f}' | '\u{a960}'..='\u{a97f}' | '\u{ac00}'..='\u{d7af}' | '\u{d7b0}'..='\u{d7ff}'))
}

fn lpm_translate_exact_stage10(lang: LanguageOption, key: &str) -> Option<&'static str> {
    Some(match key {
        "대시 보드" | "대시보드" => lpm_lang_text(lang, "Dashboard", "Панель", "ダッシュボード", "儀表板", "Bảng điều khiển", "Πίνακας", "डैशबोर्ड", "დაფა", "Dashboard", "لوحة التحكم", "Panel"),
        "ROM 작업" => lpm_lang_text(lang, "ROM Tasks", "Операции ROM", "ROM作業", "ROM 工作", "Tác vụ ROM", "Εργασίες ROM", "ROM कार्य", "ROM ამოცანები", "ROM-taken", "مهام ROM", "Tareas ROM"),
        "추가 옵션" => lpm_lang_text(lang, "Additional Options", "Дополнительные параметры", "追加オプション", "其他選項", "Tùy chọn bổ sung", "Πρόσθετες επιλογές", "अतिरिक्त विकल्प", "დამატებითი პარამეტრები", "Extra opties", "خيارات إضافية", "Opciones adicionales"),
        "펌웨어 다운로드" => lpm_lang_text(lang, "Firmware Download", "Скачать прошивку", "ファームウェアDL", "下載韌體", "Tải firmware", "Λήψη firmware", "फर्मवेयर डाउनलोड", "Firmware ჩამოტვირთვა", "Firmware downloaden", "تنزيل firmware", "Descargar firmware"),
        "QnA" => "QnA",
        "로그 관리" => lpm_lang_text(lang, "Log Management", "Управление журналом", "ログ管理", "日誌管理", "Quản lý nhật ký", "Διαχείριση καταγραφής", "लॉग प्रबंधन", "ლოგის მართვა", "Logbeheer", "إدارة السجل", "Gestión de registros"),
        "설정" => lpm_lang_text(lang, "Settings", "Настройки", "設定", "設定", "Cài đặt", "Ρυθμίσεις", "सेटिंग्स", "პარამეტრები", "Instellingen", "الإعدادات", "Configuración"),
        "기기 관리" => lpm_lang_text(lang, "Device Management", "Управление устройством", "デバイス管理", "裝置管理", "Quản lý thiết bị", "Διαχείριση συσκευής", "डिवाइस प्रबंधन", "მოწყობილობის მართვა", "Apparaatbeheer", "إدارة الجهاز", "Gestión del dispositivo"),
        "프로그램" => lpm_lang_text(lang, "Program", "Программа", "プログラム", "程式", "Chương trình", "Πρόγραμμα", "प्रोग्राम", "პროგრამა", "Programma", "البرنامج", "Programa"),
        "대기 중" => lpm_lang_text(lang, "Idle", "Ожидание", "待機中", "待機中", "Đang chờ", "Σε αναμονή", "प्रतीक्षा में", "მოლოდინში", "Inactief", "في الانتظار", "En espera"),
        "작업 중" => lpm_lang_text(lang, "Working", "Выполняется", "作業中", "執行中", "Đang xử lý", "Σε εξέλιξη", "कार्य जारी", "მუშაობს", "Bezig", "جارٍ العمل", "Trabajando"),
        "알 수 없음" => lpm_lang_text(lang, "Unknown", "Неизвестно", "不明", "未知", "Không rõ", "Άγνωστο", "अज्ञात", "უცნობია", "Onbekend", "غير معروف", "Desconocido"),
        "감지 전" => lpm_lang_text(lang, "Not detected yet", "Ещё не обнаружено", "未検出", "尚未偵測", "Chưa phát hiện", "Δεν εντοπίστηκε ακόμη", "अभी पता नहीं चला", "ჯერ არ აღმოჩენილა", "Nog niet gedetecteerd", "لم يتم الاكتشاف بعد", "Aún no detectado"),
        "차단 완료" => lpm_lang_text(lang, "Blocked", "Заблокировано", "ブロック済み", "已封鎖", "Đã chặn", "Αποκλεισμένο", "अवरुद्ध", "დაბლოკილია", "Geblokkeerd", "محظور", "Bloqueado"),
        "설정됨" => lpm_lang_text(lang, "Configured", "Настроено", "設定済み", "已設定", "Đã cấu hình", "Ρυθμίστηκε", "सेट किया गया", "დაყენებულია", "Ingesteld", "تم الإعداد", "Configurado"),
        "미설정" => lpm_lang_text(lang, "Not configured", "Не настроено", "未設定", "未設定", "Chưa cấu hình", "Δεν ρυθμίστηκε", "सेट नहीं", "არ არის დაყენებული", "Niet ingesteld", "غير مضبوط", "No configurado"),
        "PRC ↔ ROW 설치" => lpm_lang_text(lang, "PRC ↔ ROW Install", "Установка PRC ↔ ROW", "PRC ↔ ROWインストール", "PRC ↔ ROW 安裝", "Cài đặt PRC ↔ ROW", "Εγκατάσταση PRC ↔ ROW", "PRC ↔ ROW इंस्टॉल", "PRC ↔ ROW ინსტალაცია", "PRC ↔ ROW installeren", "PRC ↔ ROW تثبيت", "Instalar PRC ↔ ROW"),
        "ROW(글로벌롬) 업데이트" => lpm_lang_text(lang, "ROW Update", "Обновление ROW", "ROWアップデート", "ROW 更新", "Cập nhật ROW", "Ενημέρωση ROW", "ROW अपडेट", "ROW განახლება", "ROW-update", "تحديث ROW", "Actualización ROW"),
        "기기 복구" => lpm_lang_text(lang, "Device Recovery", "Восстановление устройства", "デバイス復旧", "裝置修復", "Khôi phục thiết bị", "Ανάκτηση συσκευής", "डिवाइस रिकवरी", "მოწყობილობის აღდგენა", "Apparaatherstel", "استرداد الجهاز", "Recuperación"),
        "데이터 초기화" => lpm_lang_text(lang, "Factory reset", "Сброс данных", "データ初期化", "恢復原廠設定", "Xóa dữ liệu", "Επαναφορά δεδομένων", "डेटा रीसेट", "მონაცემების წაშლა", "Fabrieksreset", "إعادة ضبط البيانات", "Restablecer datos"),
        "데이터 유지" => lpm_lang_text(lang, "Keep data", "Сохранить данные", "データ保持", "保留資料", "Giữ dữ liệu", "Διατήρηση δεδομένων", "डेटा रखें", "მონაცემების შენარჩუნება", "Gegevens behouden", "الاحتفاظ بالبيانات", "Conservar datos"),
        "설치 시작" => lpm_lang_text(lang, "Start Install", "Начать установку", "開始", "開始安裝", "Bắt đầu cài", "Έναρξη εγκατάστασης", "इंस्टॉल शुरू", "დაწყება", "Start installatie", "بدء التثبيت", "Iniciar"),
        "업데이트 시작" => lpm_lang_text(lang, "Start Update", "Начать обновление", "更新開始", "開始更新", "Bắt đầu cập nhật", "Έναρξη ενημέρωσης", "अपडेट शुरू", "განახლება", "Start update", "بدء التحديث", "Actualizar"),
        "복구 시작" => lpm_lang_text(lang, "Start Recovery", "Начать восстановление", "復旧開始", "開始修復", "Bắt đầu khôi phục", "Έναρξη ανάκτησης", "रिकवरी शुरू", "აღდგენა", "Start herstel", "بدء الاسترداد", "Recuperar"),
        "PRC(중국 내수롬) 또는\nROW(글로벌롬)을 설치합니다." => lpm_lang_text(lang, "Install PRC or ROW ROM.", "Установить PRC или ROW ROM.", "PRC/ROW ROMを切り替えます。", "安裝 PRC 或 ROW ROM。", "Cài PRC hoặc ROW ROM.", "Εγκατάσταση PRC ή ROW ROM.", "PRC या ROW ROM इंस्टॉल करें।", "დააყენეთ PRC ან ROW ROM.", "Installeer PRC- of ROW-ROM.", "تثبيت PRC أو ROW ROM.", "Instala ROM PRC o ROW."),
        "ROW(글로벌롬) 버전을\n업데이트 합니다." => lpm_lang_text(lang, "Update the ROW ROM version.", "Обновить версию ROW ROM.", "ROW ROMを更新します。", "更新 ROW ROM 版本。", "Cập nhật phiên bản ROW ROM.", "Ενημέρωση έκδοσης ROW ROM.", "ROW ROM संस्करण अपडेट करें।", "განაახლეთ ROW ROM ვერსია.", "Werk de ROW-ROM bij.", "تحديث إصدار ROW ROM.", "Actualiza la ROM ROW."),
        "설치 실패 / 무한 재부팅 / Red State 복구" => lpm_lang_text(lang, "Fix install failure, boot loop, or Red State", "Исправление сбоя установки, циклической перезагрузки или Red State", "失敗・再起動ループ・Red Stateを修復", "修復安裝失敗、循環重啟或 Red State", "Sửa lỗi cài đặt, lặp khởi động hoặc Red State", "Διόρθωση αποτυχίας εγκατάστασης, boot loop ή Red State", "इंस्टॉल विफलता, boot loop या Red State ठीक करें", "ინსტალაციის შეცდომის, boot loop-ის ან Red State-ის შეკეთება", "Herstel installatiefout, bootloop of Red State", "إصلاح فشل التثبيت أو boot loop أو Red State", "Repara fallo de instalación, bucle de arranque o Red State"),
        "선택한 image 폴더 정보" => lpm_lang_text(lang, "Selected Image Folder", "Выбранная папка image", "選択したimageフォルダー", "已選擇的 image 資料夾", "Thư mục image đã chọn", "Επιλεγμένος φάκελος image", "चयनित image फ़ोल्डर", "არჩეული image საქაღალდე", "Geselecteerde image-map", "مجلد image المحدد", "Carpeta image seleccionada"),
        "image 폴더 정보를 확인합니다." => lpm_lang_text(lang, "Check the selected image folder.", "Проверьте выбранную папку image.", "imageフォルダーを確認します。", "檢查已選擇的 image 資料夾。", "Kiểm tra thư mục image đã chọn.", "Ελέγξτε τον επιλεγμένο φάκελο image.", "चयनित image फ़ोल्डर जांचें।", "შეამოწმეთ არჩეული image საქაღალდე.", "Controleer de geselecteerde image-map.", "تحقق من مجلد image المحدد.", "Comprueba la carpeta image seleccionada."),
        "선택한 image 폴더" => lpm_lang_text(lang, "Selected image folder", "Выбранная папка image", "選択したimageフォルダー", "已選擇的 image 資料夾", "Thư mục image đã chọn", "Επιλεγμένος φάκελος image", "चयनित image फ़ोल्डर", "არჩეული image საქაღალდე", "Geselecteerde image-map", "مجلد image المحدد", "Carpeta image seleccionada"),
        "펌웨어 버전, 플랫폼, 모델명, 필수 partition 유효성, MTK 드라이버 및 Microsoft Visual C++ 런타임 설치 유/무를 검사합니다." => lpm_lang_text(lang, "Check firmware version, platform, model name, required partition validity, MTK driver status, and Microsoft Visual C++ Runtime status.", "Проверка версии прошивки, платформы, модели, обязательных partition, драйвера MTK и среды выполнения Microsoft Visual C++.", "ファームウェアバージョン、プラットフォーム、モデル名、必須partition、MTKドライバー、Microsoft Visual C++ ランタイムの状態を確認します。", "檢查韌體版本、平台、型號、必要 partition、MTK 驅動程式與 Microsoft Visual C++ 執行階段狀態。", "Kiểm tra phiên bản firmware, nền tảng, model, partition bắt buộc, driver MTK và Microsoft Visual C++ Runtime.", "Έλεγχος έκδοσης firmware, πλατφόρμας, μοντέλου, υποχρεωτικών partition, οδηγού MTK και Microsoft Visual C++ Runtime.", "फर्मवेयर संस्करण, प्लेटफ़ॉर्म, मॉडल, आवश्यक partition, MTK ड्राइवर और Microsoft Visual C++ Runtime स्थिति जांचें।", "შემოწმდება firmware ვერსია, პლატფორმა, მოდელი, აუცილებელი partition, MTK დრაივერი და Microsoft Visual C++ Runtime.", "Controleert firmwareversie, platform, modelnaam, vereiste partition, MTK-driver en Microsoft Visual C++ Runtime.", "التحقق من إصدار firmware والمنصة والطراز وpartition المطلوبة وتعريف MTK وMicrosoft Visual C++ Runtime.", "Comprueba la versión de firmware, plataforma, modelo, partition obligatorias, controlador MTK y Microsoft Visual C++ Runtime."),
        "다음 단계로 진행해주세요." | "다음 단계로 진행해주세요" => lpm_lang_text(lang, "Proceed to the next step.", "Перейдите к следующему шагу.", "次のステップへ進んでください。", "請進入下一步。", "Hãy chuyển sang bước tiếp theo.", "Προχωρήστε στο επόμενο βήμα.", "अगले चरण पर जाएँ।", "გადადით შემდეგ ეტაპზე.", "Ga verder naar de volgende stap.", "انتقل إلى الخطوة التالية.", "Continúe con el siguiente paso."),
        "작업 선택" => lpm_lang_text(lang, "Select Task", "Выбор задачи", "作業選択", "選擇工作", "Chọn tác vụ", "Επιλογή εργασίας", "कार्य चुनें", "ამოცანის არჩევა", "Taak kiezen", "اختيار المهمة", "Seleccionar tarea"),
        "아래 작업을 선택해서 기기에 적용합니다." => lpm_lang_text(lang, "Select a task below and apply it to the device.", "Выберите задачу ниже и примените её к устройству.", "下の作業を選択してデバイスに適用します。", "選擇下方工作並套用至裝置。", "Chọn tác vụ bên dưới và áp dụng cho thiết bị.", "Επιλέξτε μια εργασία και εφαρμόστε τη στη συσκευή.", "नीचे कार्य चुनकर डिवाइस पर लागू करें।", "აირჩიეთ ამოცანა და გამოიყენეთ მოწყობილობაზე.", "Kies hieronder een taak en pas deze toe op het apparaat.", "اختر مهمة أدناه وطبقها على الجهاز.", "Seleccione una tarea y aplíquela al dispositivo."),
        "이전 메뉴로 이동" => lpm_lang_text(lang, "Back", "Назад", "前のメニューへ", "返回", "Quay lại", "Πίσω", "वापस", "უკან", "Terug", "رجوع", "Atrás"),
        "ROW(글로벌롬)기기에 PRC(중국 내수롬)을 설치합니다." => lpm_lang_text(lang, "Install PRC ROM on the ROW device.", "Установить PRC ROM на устройство ROW.", "ROW端末にPRC ROMをインストールします。", "在 ROW 裝置上安裝 PRC ROM。", "Cài PRC ROM trên thiết bị ROW.", "Εγκατάσταση PRC ROM στη συσκευή ROW.", "ROW डिवाइस पर PRC ROM इंस्टॉल करें।", "ROW მოწყობილობაზე PRC ROM-ის დაყენება.", "Installeer PRC-ROM op het ROW-apparaat.", "تثبيت PRC ROM على جهاز ROW.", "Instala PRC ROM en el dispositivo ROW."),
        "PRC(중국 내수롬)기기에 ROW(글로벌롬)을 설치합니다." => lpm_lang_text(lang, "Install ROW ROM on the PRC device.", "Установить ROW ROM на устройство PRC.", "PRC端末にROW ROMをインストールします。", "在 PRC 裝置上安裝 ROW ROM。", "Cài ROW ROM trên thiết bị PRC.", "Εγκατάσταση ROW ROM στη συσκευή PRC.", "PRC डिवाइस पर ROW ROM इंस्टॉल करें।", "PRC მოწყობილობაზე ROW ROM-ის დაყენება.", "Installeer ROW-ROM op het PRC-apparaat.", "تثبيت ROW ROM على جهاز PRC.", "Instala ROW ROM en el dispositivo PRC."),
        "ROW(글로벌롬) 업데이트로 진행해 주세요." => lpm_lang_text(lang, "Please use ROW Update.", "Используйте обновление ROW.", "ROWアップデートを選択してください。", "請使用 ROW 更新。", "Hãy dùng cập nhật ROW.", "Χρησιμοποιήστε την ενημέρωση ROW.", "ROW अपडेट का उपयोग करें।", "გამოიყენეთ ROW განახლება.", "Gebruik ROW-update.", "استخدم تحديث ROW.", "Use la actualización ROW."),
        "기기가 연결되어 있지 않아 설치를 실행할 수 없습니다." => lpm_lang_text(lang, "Cannot install because no device is connected.", "Невозможно установить: устройство не подключено.", "デバイスが接続されていないため実行できません。", "未連接裝置，無法安裝。", "Không thể cài vì chưa kết nối thiết bị.", "Δεν είναι δυνατή η εγκατάσταση επειδή δεν υπάρχει συσκευή.", "डिवाइस कनेक्ट नहीं है, इसलिए इंस्टॉल नहीं हो सकता।", "მოწყობილობა დაკავშირებული არ არის.", "Installatie niet mogelijk: geen apparaat verbonden.", "لا يمكن التثبيت لأن الجهاز غير متصل.", "No se puede instalar porque no hay dispositivo conectado."),
        "기기가 켜지지 않거나, 무한 재부팅 등 다양한 오류를 고칩니다." => lpm_lang_text(lang, "Fixes no boot, boot loop, and other errors.", "Исправляет отсутствие запуска, циклическую перезагрузку и другие ошибки.", "起動不可や再起動ループなどを修復します。", "修復無法開機、循環重啟等問題。", "Sửa lỗi không khởi động, lặp khởi động và lỗi khác.", "Διορθώνει μη εκκίνηση, boot loop και άλλα σφάλματα.", "बूट न होना, boot loop और अन्य त्रुटियाँ ठीक करता है।", "ასწორებს არ ჩართვას, boot loop-ს და სხვა შეცდომებს.", "Herstelt niet opstarten, bootloop en andere fouten.", "إصلاح عدم الإقلاع و boot loop وأخطاء أخرى.", "Repara arranque fallido, bucle de arranque y otros errores."),
        "PRC ↔ ROW 설치 루틴" => lpm_lang_text(lang, "PRC ↔ ROW Install", "Установка PRC ↔ ROW", "PRC ↔ ROWインストール", "PRC ↔ ROW 安裝", "Cài đặt PRC ↔ ROW", "Εγκατάσταση PRC ↔ ROW", "PRC ↔ ROW इंस्टॉल", "PRC ↔ ROW ინსტალაცია", "PRC ↔ ROW-installatie", "PRC ↔ ROW تثبيت", "Instalación PRC ↔ ROW"),
        "중국 내수롬과 글로벌롬을 자유롭게 변경 가능" => lpm_lang_text(lang, "Switch between PRC and ROW ROM.", "Переключение между PRC и ROW ROM.", "PRC/ROW ROMを切り替えます。", "切換 PRC 與 ROW ROM。", "Chuyển đổi giữa PRC và ROW ROM.", "Εναλλαγή μεταξύ PRC και ROW ROM.", "PRC और ROW ROM के बीच बदलें।", "PRC და ROW ROM-ის გადართვა.", "Schakel tussen PRC- en ROW-ROM.", "التبديل بين PRC و ROW ROM.", "Cambia entre PRC y ROW ROM."),
        "데이터 초기화가 필수이기 때문에" => lpm_lang_text(lang, "A factory reset is required.", "Требуется сброс данных.", "データ初期化が必要です。", "必須清除資料。", "Bắt buộc xóa dữ liệu.", "Απαιτείται επαναφορά δεδομένων.", "डेटा रीसेट आवश्यक है।", "მონაცემების წაშლა საჭიროა.", "Fabrieksreset is vereist.", "يلزم مسح البيانات.", "Se requiere restablecer datos."),
        "시작하기 전 데이터 백업 후 진행해주세요." => lpm_lang_text(lang, "Back up your data first.", "Сначала сделайте резервную копию данных.", "先にデータをバックアップしてください。", "請先備份資料。", "Hãy sao lưu dữ liệu trước.", "Πρώτα δημιουργήστε αντίγραφο των δεδομένων.", "पहले डेटा का बैकअप लें।", "ჯერ დაარეზერვეთ მონაცემები.", "Maak eerst een back-up van uw gegevens.", "انسخ بياناتك احتياطيًا أولاً.", "Haga una copia de seguridad primero."),
        "ROW(글로벌롬) 업데이트 루틴" => lpm_lang_text(lang, "ROW Update", "Обновление ROW", "ROWアップデート", "ROW 更新", "Cập nhật ROW", "Ενημέρωση ROW", "ROW अपडेट", "ROW განახლება", "ROW-update", "تحديث ROW", "Actualización ROW"),
        "글로벌롬 펌웨어 버전을 업데이트합니다." => lpm_lang_text(lang, "Updates the ROW ROM version.", "Обновляет версию ROW ROM.", "ROW ROMを更新します。", "更新 ROW ROM 版本。", "Cập nhật phiên bản ROW ROM.", "Ενημερώνει την έκδοση ROW ROM.", "ROW ROM संस्करण अपडेट करता है।", "განაახლებს ROW ROM ვერსიას.", "Werkt de ROW-ROM bij.", "يحدّث إصدار ROW ROM.", "Actualiza la versión ROW ROM."),
        "기기에 설치된 버전보다 낮을 경우/초기화 O." => lpm_lang_text(lang, "Lower version: factory reset required.", "Версия ниже: требуется сброс.", "低いバージョンは初期化が必要です。", "版本較低：需要清除資料。", "Phiên bản thấp hơn: cần xóa dữ liệu.", "Χαμηλότερη έκδοση: απαιτείται επαναφορά.", "कम संस्करण: डेटा रीसेट आवश्यक।", "უფრო დაბალი ვერსია: საჭიროა წაშლა.", "Lagere versie: reset vereist.", "إصدار أقدم: يلزم مسح البيانات.", "Versión inferior: requiere restablecer."),
        "기기에 설치된 버전보다 높은 경우/초기화 X." => lpm_lang_text(lang, "Higher version: keep data.", "Версия выше: данные сохраняются.", "高いバージョンはデータ保持。", "版本較高：保留資料。", "Phiên bản cao hơn: giữ dữ liệu.", "Υψηλότερη έκδοση: διατήρηση δεδομένων.", "उच्च संस्करण: डेटा रखें।", "უფრო მაღალი ვერსია: მონაცემები შენარჩუნდება.", "Hogere versie: gegevens behouden.", "إصدار أحدث: الاحتفاظ بالبيانات.", "Versión superior: conserva datos."),
        "기기 복구 루틴" => lpm_lang_text(lang, "Device Recovery", "Восстановление устройства", "デバイス復旧", "裝置修復", "Khôi phục thiết bị", "Ανάκτηση συσκευής", "डिवाइस रिकवरी", "მოწყობილობის აღდგენა", "Apparaatherstel", "استرداد الجهاز", "Recuperación"),
        "'펌웨어 설치 실패' 등 기기를 복구합니다." => lpm_lang_text(lang, "Recover from firmware install failure.", "Восстановление после сбоя установки прошивки.", "ファームウェア失敗などを修復します。", "修復韌體安裝失敗等問題。", "Khôi phục lỗi cài firmware.", "Ανάκτηση από αποτυχία εγκατάστασης firmware.", "फर्मवेयर इंस्टॉल विफलता से रिकवर करें।", "firmware-ის დაყენების შეცდომის აღდგენა.", "Herstel na mislukte firmware-installatie.", "الاسترداد من فشل تثبيت firmware.", "Recupera fallos de instalación de firmware."),
        "검색 결과가 없습니다." => lpm_lang_text(lang, "No results.", "Нет результатов.", "結果がありません。", "沒有結果。", "Không có kết quả.", "Δεν υπάρχουν αποτελέσματα.", "कोई परिणाम नहीं।", "შედეგები არ არის.", "Geen resultaten.", "لا توجد نتائج.", "Sin resultados."),
        "국가 코드 또는 국가명 검색" => lpm_lang_text(lang, "Search country code or country name", "Поиск кода или страны", "国コードまたは国名を検索", "搜尋國家代碼或國家名稱", "Tìm mã hoặc tên quốc gia", "Αναζήτηση κωδικού ή χώρας", "देश कोड या नाम खोजें", "ქვეყნის კოდის ან სახელის ძებნა", "Zoek landcode of landnaam", "ابحث عن رمز الدولة أو اسمها", "Buscar código o país"),
        "국가 코드 변경" => lpm_lang_text(lang, "Change Country Code", "Изменить код страны", "国コード変更", "變更國家代碼", "Đổi mã quốc gia", "Αλλαγή κωδικού χώρας", "देश कोड बदलें", "ქვეყნის კოდის შეცვლა", "Landcode wijzigen", "تغيير رمز الدولة", "Cambiar código de país"),
        "MTK 드라이버 설치가 필요합니다!" => lpm_lang_text(lang, "MTK driver is required!", "Требуется драйвер MTK!", "MTKドライバーが必要です！", "需要 MTK 驅動程式！", "Cần driver MTK!", "Απαιτείται οδηγός MTK!", "MTK ड्राइवर आवश्यक है!", "საჭიროა MTK დრაივერი!", "MTK-driver vereist!", "مطلوب برنامج تشغيل MTK!", "Se requiere controlador MTK!"),
        "LPMBOX를 사용하기 위해선\n반드시 설치가 필요합니다\n드라이버 설치를 해주세요." => lpm_lang_text(lang, "Install the MTK driver before using LPMBox.", "Установите драйвер MTK перед использованием LPMBox.", "LPMBoxの使用前にMTKドライバーをインストールしてください。", "使用 LPMBox 前請先安裝 MTK 驅動程式。", "Hãy cài driver MTK trước khi dùng LPMBox.", "Εγκαταστήστε τον οδηγό MTK πριν χρησιμοποιήσετε το LPMBox.", "LPMBox उपयोग करने से पहले MTK ड्राइवर इंस्टॉल करें।", "LPMBox-ის გამოყენებამდე დააყენეთ MTK დრაივერი.", "Installeer de MTK-driver voordat u LPMBox gebruikt.", "ثبّت برنامج تشغيل MTK قبل استخدام LPMBox.", "Instale el controlador MTK antes de usar LPMBox."),
        "Microsoft Visual C++ 런타임 설치가 필요합니다!" => lpm_lang_text(lang, "Microsoft Visual C++ Runtime is required!", "Требуется среда выполнения Microsoft Visual C++!", "Microsoft Visual C++ ランタイムが必要です！", "需要 Microsoft Visual C++ 執行階段！", "Cần Microsoft Visual C++ Runtime!", "Απαιτείται Microsoft Visual C++ Runtime!", "Microsoft Visual C++ Runtime आवश्यक है!", "საჭიროა Microsoft Visual C++ Runtime!", "Microsoft Visual C++ Runtime is vereist!", "مطلوب Microsoft Visual C++ Runtime!", "Se requiere Microsoft Visual C++ Runtime!"),
        "VCRUNTIME140.dll / MSVCP140.dll 오류를 방지하기 위해\nMicrosoft Visual C++ x86(32비트) 및 x64(64비트) 런타임을 설치해주세요." => lpm_lang_text(lang, "Install Microsoft Visual C++ x86 (32-bit) and x64 (64-bit) Runtime to prevent VCRUNTIME140.dll / MSVCP140.dll errors.", "Установите Microsoft Visual C++ Runtime x86 (32-разрядную) и x64 (64-разрядную), чтобы предотвратить ошибки VCRUNTIME140.dll / MSVCP140.dll.", "VCRUNTIME140.dll / MSVCP140.dll エラーを防ぐため、Microsoft Visual C++ x86（32ビット）および x64（64ビット）ランタイムをインストールしてください。", "為避免 VCRUNTIME140.dll / MSVCP140.dll 錯誤，請安裝 Microsoft Visual C++ x86（32 位元）與 x64（64 位元）執行階段。", "Hãy cài Microsoft Visual C++ Runtime x86 (32-bit) và x64 (64-bit) để tránh lỗi VCRUNTIME140.dll / MSVCP140.dll.", "Εγκαταστήστε το Microsoft Visual C++ Runtime x86 (32-bit) και x64 (64-bit) για να αποτρέψετε σφάλματα VCRUNTIME140.dll / MSVCP140.dll.", "VCRUNTIME140.dll / MSVCP140.dll त्रुटियों से बचने के लिए Microsoft Visual C++ x86 (32-bit) और x64 (64-bit) Runtime इंस्टॉल करें।", "VCRUNTIME140.dll / MSVCP140.dll შეცდომების თავიდან ასაცილებლად დააყენეთ Microsoft Visual C++ x86 (32-bit) და x64 (64-bit) Runtime.", "Installeer Microsoft Visual C++ Runtime x86 (32-bits) en x64 (64-bits) om VCRUNTIME140.dll / MSVCP140.dll-fouten te voorkomen.", "ثبّت Microsoft Visual C++ Runtime بإصداري x86 ‏(32 بت) وx64 ‏(64 بت) لمنع أخطاء VCRUNTIME140.dll / MSVCP140.dll.", "Instale Microsoft Visual C++ Runtime x86 (32 bits) y x64 (64 bits) para evitar errores de VCRUNTIME140.dll / MSVCP140.dll."),
        "Visual C++ 설치" => lpm_lang_text(lang, "Install Visual C++", "Установить Visual C++", "Visual C++ をインストール", "安裝 Visual C++", "Cài Visual C++", "Εγκατάσταση Visual C++", "Visual C++ इंस्टॉल करें", "Visual C++-ის დაყენება", "Visual C++ installeren", "تثبيت Visual C++", "Instalar Visual C++"),
        "Proceed to next step." | "Proceed to the next step." => lpm_lang_text(lang, "Proceed to the next step.", "Перейдите к следующему шагу.", "次のステップへ進んでください。", "請進入下一步。", "Chuyển sang bước tiếp theo.", "Προχωρήστε στο επόμενο βήμα.", "अगले चरण पर जाएँ।", "გადადით შემდეგ ეტაპზე.", "Ga verder naar de volgende stap.", "انتقل إلى الخطوة التالية.", "Continúe con el siguiente paso."),
        "Check the image folder information." => lpm_lang_text(lang, "Check the selected image folder.", "Проверьте выбранную папку image.", "選択したimageフォルダーを確認します。", "檢查已選擇的 image 資料夾。", "Kiểm tra thư mục image đã chọn.", "Ελέγξτε τον επιλεγμένο φάκελο image.", "चयनित image फ़ोल्डर जांचें।", "შეამოწმეთ არჩეული image საქაღალდე.", "Controleer de geselecteerde image-map.", "تحقق من مجلد image المحدد.", "Comprueba la carpeta image seleccionada."),
        _ => return None,
    })
}

fn lpm_translate_stage10_cleanup(lang: LanguageOption, content: String) -> String {
    if lang.is_korean() {
        return content;
    }

    let mut out = content;
    for (from, to) in lpm_stage10_cleanup_pairs(lang) {
        out = out.replace(from, to);
    }
    out
}

fn lpm_stage10_cleanup_pairs(lang: LanguageOption) -> &'static [(&'static str, &'static str)] {
    use LanguageOption::*;
    match lang {
        English => &[
            ("PRC(China ROM) 또는\nROW(Global ROM)을 install.", "Install PRC or ROW ROM."),
            ("ROW(Global ROM) Version을\n업데이트 합니다.", "Update the ROW ROM version."),
            ("기기가 켜지지 않거나, 무한 재부팅 등 다양한 errors를 고칩니다.", "Fixes no boot, boot loop, and other errors."),
            ("설치된 Version", "installed version"),
            ("Version을", "version"),
            ("업데이트 합니다", "update"),
            ("설치합니다", "install"),
            ("고칩니다", "fixes"),
            ("또는", "or"),
            ("합니다", ""),
            ("해주세요", ""),
            ("되어있지", "is not"),
            ("않거나", "or"),
            ("무한 재부팅", "boot loop"),
            ("다양한 오류", "various errors"),
            ("오류를", "errors"),
            ("기기가", "The device"),
            ("켜지지", "does not turn on"),
            ("설치된", "installed"),
            ("보다 낮은 경우", "lower than"),
            ("보다 높은 경우", "higher than"),
            ("초기화 O", "factory reset required"),
            ("초기화 X", "keep data"),
            ("선택한 image folder", "Selected image folder"),
            ("image folder", "image folder"),
            ("기기 감지", "device detection"),
            ("감지 complete", "detection complete"),
            ("OTA update 완료", "OTA update completed"),
            ("저장했습니다", "saved"),
            ("언어 setting file", "language setting file"),
            ("프로그램 language", "program language"),
            ("시작합니다", "starting"),
            ("완료되었습니다", "completed"),
            ("완료", "completed"),
            ("실패", "failed"),
            ("오류", "error"),
            ("경고", "warning"),
            ("차단", "blocked"),
            ("국가 code", "country code"),
            ("폴더", "folder"),
        ],
        Japanese => &[
            ("PRC(中国版ROM) または\nROW(グローバルROM)をインストールします。", "PRC/ROW ROMを切り替えます。"),
            ("ROW(グローバルROM) バージョンを\n更新します。", "ROW ROMを更新します。"),
            ("PRC(中国版ROM) 또는\nROW(グローバルROM)을 インストール합니다.", "PRC/ROW ROMを切り替えます。"),
            ("ROW(グローバルROM) Version을\n更新합니다.", "ROW ROMを更新します。"),
            (" 기기에 ", "デバイスに"),
            ("설치된", "インストール済み"),
            ("Version", "バージョン"),
            ("보다 낮은 경우", "より低い場合"),
            ("보다 높은 경우", "より高い場合"),
            ("초기화 O", "初期化あり"),
            ("초기화 X", "初期化なし"),
            ("진행해 주세요", "進めてください"),
            ("기기가 켜지지 않거나", "起動不可や"),
            ("무한 재부팅", "再起動ループ"),
            ("다양한 errors를 고칩니다", "さまざまなエラーを修復します"),
            ("등", "など"),
            ("또는", "または"),
            ("을 ", "を"),
            ("를 ", "を"),
            ("합니다", "します"),
            ("해주세요", "してください"),
            ("선택한 imageフォルダー", "選択したimageフォルダー"),
            ("imageフォルダー情報", "imageフォルダー情報"),
            ("デバイスの元ROM", "元ROM"),
            ("インストール済みROM", "インストール済みROM"),
            ("ブロック完了", "ブロック済み"),
        ],
        TraditionalChinese => &[
            ("PRC(中國版 ROM) または\nROW(全球版 ROM)を安裝します。", "安裝 PRC 或 ROW ROM。"),
            ("ROW(全球版 ROM) Version을\n更新합니다.", "更新 ROW ROM 版本。"),
            ("또는", "或"),
            ("설치합니다", "安裝"),
            ("업데이트 합니다", "更新"),
            ("Version을", "版本"),
            ("실제 값과 다를 수 있음", "可能與實際值不同"),
            ("초기화 O", "需要清除資料"),
            ("초기화 X", "保留資料"),
            ("보다 낮은 경우", "較低版本"),
            ("보다 높은 경우", "較高版本"),
        ],
        Spanish => &[
            ("PRC(ROM china) 또는\nROW(ROM global)을 instalar.", "Instala ROM PRC o ROW."),
            ("ROW(ROM global) Version을\nactualización 합니다.", "Actualiza la ROM ROW."),
            ("Version을", "versión"),
            ("또는", "o"),
            ("설치합니다", "instala"),
            ("업데이트 합니다", "actualiza"),
            ("기기가 켜지지 않거나", "no arranca o"),
            ("무한 재부팅", "bucle de arranque"),
            ("다양한 errors를 고칩니다", "repara varios errores"),
            ("초기화 O", "requiere restablecer"),
            ("초기화 X", "conserva datos"),
        ],
        Russian | Vietnamese | Greek | Hindi | Georgian | Dutch | Arabic => &[
            ("또는", " / "),
            ("을 ", " "),
            ("를 ", " "),
            ("합니다", ""),
            ("해주세요", ""),
            ("Version", "version"),
            ("실제 값과 다를 수 있음", "may differ from the actual value"),
        ],
        Korean => &[],
    }
}

fn lpm_sidebar_label_size() -> u32 {
    match active_language_option() {
        LanguageOption::English | LanguageOption::Korean => 13,
        _ => 11,
    }
}

fn lpm_dashboard_title_size() -> u32 {
    match active_language_option() {
        LanguageOption::English | LanguageOption::Korean | LanguageOption::Russian => 15,
        LanguageOption::Japanese | LanguageOption::TraditionalChinese => 14,
        LanguageOption::Arabic => 12,
        _ => 13,
    }
}

fn lpm_dashboard_body_size() -> u32 {
    match active_language_option() {
        LanguageOption::English | LanguageOption::Korean | LanguageOption::Russian => 12,
        LanguageOption::Arabic => 10,
        _ => 11,
    }
}

fn lpm_dashboard_description_size() -> f32 {
    (lpm_dashboard_body_size() as f32 - 0.5).max(8.0)
}

fn lpm_dashboard_promo_text(lang: LanguageOption, key: &str) -> &'static str {
    match key {
        "donate_title" => match lang {
            LanguageOption::English => "Support the developer",
            LanguageOption::Korean => "후원하기",
            LanguageOption::Russian => "Поддержать разработчика",
            LanguageOption::Japanese => "支援する",
            LanguageOption::TraditionalChinese => "贊助支持",
            LanguageOption::Vietnamese => "Ủng hộ",
            LanguageOption::Greek => "Υποστήριξη",
            LanguageOption::Hindi => "सहयोग करें",
            LanguageOption::Georgian => "მხარდაჭერა",
            LanguageOption::Dutch => "Doneren",
            LanguageOption::Arabic => "دعم المطور",
            LanguageOption::Spanish => "Apoyar",
        },
        "donate_description" => match lang {
            LanguageOption::English => "Your support is a great source of strength and encouragement for the developer.",
            LanguageOption::Korean => "개발자에게 큰 힘과 응원이 됩니다.",
            LanguageOption::Russian => "Ваша поддержка придаёт разработчику сил и вдохновения.",
            LanguageOption::Japanese => "開発者にとって大きな力と励みになります。",
            LanguageOption::TraditionalChinese => "您的支持將給開發者帶來莫大的力量與鼓勵。",
            LanguageOption::Vietnamese => "Sự ủng hộ của bạn là nguồn động lực lớn cho nhà phát triển.",
            LanguageOption::Greek => "Η υποστήριξή σας δίνει μεγάλη δύναμη και ενθάρρυνση στον προγραμματιστή.",
            LanguageOption::Hindi => "आपका सहयोग डेवलपर को बहुत शक्ति और प्रोत्साहन देता है।",
            LanguageOption::Georgian => "თქვენი მხარდაჭერა დეველოპერს დიდ ძალასა და მოტივაციას აძლევს.",
            LanguageOption::Dutch => "Uw steun geeft de ontwikkelaar veel kracht en motivatie.",
            LanguageOption::Arabic => "يمنح دعمك المطور قوة وتشجيعًا كبيرين.",
            LanguageOption::Spanish => "Tu apoyo brinda una gran fuerza y motivación al desarrollador.",
        },
        "start_title" => match lang {
            LanguageOption::English => "Start LPMBOX",
            LanguageOption::Korean => "LPMBOX 시작",
            LanguageOption::Russian => "Запустить LPMBOX",
            LanguageOption::Japanese => "LPMBOXを開始",
            LanguageOption::TraditionalChinese => "啟動 LPMBOX",
            LanguageOption::Vietnamese => "Bắt đầu LPMBOX",
            LanguageOption::Greek => "Έναρξη LPMBOX",
            LanguageOption::Hindi => "LPMBOX शुरू करें",
            LanguageOption::Georgian => "LPMBOX-ის დაწყება",
            LanguageOption::Dutch => "LPMBOX starten",
            LanguageOption::Arabic => "بدء LPMBOX",
            LanguageOption::Spanish => "Iniciar LPMBOX",
        },
        "start_description" => match lang {
            LanguageOption::English => "Install a China ROM or global ROM, update the global ROM version, or recover the device.",
            LanguageOption::Korean => "기기에 중국 내수롬 또는 글로벌롬 설치, 글로벌롬 버전 업데이트, 기기를 복구할 수 있습니다.",
            LanguageOption::Russian => "Устанавливайте китайскую или глобальную ROM, обновляйте глобальную ROM и восстанавливайте устройство.",
            LanguageOption::Japanese => "中国版ROMまたはグローバルROMのインストール、グローバルROMの更新、端末の復旧ができます。",
            LanguageOption::TraditionalChinese => "可為裝置安裝中國版或全球版 ROM、更新全球版 ROM，並修復裝置。",
            LanguageOption::Vietnamese => "Cài ROM nội địa Trung Quốc hoặc ROM quốc tế, cập nhật ROM quốc tế và khôi phục thiết bị.",
            LanguageOption::Greek => "Εγκαταστήστε κινεζική ή παγκόσμια ROM, ενημερώστε την παγκόσμια ROM και ανακτήστε τη συσκευή.",
            LanguageOption::Hindi => "डिवाइस पर चीनी या ग्लोबल ROM इंस्टॉल करें, ग्लोबल ROM अपडेट करें और डिवाइस रिकवर करें।",
            LanguageOption::Georgian => "მოწყობილობაზე დააყენეთ ჩინური ან გლობალური ROM, განაახლეთ გლობალური ROM და აღადგინეთ მოწყობილობა.",
            LanguageOption::Dutch => "Installeer een Chinese of wereldwijde ROM, werk de wereldwijde ROM bij en herstel het apparaat.",
            LanguageOption::Arabic => "يمكنك تثبيت الروم الصيني أو العالمي، وتحديث الروم العالمي، واستعادة الجهاز.",
            LanguageOption::Spanish => "Instala una ROM china o global, actualiza la ROM global y recupera el dispositivo.",
        },
        "more_title" => match lang {
            LanguageOption::English => "More programs",
            LanguageOption::Korean => "더 많은 프로그램",
            LanguageOption::Russian => "Больше программ",
            LanguageOption::Japanese => "その他のプログラム",
            LanguageOption::TraditionalChinese => "更多程式",
            LanguageOption::Vietnamese => "Thêm chương trình",
            LanguageOption::Greek => "Περισσότερα προγράμματα",
            LanguageOption::Hindi => "और प्रोग्राम",
            LanguageOption::Georgian => "მეტი პროგრამა",
            LanguageOption::Dutch => "Meer programma's",
            LanguageOption::Arabic => "المزيد من البرامج",
            LanguageOption::Spanish => "Más programas",
        },
        "more_description" => match lang {
            LanguageOption::English => "Find useful programs and guides for Xiaoxin Pad.",
            LanguageOption::Korean => "샤오신패드에 유용한 프로그램과 가이드를 확인하실 수 있습니다.",
            LanguageOption::Russian => "Полезные программы и руководства для Xiaoxin Pad.",
            LanguageOption::Japanese => "Xiaoxin Padに役立つプログラムとガイドを確認できます。",
            LanguageOption::TraditionalChinese => "查看適用於小新平板的實用程式與指南。",
            LanguageOption::Vietnamese => "Xem các chương trình và hướng dẫn hữu ích cho Xiaoxin Pad.",
            LanguageOption::Greek => "Δείτε χρήσιμα προγράμματα και οδηγούς για το Xiaoxin Pad.",
            LanguageOption::Hindi => "Xiaoxin Pad के लिए उपयोगी प्रोग्राम और गाइड देखें।",
            LanguageOption::Georgian => "იხილეთ Xiaoxin Pad-ისთვის სასარგებლო პროგრამები და გზამკვლევები.",
            LanguageOption::Dutch => "Bekijk nuttige programma's en handleidingen voor Xiaoxin Pad.",
            LanguageOption::Arabic => "اطّلع على برامج وأدلة مفيدة لجهاز Xiaoxin Pad.",
            LanguageOption::Spanish => "Consulta programas y guías útiles para Xiaoxin Pad.",
        },
        "move_button" => match lang {
            LanguageOption::English => "Open",
            LanguageOption::Korean => "이동",
            LanguageOption::Russian => "Перейти",
            LanguageOption::Japanese => "移動",
            LanguageOption::TraditionalChinese => "前往",
            LanguageOption::Vietnamese => "Đi tới",
            LanguageOption::Greek => "Μετάβαση",
            LanguageOption::Hindi => "खोलें",
            LanguageOption::Georgian => "გადასვლა",
            LanguageOption::Dutch => "Openen",
            LanguageOption::Arabic => "انتقال",
            LanguageOption::Spanish => "Ir",
        },
        "start_button" => match lang {
            LanguageOption::English => "Get started",
            LanguageOption::Korean => "시작하기",
            LanguageOption::Russian => "Начать",
            LanguageOption::Japanese => "開始する",
            LanguageOption::TraditionalChinese => "開始使用",
            LanguageOption::Vietnamese => "Bắt đầu",
            LanguageOption::Greek => "Έναρξη",
            LanguageOption::Hindi => "शुरू करें",
            LanguageOption::Georgian => "დაწყება",
            LanguageOption::Dutch => "Aan de slag",
            LanguageOption::Arabic => "ابدأ الآن",
            LanguageOption::Spanish => "Empezar",
        },
        _ => "",
    }
}

fn lpm_routine_version_log_text(
    lang: LanguageOption,
    firmware: Option<&FirmwareInfo>,
) -> String {
    let image_version = if let Some(info) = firmware {
        let model = info.model.trim();
        let region = match info.region {
            RomRegion::Prc => "PRC",
            RomRegion::Row => "ROW",
            RomRegion::Unknown => "UNKNOWN",
        };
        let raw_version = info
            .version
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("UNKNOWN");
        let version = raw_version.strip_suffix("_ST").unwrap_or(raw_version);

        format!("{model}_{region}_{version}")
    } else {
        "UNKNOWN_UNKNOWN_UNKNOWN".to_string()
    };

    match lang {
        LanguageOption::English => format!(
            "[LPMBOX] LPMBOX program version: {APP_DISPLAY_VERSION} / Selected image folder: {image_version}"
        ),
        LanguageOption::Korean => format!(
            "[LPMBOX] LPMBOX 프로그램 버전: {APP_DISPLAY_VERSION} / 선택한 image 폴더: {image_version}"
        ),
        LanguageOption::Russian => format!(
            "[LPMBOX] Версия программы LPMBOX: {APP_DISPLAY_VERSION} / Выбранная папка image: {image_version}"
        ),
        LanguageOption::Japanese => format!(
            "[LPMBOX] LPMBOXプログラムバージョン: {APP_DISPLAY_VERSION} / 選択したimageフォルダー: {image_version}"
        ),
        LanguageOption::TraditionalChinese => format!(
            "[LPMBOX] LPMBOX 程式版本：{APP_DISPLAY_VERSION} / 已選擇的 image 資料夾：{image_version}"
        ),
        LanguageOption::Vietnamese => format!(
            "[LPMBOX] Phiên bản chương trình LPMBOX: {APP_DISPLAY_VERSION} / Thư mục image đã chọn: {image_version}"
        ),
        LanguageOption::Greek => format!(
            "[LPMBOX] Έκδοση προγράμματος LPMBOX: {APP_DISPLAY_VERSION} / Επιλεγμένος φάκελος image: {image_version}"
        ),
        LanguageOption::Hindi => format!(
            "[LPMBOX] LPMBOX प्रोग्राम संस्करण: {APP_DISPLAY_VERSION} / चयनित image फ़ोल्डर: {image_version}"
        ),
        LanguageOption::Georgian => format!(
            "[LPMBOX] LPMBOX პროგრამის ვერსია: {APP_DISPLAY_VERSION} / არჩეული image საქაღალდე: {image_version}"
        ),
        LanguageOption::Dutch => format!(
            "[LPMBOX] LPMBOX-programmaversie: {APP_DISPLAY_VERSION} / Geselecteerde image-map: {image_version}"
        ),
        LanguageOption::Arabic => format!(
            "[LPMBOX] إصدار برنامج LPMBOX: {APP_DISPLAY_VERSION} / مجلد image المحدد: {image_version}"
        ),
        LanguageOption::Spanish => format!(
            "[LPMBOX] Versión del programa LPMBOX: {APP_DISPLAY_VERSION} / Carpeta image seleccionada: {image_version}"
        ),
    }
}

fn lpm_equal_button_width(labels: &[&str], min_width: f32, max_width: f32) -> f32 {
    let mut widest = min_width;
    for label in labels {
        let mut width = 32.0;
        for ch in label.chars() {
            width += if ch.is_ascii() { 7.0 } else { 11.0 };
        }
        if width > widest {
            widest = width;
        }
    }
    widest.clamp(min_width, max_width)
}

fn lpm_routine_title_size() -> u32 {
    match active_language_option() {
        LanguageOption::English | LanguageOption::Korean => 22,
        LanguageOption::Japanese | LanguageOption::TraditionalChinese => 21,
        _ => 19,
    }
}

fn lpm_routine_body_size() -> u32 {
    match active_language_option() {
        LanguageOption::English | LanguageOption::Korean => 13,
        _ => 12,
    }
}

fn lpm_slide_title_size() -> u32 {
    match active_language_option() {
        LanguageOption::English | LanguageOption::Korean => 13,
        _ => 12,
    }
}

fn lpm_slide_body_size() -> u32 {
    match active_language_option() {
        LanguageOption::English | LanguageOption::Korean => 11,
        _ => 10,
    }
}

fn lpm_omitted_log_message(count: usize) -> String {
    match active_language_option() {
        LanguageOption::English => format!("... {count} previous log lines were omitted for screen rendering performance. You can check the full contents by saving the log. ..."),
        LanguageOption::Korean => format!("... 이전 로그 {count}개는 화면 렌더링 성능을 위해 생략되었습니다. 전체 내용은 로그 저장으로 확인할 수 있습니다. ..."),
        LanguageOption::Russian => format!("... {count} предыдущих строк журнала пропущено для производительности отображения. Полный текст можно проверить, сохранив журнал. ..."),
        LanguageOption::Japanese => format!("... 以前のログ {count} 件は表示性能のため省略されました。全文はログ保存で確認できます。 ..."),
        LanguageOption::TraditionalChinese => format!("... 為了畫面渲染效能，已省略先前 {count} 筆日誌。完整內容可透過儲存日誌確認。 ..."),
        LanguageOption::Vietnamese => format!("... {count} dòng nhật ký trước đã được ẩn để tối ưu hiệu năng hiển thị. Có thể xem toàn bộ bằng cách lưu nhật ký. ..."),
        LanguageOption::Greek => format!("... {count} προηγούμενες γραμμές καταγραφής παραλείφθηκαν για καλύτερη απόδοση εμφάνισης. Μπορείτε να δείτε το πλήρες περιεχόμενο αποθηκεύοντας το αρχείο καταγραφής. ..."),
        LanguageOption::Hindi => format!("... पिछली {count} लॉग पंक्तियाँ स्क्रीन रेंडरिंग प्रदर्शन के लिए छोड़ी गईं। पूरा विवरण लॉग सहेजकर देखा जा सकता है। ..."),
        LanguageOption::Georgian => format!("... წინა ლოგის {count} ხაზი ეკრანის სწრაფი ჩვენებისთვის გამოტოვებულია. სრული შინაარსი შეგიძლიათ ნახოთ ლოგის შენახვით. ..."),
        LanguageOption::Dutch => format!("... {count} vorige logregels zijn overgeslagen voor betere schermprestaties. De volledige inhoud is beschikbaar door het logboek op te slaan. ..."),
        LanguageOption::Arabic => format!("... تم إخفاء {count} سطرًا سابقًا من السجل لتحسين أداء العرض. يمكن عرض المحتوى الكامل بحفظ السجل. ..."),
        LanguageOption::Spanish => format!("... Se omitieron {count} líneas de registro anteriores para mejorar el rendimiento de renderizado. Puede ver el contenido completo guardando el registro. ..."),
    }
}

fn lpm_translate_exact_stage8(lang: LanguageOption, key: &str) -> Option<&'static str> {
    Some(match key {
        "저장된 언어 설정" => lpm_lang_text(lang, "Saved language setting", "Сохранённая настройка языка", "保存された言語設定", "已儲存的語言設定", "Cài đặt ngôn ngữ đã lưu", "Αποθηκευμένη ρύθμιση γλώσσας", "सहेजी गई भाषा सेटिंग", "შენახული ენის პარამეტრი", "Opgeslagen taalinstelling", "إعداد اللغة المحفوظ", "Configuración de idioma guardada"),
        "Windows OS 언어" => lpm_lang_text(lang, "Windows OS language", "Язык Windows", "Windows OSの言語", "Windows OS 語言", "Ngôn ngữ Windows", "Γλώσσα Windows", "Windows OS भाषा", "Windows OS-ის ენა", "Windows-taal", "لغة Windows", "Idioma de Windows"),
        "기본값 English" => lpm_lang_text(lang, "Default English", "Английский по умолчанию", "既定値 English", "預設 English", "Mặc định English", "Προεπιλογή English", "डिफ़ॉल्ट English", "ნაგულისხმევი English", "Standaard English", "English افتراضي", "English predeterminado"),
        "한국어 (ko)" => lpm_lang_text(
            lang,
            "Korean (ko)",
            "Корейский (ko)",
            "韓国語 (ko)",
            "韓文 (ko)",
            "Tiếng Hàn (ko)",
            "Κορεατικά (ko)",
            "कोरियाई (ko)",
            "კორეული (ko)",
            "Koreaans (ko)",
            "الكورية (ko)",
            "Coreano (ko)",
        ),
        _ => return None,
    })
}

fn lpm_translate_stage8_phrasewise(lang: LanguageOption, content: String) -> String {
    if lang.is_korean() || !content.chars().any(|c| ('가'..='힣').contains(&c)) {
        return content;
    }

    let mut out = content;
    for (from, to) in lpm_stage8_phrase_pairs(lang) {
        out = out.replace(from, to);
    }
    out
}

fn lpm_stage8_phrase_pairs(lang: LanguageOption) -> &'static [(&'static str, &'static str)] {
    use LanguageOption::*;
    match lang {
        English => &[
            ("[설정]", "[Settings]"),
            ("[경고]", "[Warning]"),
            ("[추가 옵션]", "[Additional Options]"),
            ("[ROM 옵션]", "[ROM Options]"),
            ("[프로그램]", "[Program]"),
            ("[완료]", "[Done]"),
            ("초기 언어 설정", "initial language setting"),
            ("언어 설정 파일 경로", "language setting file path"),
            ("언어 설정 파일에 저장했습니다", "saved to the language setting file"),
            ("기준", "source"),
            ("이미 최신 릴리즈 확인이 진행 중입니다.", "A latest release check is already in progress."),
            ("최신 LPMBox 릴리즈를 확인합니다.", "Checking the latest LPMBox release."),
            ("새 LPMBox 버전을 찾았습니다", "New LPMBox version found"),
            ("릴리즈 ZIP 파일", "Release ZIP file"),
            ("대시보드에 업데이트 안내 창을 표시합니다", "Showing the update notice on the dashboard"),
            ("수동 확인을 위해 GitHub Releases 페이지를 엽니다.", "Opening the GitHub Releases page for manual checking."),
            ("GitHub Releases 페이지 열기 실패", "Failed to open the GitHub Releases page"),
            ("이번 업데이트 안내를 다음에 다시 확인합니다.", "This update notice will be checked again later."),
            ("펌웨어 다운로드 링크 열기 실패", "Failed to open the firmware download link"),
            ("개발자 유튜브 링크 열기 실패", "Failed to open the developer YouTube link"),
            ("후원하기 링크 열기 실패", "Failed to open the sponsorship link"),
            ("피드백 링크 열기 실패", "Failed to open the feedback link"),
            ("1번 옵션을 시작합니다", "Starting option 1"),
            ("2번 옵션을 시작합니다", "Starting option 2"),
            ("3번 옵션을 시작합니다", "Starting option 3"),
            ("PRC/ROW 펌웨어 설치", "PRC/ROW firmware installation"),
            ("ROW(글로벌) 펌웨어 업데이트", "ROW global firmware update"),
            ("펌웨어 업데이트", "firmware update"),
            ("펌웨어 설치", "firmware installation"),
            ("[데이터 초기화]", "[factory reset]"),
            ("[데이터 유지]", "[keep data]"),
            ("MTK 드라이버 설치를 시작합니다.", "Starting MTK driver installation."),
            ("MTK 드라이버 설치 파일 준비를 시작합니다.", "Preparing the MTK driver installer."),
            ("MTK 드라이버 설치 파일 준비가 완료되었습니다.", "MTK driver installer preparation is complete."),
            ("기기가 연결 되어있지 않거나, ADB가 감지되지 않습니다.", "The device is not connected or ADB was not detected."),
            ("케이블을 PC 후면", "Connect the cable to the rear USB port of the PC"),
            ("꽂아주세요", "and try again"),
            ("올바른 데이터 케이블을 사용해주세요", "Use a proper data cable"),
            ("QnA 참고", "see Q&A"),
            ("연결한 기기(", "Connected device ("),
            (")는 LPMBOX를 사용할 수 없습니다.", ") cannot use LPMBOX."),
            (")와 선택한 image 폴더(", ") and selected image folder ("),
            (")는 호환되지 않습니다.", ") are not compatible."),
            ("올바른 image 폴더 재선택", "Reselect the correct image folder"),
            ("다른 버전 파일로 재시도", "Retry with a different version file"),
            ("펌웨어 검사 실패", "Firmware check failed"),
            ("펌웨어 검사 결과", "Firmware check result"),
            ("펌웨어 검사 완료", "Firmware check completed"),
            ("유효성 검사", "Validation check"),
            ("차단 버전 목록", "blocked version list"),
            ("ROM 타입", "ROM type"),
            ("필수 partition 검사", "required partition check"),
            ("필수 partition 상세", "required partition details"),
            ("미리보기", "preview"),
            ("ROOT 경로에 SPFlashToolV6 준비 완료", "SPFlashToolV6 is ready in the ROOT path"),
            ("파일이 생성되지 않았거나 크기가 0입니다", "the file was not created or its size is 0"),
            ("태블릿 재부팅 중", "rebooting tablet"),
            ("[1단계]", "[Step 1]"),
            ("[2단계]", "[Step 2]"),
            ("[3단계]", "[Step 3]"),
            ("[4단계]", "[Step 4]"),
            ("[5단계]", "[Step 5]"),
            ("[6단계]", "[Step 6]"),
            ("[7단계]", "[Step 7]"),
            ("[8단계]", "[Step 8]"),
            ("image 폴더 검사 및 플래싱 준비", "image folder check and flashing preparation"),
            ("image 폴더 기본 검사 및 image 파일 모델/플랫폼 확인", "basic image folder check and image file model/platform verification"),
            ("image 폴더 기본 검사", "basic image folder check"),
            ("image 폴더 검사 실패", "image folder check failed"),
            ("Flash Plan 준비 및 작업용 scatter/xml 생성", "Flash Plan preparation and work scatter/xml generation"),
            ("current slot A 설정", "set current slot A"),
            ("MediaTek PreLoader 포트 감지", "MediaTek PreLoader port detection"),
            ("ROM 설치", "ROM installation"),
            ("기기 복구 준비", "device recovery preparation"),
            ("기기 복구 준비 실패", "device recovery preparation failed"),
            ("옵션 선택 페이지와 국가 코드 변경 없이 진행합니다.", "Continuing without the options page or country code change."),
            ("데이터 유지/초기화 정책", "data keep/reset policy"),
            ("작업이 완료되었습니다", "task completed"),
            ("국가 코드 재설정 실패", "country code reset failed"),
            ("국가 코드 재설정용 재부팅 및 MediaTek PreLoader 포트 감지", "reboot and MediaTek PreLoader port detection for country code reset"),
            ("proinfo 백업 및 선택한 국가 코드로 수정", "back up proinfo and patch it with the selected country code"),
            ("proinfo 전용 Flash Plan 준비 및 작업용 scatter/xml 생성", "prepare a proinfo-only Flash Plan and work scatter/xml"),
            ("수정한 proinfo 플래싱용 재부팅", "reboot for flashing the modified proinfo"),
            ("SPFlashToolV6 proinfo 파티션만 플래싱", "flash only the proinfo partition with SPFlashToolV6"),
            ("SPFlashToolV6 국가 코드 재설정 실패", "SPFlashToolV6 country code reset failed"),
            ("Flash Plan 준비 전", "before Flash Plan preparation"),
            ("spft_log 폴더를 제거했습니다", "removed the spft_log folder"),
            ("국가 코드 변경을 위해 proinfo 파티션을 백업합니다.", "Backing up the proinfo partition for the country code change."),
            ("30초 안에 MediaTek PreLoader USB VCOM 포트를 찾지 못했습니다", "could not find the MediaTek PreLoader USB VCOM port within 30 seconds"),
            ("감지 문자열", "detected string"),
            ("백업 파일 저장 실패", "failed to save the backup file"),
            ("사용자가 선택한 국가 코드", "user-selected country code"),
            ("proinfo에 국가 코드를 변경합니다.", "Changing the country code in proinfo."),
            ("국가 코드 변경 실패", "country code change failed"),
            ("국가 코드 변경 완료", "country code change completed"),
            ("수정한 proinfo 파일을 image 폴더로 이동하지 못했습니다", "failed to move the modified proinfo file to the image folder"),
            ("proinfo 파일에 국가 코드가 설정 되었는지 확인합니다.", "Checking whether the country code was written to the proinfo file."),
            ("국가 코드 재수정 실패", "country code re-patch failed"),
            ("국가 코드가 설정되지 않았습니다", "country code was not written"),
            ("국가 코드 확인 완료", "country code verification completed"),
            ("올바르지 않은 국가 코드입니다", "Invalid country code"),
            ("국가 코드 토큰을 찾지 못했습니다", "country code token was not found"),
            ("기기에", "On the device,"),
            ("시도해주세요", "try this option"),
            ("지원하지 않습니다", "is not supported"),
            ("proinfo 파티션 추출 중", "extracting the proinfo partition"),
            ("patch 변경 수", "patch change count"),
            ("작업 scatter XML 재파싱", "work scatter XML reparse"),
            ("데이터 유지 여부", "keep data"),
            ("변경", "changes"),
            ("오류", "errors"),
            ("경고", "warnings"),
            ("성공", "success"),
            ("실패", "failed"),
            ("확인 불가", "cannot verify"),
            ("완료", "complete"),
            ("포트가 감지되면", "When the port is detected,"),
            ("바로 실행합니다", "runs immediately"),
            ("주의: 이 작업은 실제 SPFlashToolV6 download를 실행합니다.", "Caution: this task runs the actual SPFlashToolV6 download."),
            ("도구 폴더", "tool folder"),
            ("저장하는 중", "saving"),
        ],
        Japanese => &[
            ("[설정]", "[設定]"),
            ("[경고]", "[警告]"),
            ("[추가 옵션]", "[追加オプション]"),
            ("[ROM 옵션]", "[ROMオプション]"),
            ("[프로그램]", "[プログラム]"),
            ("[완료]", "[完了]"),
            ("초기 언어 설정", "初期言語設定"),
            ("언어 설정 파일 경로", "言語設定ファイルのパス"),
            ("언어 설정 파일에 저장했습니다", "言語設定ファイルに保存しました"),
            ("기준", "基準"),
            ("최신 LPMBox 릴리즈를 확인합니다.", "最新のLPMBoxリリースを確認します。"),
            ("새 LPMBox 버전을 찾았습니다", "新しいLPMBoxバージョンを検出しました"),
            ("릴리즈 ZIP 파일", "リリースZIPファイル"),
            ("GitHub Releases 페이지 열기 실패", "GitHub Releasesページを開けませんでした"),
            ("1번 옵션을 시작합니다", "1番オプションを開始します"),
            ("2번 옵션을 시작합니다", "2番オプションを開始します"),
            ("3번 옵션을 시작합니다", "3番オプションを開始します"),
            ("PRC/ROW 펌웨어 설치", "PRC/ROWファームウェアインストール"),
            ("ROW(글로벌) 펌웨어 업데이트", "ROWグローバルファームウェア更新"),
            ("[데이터 초기화]", "[データ初期化]"),
            ("[데이터 유지]", "[データ保持]"),
            ("MTK 드라이버 설치를 시작합니다.", "MTKドライバーのインストールを開始します。"),
            ("펌웨어 검사 실패", "ファームウェア検査失敗"),
            ("유효성 검사", "有効性検査"),
            ("[1단계]", "[ステップ1]"),
            ("[2단계]", "[ステップ2]"),
            ("[3단계]", "[ステップ3]"),
            ("[4단계]", "[ステップ4]"),
            ("[5단계]", "[ステップ5]"),
            ("[6단계]", "[ステップ6]"),
            ("[7단계]", "[ステップ7]"),
            ("[8단계]", "[ステップ8]"),
            ("작업이 완료되었습니다", "作業が完了しました"),
            ("국가 코드 재설정 실패", "国コードリセット失敗"),
            ("올바르지 않은 국가 코드입니다", "無効な国コードです"),
            ("성공", "成功"),
            ("실패", "失敗"),
            ("오류", "エラー"),
            ("경고", "警告"),
            ("확인 불가", "確認不可"),
            ("완료", "完了"),
        ],
        TraditionalChinese => &[
            ("[설정]", "[設定]"),
            ("[경고]", "[警告]"),
            ("[추가 옵션]", "[其他選項]"),
            ("[ROM 옵션]", "[ROM 選項]"),
            ("[프로그램]", "[程式]"),
            ("[완료]", "[完成]"),
            ("초기 언어 설정", "初始語言設定"),
            ("언어 설정 파일 경로", "語言設定檔路徑"),
            ("언어 설정 파일에 저장했습니다", "已儲存到語言設定檔"),
            ("기준", "依據"),
            ("최신 LPMBox 릴리즈를 확인합니다.", "正在檢查最新 LPMBox 版本。"),
            ("새 LPMBox 버전을 찾았습니다", "已找到新的 LPMBox 版本"),
            ("릴리즈 ZIP 파일", "Release ZIP 檔案"),
            ("GitHub Releases 페이지 열기 실패", "無法開啟 GitHub Releases 頁面"),
            ("1번 옵션을 시작합니다", "開始選項 1"),
            ("2번 옵션을 시작합니다", "開始選項 2"),
            ("3번 옵션을 시작합니다", "開始選項 3"),
            ("PRC/ROW 펌웨어 설치", "PRC/ROW 韌體安裝"),
            ("ROW(글로벌) 펌웨어 업데이트", "ROW 全球版韌體更新"),
            ("[데이터 초기화]", "[清除資料]"),
            ("[데이터 유지]", "[保留資料]"),
            ("MTK 드라이버 설치를 시작합니다.", "開始安裝 MTK 驅動程式。"),
            ("펌웨어 검사 실패", "韌體檢查失敗"),
            ("유효성 검사", "有效性檢查"),
            ("[1단계]", "[步驟 1]"),
            ("[2단계]", "[步驟 2]"),
            ("[3단계]", "[步驟 3]"),
            ("[4단계]", "[步驟 4]"),
            ("[5단계]", "[步驟 5]"),
            ("[6단계]", "[步驟 6]"),
            ("[7단계]", "[步驟 7]"),
            ("[8단계]", "[步驟 8]"),
            ("작업이 완료되었습니다", "工作已完成"),
            ("국가 코드 재설정 실패", "國家代碼重設失敗"),
            ("올바르지 않은 국가 코드입니다", "國家代碼無效"),
            ("성공", "成功"),
            ("실패", "失敗"),
            ("오류", "錯誤"),
            ("경고", "警告"),
            ("확인 불가", "無法確認"),
            ("완료", "完成"),
        ],
        Spanish => &[
            ("[설정]", "[Configuración]"),
            ("[경고]", "[Advertencia]"),
            ("[추가 옵션]", "[Opciones adicionales]"),
            ("[ROM 옵션]", "[Opciones de ROM]"),
            ("[프로그램]", "[Programa]"),
            ("[완료]", "[Completado]"),
            ("초기 언어 설정", "idioma inicial"),
            ("언어 설정 파일 경로", "ruta del archivo de idioma"),
            ("언어 설정 파일에 저장했습니다", "guardado en el archivo de idioma"),
            ("기준", "origen"),
            ("최신 LPMBox 릴리즈를 확인합니다.", "Comprobando la última versión de LPMBox."),
            ("새 LPMBox 버전을 찾았습니다", "Se encontró una nueva versión de LPMBox"),
            ("릴리즈 ZIP 파일", "archivo ZIP de la versión"),
            ("GitHub Releases 페이지 열기 실패", "No se pudo abrir la página de GitHub Releases"),
            ("1번 옵션을 시작합니다", "Iniciando la opción 1"),
            ("2번 옵션을 시작합니다", "Iniciando la opción 2"),
            ("3번 옵션을 시작합니다", "Iniciando la opción 3"),
            ("PRC/ROW 펌웨어 설치", "instalación de firmware PRC/ROW"),
            ("ROW(글로벌) 펌웨어 업데이트", "actualización de firmware ROW global"),
            ("[데이터 초기화]", "[restablecer datos]"),
            ("[데이터 유지]", "[conservar datos]"),
            ("MTK 드라이버 설치를 시작합니다.", "Iniciando la instalación del controlador MTK."),
            ("펌웨어 검사 실패", "fallo en la comprobación del firmware"),
            ("유효성 검사", "validación"),
            ("[1단계]", "[Paso 1]"),
            ("[2단계]", "[Paso 2]"),
            ("[3단계]", "[Paso 3]"),
            ("[4단계]", "[Paso 4]"),
            ("[5단계]", "[Paso 5]"),
            ("[6단계]", "[Paso 6]"),
            ("[7단계]", "[Paso 7]"),
            ("[8단계]", "[Paso 8]"),
            ("작업이 완료되었습니다", "tarea completada"),
            ("국가 코드 재설정 실패", "falló el restablecimiento del código de país"),
            ("올바르지 않은 국가 코드입니다", "el código de país no es válido"),
            ("성공", "correcto"),
            ("실패", "falló"),
            ("오류", "error"),
            ("경고", "advertencia"),
            ("확인 불가", "no se puede verificar"),
            ("완료", "completado"),
        ],
        Russian => &[
            ("[설정]", "[Настройки]"), ("[경고]", "[Предупреждение]"), ("[추가 옵션]", "[Дополнительные параметры]"), ("[ROM 옵션]", "[Параметры ROM]"), ("[프로그램]", "[Программа]"), ("[완료]", "[Готово]"),
            ("초기 언어 설정", "начальный язык"), ("언어 설정 파일 경로", "путь к файлу языка"), ("기준", "источник"), ("최신 LPMBox 릴리즈를 확인합니다.", "Проверка последнего релиза LPMBox."), ("새 LPMBox 버전을 찾았습니다", "Найдена новая версия LPMBox"), ("릴리즈 ZIP 파일", "ZIP-файл релиза"),
            ("1번 옵션을 시작합니다", "Запуск варианта 1"), ("2번 옵션을 시작합니다", "Запуск варианта 2"), ("3번 옵션을 시작합니다", "Запуск варианта 3"), ("[데이터 초기화]", "[сброс данных]"), ("[데이터 유지]", "[сохранить данные]"),
            ("[1단계]", "[Шаг 1]"), ("[2단계]", "[Шаг 2]"), ("[3단계]", "[Шаг 3]"), ("[4단계]", "[Шаг 4]"), ("[5단계]", "[Шаг 5]"), ("[6단계]", "[Шаг 6]"), ("[7단계]", "[Шаг 7]"), ("[8단계]", "[Шаг 8]"),
            ("성공", "успех"), ("실패", "ошибка"), ("오류", "ошибка"), ("경고", "предупреждение"), ("확인 불가", "невозможно проверить"), ("완료", "готово"),
        ],
        Vietnamese => &[
            ("[설정]", "[Cài đặt]"), ("[경고]", "[Cảnh báo]"), ("[추가 옵션]", "[Tùy chọn bổ sung]"), ("[ROM 옵션]", "[Tùy chọn ROM]"), ("[프로그램]", "[Chương trình]"), ("[완료]", "[Hoàn tất]"),
            ("초기 언어 설정", "ngôn ngữ ban đầu"), ("언어 설정 파일 경로", "đường dẫn tệp ngôn ngữ"), ("기준", "nguồn"), ("최신 LPMBox 릴리즈를 확인합니다.", "Đang kiểm tra bản phát hành LPMBox mới nhất."), ("새 LPMBox 버전을 찾았습니다", "Đã tìm thấy phiên bản LPMBox mới"),
            ("1번 옵션을 시작합니다", "Bắt đầu tùy chọn 1"), ("2번 옵션을 시작합니다", "Bắt đầu tùy chọn 2"), ("3번 옵션을 시작합니다", "Bắt đầu tùy chọn 3"), ("[데이터 초기화]", "[xóa dữ liệu]"), ("[데이터 유지]", "[giữ dữ liệu]"),
            ("[1단계]", "[Bước 1]"), ("[2단계]", "[Bước 2]"), ("[3단계]", "[Bước 3]"), ("[4단계]", "[Bước 4]"), ("[5단계]", "[Bước 5]"), ("[6단계]", "[Bước 6]"), ("[7단계]", "[Bước 7]"), ("[8단계]", "[Bước 8]"),
            ("성공", "thành công"), ("실패", "thất bại"), ("오류", "lỗi"), ("경고", "cảnh báo"), ("확인 불가", "không thể xác minh"), ("완료", "hoàn tất"),
        ],
        Greek => &[("[설정]", "[Ρυθμίσεις]"), ("[경고]", "[Προειδοποίηση]"), ("[추가 옵션]", "[Πρόσθετες επιλογές]"), ("[ROM 옵션]", "[Επιλογές ROM]"), ("[프로그램]", "[Πρόγραμμα]"), ("[완료]", "[Ολοκληρώθηκε]"), ("초기 언어 설정", "αρχική γλώσσα"), ("기준", "πηγή"), ("[1단계]", "[Βήμα 1]"), ("[2단계]", "[Βήμα 2]"), ("[3단계]", "[Βήμα 3]"), ("성공", "επιτυχία"), ("실패", "αποτυχία"), ("오류", "σφάλμα"), ("경고", "προειδοποίηση"), ("완료", "ολοκληρώθηκε")],
        Hindi => &[("[설정]", "[सेटिंग्स]"), ("[경고]", "[चेतावनी]"), ("[추가 옵션]", "[अतिरिक्त विकल्प]"), ("[ROM 옵션]", "[ROM विकल्प]"), ("[프로그램]", "[प्रोग्राम]"), ("[완료]", "[पूरा]"), ("초기 언어 설정", "प्रारंभिक भाषा"), ("기준", "स्रोत"), ("[1단계]", "[चरण 1]"), ("[2단계]", "[चरण 2]"), ("[3단계]", "[चरण 3]"), ("성공", "सफल"), ("실패", "विफल"), ("오류", "त्रुटि"), ("경고", "चेतावनी"), ("완료", "पूरा")],
        Georgian => &[("[설정]", "[პარამეტრები]"), ("[경고]", "[გაფრთხილება]"), ("[추가 옵션]", "[დამატებითი პარამეტრები]"), ("[ROM 옵션]", "[ROM პარამეტრები]"), ("[프로그램]", "[პროგრამა]"), ("[완료]", "[დასრულდა]"), ("초기 언어 설정", "საწყისი ენა"), ("기준", "წყარო"), ("[1단계]", "[ნაბიჯი 1]"), ("[2단계]", "[ნაბიჯი 2]"), ("[3단계]", "[ნაბიჯი 3]"), ("성공", "წარმატება"), ("실패", "ვერ მოხერხდა"), ("오류", "შეცდომა"), ("경고", "გაფრთხილება"), ("완료", "დასრულდა")],
        Dutch => &[("[설정]", "[Instellingen]"), ("[경고]", "[Waarschuwing]"), ("[추가 옵션]", "[Extra opties]"), ("[ROM 옵션]", "[ROM-opties]"), ("[프로그램]", "[Programma]"), ("[완료]", "[Voltooid]"), ("초기 언어 설정", "begintaal"), ("기준", "bron"), ("[1단계]", "[Stap 1]"), ("[2단계]", "[Stap 2]"), ("[3단계]", "[Stap 3]"), ("성공", "geslaagd"), ("실패", "mislukt"), ("오류", "fout"), ("경고", "waarschuwing"), ("완료", "voltooid")],
        Arabic => &[("[설정]", "[الإعدادات]"), ("[경고]", "[تحذير]"), ("[추가 옵션]", "[خيارات إضافية]"), ("[ROM 옵션]", "[خيارات ROM]"), ("[프로그램]", "[البرنامج]"), ("[완료]", "[تم]"), ("초기 언어 설정", "اللغة الأولية"), ("기준", "المصدر"), ("[1단계]", "[الخطوة 1]"), ("[2단계]", "[الخطوة 2]"), ("[3단계]", "[الخطوة 3]"), ("성공", "نجاح"), ("실패", "فشل"), ("오류", "خطأ"), ("경고", "تحذير"), ("완료", "تم")],
        Korean => &[],
    }
}

fn lpm_translate_exact_stage7(lang: LanguageOption, key: &str) -> Option<&'static str> {
    Some(match key {
        "로그가 없습니다." => lpm_lang_text(lang, "No logs.", "Журналы отсутствуют.", "ログがありません。", "沒有日誌。", "Không có nhật ký.", "Δεν υπάρχουν αρχεία καταγραφής.", "कोई लॉग नहीं है।", "ლოგები არ არის.", "Geen logs.", "لا توجد سجلات.", "No hay registros."),
        _ => return None,
    })
}

fn lpm_translate_stage7_phrasewise(lang: LanguageOption, content: String) -> String {
    if lang.is_korean() || !content.chars().any(|c| ('가'..='힣').contains(&c)) {
        return content;
    }

    let mut out = content;
    for (from, to) in lpm_stage7_phrase_pairs(lang) {
        out = out.replace(from, to);
    }
    out
}

fn lpm_stage7_phrase_pairs(lang: LanguageOption) -> &'static [(&'static str, &'static str)] {
    use LanguageOption::*;
    match lang {
        English => &[
            ("[국가 코드 재설정]", "[Country Code Reset]"),
            ("국가 코드 재설정 작업을 시작합니다", "Starting the country code reset task"),
            ("국가 코드 재설정", "Country code reset"),
            ("국가 코드를 선택했습니다", "Selected country code"),
            ("선택한 국가 코드", "Selected country code"),
            ("감지된 국가 코드", "Detected country code"),
            ("국가 코드", "Country code"),
            ("재설정", "reset"),
            ("개는 화면 렌더링 성능을 위해 생략되었습니다. 전체 내용은 로그 저장으로 확인할 수 있습니다.", " lines were omitted for screen rendering performance. You can check the full contents by saving the log."),
            ("이전 로그", "previous log lines"),
        ],
        Russian => &[
            ("[국가 코드 재설정]", "[Сброс кода страны]"),
            ("국가 코드 재설정 작업을 시작합니다", "Запуск сброса кода страны"),
            ("국가 코드 재설정", "Сброс кода страны"),
            ("국가 코드를 선택했습니다", "Выбран код страны"),
            ("선택한 국가 코드", "Выбранный код страны"),
            ("감지된 국가 코드", "Обнаруженный код страны"),
            ("국가 코드", "Код страны"),
            ("재설정", "сброс"),
            ("개는 화면 렌더링 성능을 위해 생략되었습니다. 전체 내용은 로그 저장으로 확인할 수 있습니다.", " строк пропущено для производительности отображения. Полный текст можно проверить, сохранив журнал."),
            ("이전 로그", "предыдущие строки журнала"),
        ],
        Japanese => &[
            ("[국가 코드 재설정]", "[国コードリセット]"),
            ("국가 코드 재설정 작업을 시작합니다", "国コードリセット作業を開始します"),
            ("국가 코드 재설정", "国コードリセット"),
            ("국가 코드를 선택했습니다", "国コードを選択しました"),
            ("선택한 국가 코드", "選択した国コード"),
            ("감지된 국가 코드", "検出された国コード"),
            ("국가 코드", "国コード"),
            ("재설정", "リセット"),
            ("개는 화면 렌더링 성능을 위해 생략되었습니다. 전체 내용은 로그 저장으로 확인할 수 있습니다.", "件の以前のログは表示性能のため省略されました。全文はログ保存で確認できます。"),
            ("이전 로그", "以前のログ"),
        ],
        TraditionalChinese => &[
            ("[국가 코드 재설정]", "[重設國家代碼]"),
            ("국가 코드 재설정 작업을 시작합니다", "開始重設國家代碼工作"),
            ("국가 코드 재설정", "重設國家代碼"),
            ("국가 코드를 선택했습니다", "已選擇國家代碼"),
            ("선택한 국가 코드", "已選擇的國家代碼"),
            ("감지된 국가 코드", "偵測到的國家代碼"),
            ("국가 코드", "國家代碼"),
            ("재설정", "重設"),
            ("개는 화면 렌더링 성능을 위해 생략되었습니다. 전체 내용은 로그 저장으로 확인할 수 있습니다.", "筆已為了畫面渲染效能而省略。完整內容可透過儲存日誌確認。"),
            ("이전 로그", "先前日誌"),
        ],
        Vietnamese => &[
            ("[국가 코드 재설정]", "[Đặt lại mã quốc gia]"),
            ("국가 코드 재설정 작업을 시작합니다", "Bắt đầu tác vụ đặt lại mã quốc gia"),
            ("국가 코드 재설정", "Đặt lại mã quốc gia"),
            ("국가 코드를 선택했습니다", "Đã chọn mã quốc gia"),
            ("선택한 국가 코드", "Mã quốc gia đã chọn"),
            ("감지된 국가 코드", "Mã quốc gia đã phát hiện"),
            ("국가 코드", "Mã quốc gia"),
            ("재설정", "đặt lại"),
            ("개는 화면 렌더링 성능을 위해 생략되었습니다. 전체 내용은 로그 저장으로 확인할 수 있습니다.", " dòng đã được ẩn để tối ưu hiệu năng hiển thị. Có thể xem toàn bộ bằng cách lưu nhật ký."),
            ("이전 로그", "dòng nhật ký trước"),
        ],
        Greek => &[
            ("[국가 코드 재설정]", "[Επαναφορά κωδικού χώρας]"),
            ("국가 코드 재설정 작업을 시작합니다", "Εκκίνηση εργασίας επαναφοράς κωδικού χώρας"),
            ("국가 코드 재설정", "Επαναφορά κωδικού χώρας"),
            ("국가 코드를 선택했습니다", "Επιλέχθηκε κωδικός χώρας"),
            ("선택한 국가 코드", "Επιλεγμένος κωδικός χώρας"),
            ("감지된 국가 코드", "Εντοπισμένος κωδικός χώρας"),
            ("국가 코드", "Κωδικός χώρας"),
            ("재설정", "επαναφορά"),
            ("개는 화면 렌더링 성능을 위해 생략되었습니다. 전체 내용은 로그 저장으로 확인할 수 있습니다.", " γραμμές παραλείφθηκαν για καλύτερη απόδοση εμφάνισης. Μπορείτε να δείτε το πλήρες περιεχόμενο αποθηκεύοντας το αρχείο καταγραφής."),
            ("이전 로그", "προηγούμενες γραμμές καταγραφής"),
        ],
        Hindi => &[
            ("[국가 코드 재설정]", "[देश कोड रीसेट]"),
            ("국가 코드 재설정 작업을 시작합니다", "देश कोड रीसेट कार्य शुरू हो रहा है"),
            ("국가 코드 재설정", "देश कोड रीसेट"),
            ("국가 코드를 선택했습니다", "देश कोड चुना गया"),
            ("선택한 국가 코드", "चुना गया देश कोड"),
            ("감지된 국가 코드", "पहचाना गया देश कोड"),
            ("국가 코드", "देश कोड"),
            ("재설정", "रीसेट"),
            ("개는 화면 렌더링 성능을 위해 생략되었습니다. 전체 내용은 로그 저장으로 확인할 수 있습니다.", " पंक्तियाँ स्क्रीन रेंडरिंग प्रदर्शन के लिए छोड़ी गईं। पूरा 내용 लॉग सहेजकर देखा जा सकता है."),
            ("이전 로그", "पिछली लॉग पंक्तियाँ"),
        ],
        Georgian => &[
            ("[국가 코드 재설정]", "[ქვეყნის კოდის გადაყენება]"),
            ("국가 코드 재설정 작업을 시작합니다", "იწყება ქვეყნის კოდის გადაყენება"),
            ("국가 코드 재설정", "ქვეყნის კოდის გადაყენება"),
            ("국가 코드를 선택했습니다", "ქვეყნის კოდი არჩეულია"),
            ("선택한 국가 코드", "არჩეული ქვეყნის კოდი"),
            ("감지된 국가 코드", "აღმოჩენილი ქვეყნის კოდი"),
            ("국가 코드", "ქვეყნის კოდი"),
            ("재설정", "გადაყენება"),
            ("개는 화면 렌더링 성능을 위해 생략되었습니다. 전체 내용은 로그 저장으로 확인할 수 있습니다.", " ხაზი გამოტოვებულია ეკრანის სწრაფი ჩვენებისთვის. სრული შინაარსი შეგიძლიათ ნახოთ ლოგის შენახვით."),
            ("이전 로그", "წინა ლოგის ხაზები"),
        ],
        Dutch => &[
            ("[국가 코드 재설정]", "[Landcode resetten]"),
            ("국가 코드 재설정 작업을 시작합니다", "Taak voor landcode resetten wordt gestart"),
            ("국가 코드 재설정", "Landcode resetten"),
            ("국가 코드를 선택했습니다", "Landcode geselecteerd"),
            ("선택한 국가 코드", "Geselecteerde landcode"),
            ("감지된 국가 코드", "Gedetecteerde landcode"),
            ("국가 코드", "Landcode"),
            ("재설정", "resetten"),
            ("개는 화면 렌더링 성능을 위해 생략되었습니다. 전체 내용은 로그 저장으로 확인할 수 있습니다.", " regels zijn overgeslagen voor betere schermprestaties. De volledige inhoud is beschikbaar door het logboek op te slaan."),
            ("이전 로그", "vorige logregels"),
        ],
        Arabic => &[
            ("[국가 코드 재설정]", "[إعادة ضبط رمز البلد]"),
            ("국가 코드 재설정 작업을 시작합니다", "بدء مهمة إعادة ضبط رمز البلد"),
            ("국가 코드 재설정", "إعادة ضبط رمز البلد"),
            ("국가 코드를 선택했습니다", "تم اختيار رمز البلد"),
            ("선택한 국가 코드", "رمز البلد المختار"),
            ("감지된 국가 코드", "رمز البلد المكتشف"),
            ("국가 코드", "رمز البلد"),
            ("재설정", "إعادة ضبط"),
            ("개는 화면 렌더링 성능을 위해 생략되었습니다. 전체 내용은 로그 저장으로 확인할 수 있습니다.", " سطرًا تم إخفاؤها لتحسين أداء عرض الشاشة. يمكن عرض المحتوى الكامل بحفظ السجل."),
            ("이전 로그", "أسطر السجل السابقة"),
        ],
        Spanish => &[
            ("[국가 코드 재설정]", "[Restablecer código de país]"),
            ("국가 코드 재설정 작업을 시작합니다", "Iniciando tarea de restablecimiento del código de país"),
            ("국가 코드 재설정", "Restablecer código de país"),
            ("국가 코드를 선택했습니다", "Código de país seleccionado"),
            ("선택한 국가 코드", "Código de país seleccionado"),
            ("감지된 국가 코드", "Código de país detectado"),
            ("국가 코드", "Código de país"),
            ("재설정", "restablecer"),
            ("개는 화면 렌더링 성능을 위해 생략되었습니다. 전체 내용은 로그 저장으로 확인할 수 있습니다.", " líneas se omitieron para mejorar el rendimiento de renderizado. Puede ver el contenido completo guardando el registro."),
            ("이전 로그", "líneas de registro anteriores"),
        ],
        Korean => &[],
    }
}

fn lpm_lang_text(
    lang: LanguageOption,
    en: &'static str,
    ru: &'static str,
    ja: &'static str,
    zh: &'static str,
    vi: &'static str,
    el: &'static str,
    hi: &'static str,
    ka: &'static str,
    nl: &'static str,
    ar: &'static str,
    es: &'static str,
) -> &'static str {
    match lang {
        LanguageOption::English => en,
        LanguageOption::Russian => ru,
        LanguageOption::Japanese => ja,
        LanguageOption::TraditionalChinese => zh,
        LanguageOption::Vietnamese => vi,
        LanguageOption::Greek => el,
        LanguageOption::Hindi => hi,
        LanguageOption::Georgian => ka,
        LanguageOption::Dutch => nl,
        LanguageOption::Arabic => ar,
        LanguageOption::Spanish => es,
        LanguageOption::Korean => en,
    }
}


fn lpm_rom_routine_ui_text(lang: LanguageOption, key: &'static str) -> &'static str {
    if lang.is_korean() {
        return key;
    }

    match key {
        "PRC ↔ ROW 설치" => lpm_lang_text(lang, "PRC ↔ ROW Install", "Установка PRC ↔ ROW", "PRC ↔ ROWインストール", "PRC ↔ ROW 安裝", "Cài đặt PRC ↔ ROW", "Εγκατάσταση PRC ↔ ROW", "PRC ↔ ROW इंस्टॉल", "PRC ↔ ROW ინსტალაცია", "PRC ↔ ROW installeren", "PRC ↔ ROW تثبيت", "Instalar PRC ↔ ROW"),
        "ROW(글로벌롬) 업데이트" => lpm_lang_text(lang, "ROW Update", "Обновление ROW", "ROWアップデート", "ROW 更新", "Cập nhật ROW", "Ενημέρωση ROW", "ROW अपडेट", "ROW განახლება", "ROW-update", "تحديث ROW", "Actualización ROW"),
        "기기 복구" => lpm_lang_text(lang, "Device Recovery", "Восстановление устройства", "デバイス復旧", "裝置修復", "Khôi phục thiết bị", "Ανάκτηση συσκευής", "डिवाइस रिकवरी", "მოწყობილობის აღდგენა", "Apparaatherstel", "استرداد الجهاز", "Recuperación"),
        "기기가 연결되어 있지 않아 실행할 수 없습니다." => lpm_lang_text(lang, "Cannot run because the device is not connected.", "Невозможно выполнить: устройство не подключено.", "デバイスが接続されていないため実行できません。", "裝置未連接，無法執行。", "Không thể chạy vì thiết bị chưa được kết nối.", "Δεν μπορεί να εκτελεστεί επειδή η συσκευή δεν είναι συνδεδεμένη.", "डिवाइस कनेक्ट नहीं है, इसलिए चलाया नहीं जा सकता।", "ვერ შესრულდება, რადგან მოწყობილობა დაკავშირებული არ არის.", "Kan niet uitvoeren omdat het apparaat niet is aangesloten.", "لا يمكن التشغيل لأن الجهاز غير متصل.", "No se puede ejecutar porque el dispositivo no está conectado."),
        "ROW(글로벌롬) 버전을 업데이트합니다." => lpm_lang_text(lang, "Update the ROW ROM version.", "Обновить версию ROW ROM.", "ROW ROMを更新します。", "更新 ROW ROM 版本。", "Cập nhật phiên bản ROW ROM.", "Ενημέρωση έκδοσης ROW ROM.", "ROW ROM संस्करण अपडेट करें।", "განაახლეთ ROW ROM ვერსია.", "Werk de ROW-ROM bij.", "تحديث إصدار ROW ROM.", "Actualiza la ROM ROW."),
        "기기가 PRC(중국 내수롬)이므로 불가능 합니다." => lpm_lang_text(lang, "Unavailable because the device is PRC ROM.", "Недоступно, потому что устройство использует PRC ROM.", "端末がPRC ROMのため実行できません。", "裝置為 PRC ROM，無法執行。", "Không khả dụng vì thiết bị đang dùng PRC ROM.", "Δεν είναι διαθέσιμο επειδή η συσκευή χρησιμοποιεί PRC ROM.", "डिवाइस PRC ROM है, इसलिए उपलब्ध नहीं है।", "მიუწვდომელია, რადგან მოწყობილობა PRC ROM-ზეა.", "Niet beschikbaar omdat het apparaat PRC-ROM gebruikt.", "غير متاح لأن الجهاز يستخدم PRC ROM.", "No disponible porque el dispositivo usa ROM PRC."),
        "image 폴더 유형이 PRC(중국 내수롬)이므로 불가능 합니다." => lpm_lang_text(lang, "Unavailable because the image folder is PRC ROM.", "Недоступно, потому что папка image содержит PRC ROM.", "imageフォルダーがPRC ROMのため実行できません。", "image 資料夾為 PRC ROM，無法執行。", "Không khả dụng vì thư mục image là PRC ROM.", "Δεν είναι διαθέσιμο επειδή ο φάκελος image είναι PRC ROM.", "image फ़ोल्डर PRC ROM है, इसलिए उपलब्ध नहीं है।", "მიუწვდომელია, რადგან image საქაღალდე PRC ROM-ია.", "Niet beschikbaar omdat de image-map PRC-ROM bevat.", "غير متاح لأن مجلد image يحتوي على PRC ROM.", "No disponible porque la carpeta image contiene ROM PRC."),
        "ROM 타입을 확인할 수 없어 업데이트 실행을 보류합니다." => lpm_lang_text(lang, "Update is on hold because the ROM type cannot be verified.", "Обновление отложено, потому что тип ROM не удалось проверить.", "ROMタイプを確認できないため、更新を保留します。", "無法確認 ROM 類型，因此暫停更新。", "Tạm dừng cập nhật vì không thể xác minh loại ROM.", "Η ενημέρωση έχει τεθεί σε αναμονή επειδή δεν μπορεί να επαληθευτεί ο τύπος ROM.", "ROM प्रकार सत्यापित नहीं हो सका, इसलिए अपडेट रोक दिया गया है।", "განახლება შეჩერებულია, რადგან ROM-ის ტიპი ვერ მოწმდება.", "Update is gepauzeerd omdat het ROM-type niet kan worden gecontroleerd.", "تم تعليق التحديث لأنه لا يمكن التحقق من نوع ROM.", "La actualización está en espera porque no se puede verificar el tipo de ROM."),
        "기기가 연결되어 있지 않아 설치를 실행할 수 없습니다." => lpm_lang_text(lang, "Cannot install because the device is not connected.", "Невозможно установить: устройство не подключено.", "デバイスが接続されていないためインストールできません。", "裝置未連接，無法安裝。", "Không thể cài đặt vì thiết bị chưa được kết nối.", "Δεν μπορεί να γίνει εγκατάσταση επειδή η συσκευή δεν είναι συνδεδεμένη.", "डिवाइस कनेक्ट नहीं है, इसलिए इंस्टॉल नहीं किया जा सकता।", "ვერ დაინსტალირდება, რადგან მოწყობილობა დაკავშირებული არ არის.", "Kan niet installeren omdat het apparaat niet is aangesloten.", "لا يمكن التثبيت لأن الجهاز غير متصل.", "No se puede instalar porque el dispositivo no está conectado."),
        "ROW(글로벌롬)기기에 PRC(중국 내수롬)을 설치합니다." => lpm_lang_text(lang, "Install PRC ROM on the ROW device.", "Установить PRC ROM на устройство ROW.", "ROW端末にPRC ROMをインストールします。", "在 ROW 裝置上安裝 PRC ROM。", "Cài PRC ROM trên thiết bị ROW.", "Εγκατάσταση PRC ROM στη συσκευή ROW.", "ROW डिवाइस पर PRC ROM इंस्टॉल करें।", "ROW მოწყობილობაზე PRC ROM-ის დაყენება.", "Installeer PRC-ROM op het ROW-apparaat.", "تثبيت PRC ROM على جهاز ROW.", "Instala PRC ROM en el dispositivo ROW."),
        "PRC(중국 내수롬)기기에 ROW(글로벌롬)을 설치합니다." => lpm_lang_text(lang, "Install ROW ROM on the PRC device.", "Установить ROW ROM на устройство PRC.", "PRC端末にROW ROMをインストールします。", "在 PRC 裝置上安裝 ROW ROM。", "Cài ROW ROM trên thiết bị PRC.", "Εγκατάσταση ROW ROM στη συσκευή PRC.", "PRC डिवाइस पर ROW ROM इंस्टॉल करें।", "PRC მოწყობილობაზე ROW ROM-ის დაყენება.", "Installeer ROW-ROM op het PRC-apparaat.", "تثبيت ROW ROM على جهاز PRC.", "Instala ROW ROM en el dispositivo PRC."),
        "ROW(글로벌롬) 업데이트로 진행해 주세요." => lpm_lang_text(lang, "Please use ROW Update.", "Используйте обновление ROW.", "ROWアップデートを選択してください。", "請使用 ROW 更新。", "Hãy dùng cập nhật ROW.", "Χρησιμοποιήστε την ενημέρωση ROW.", "ROW अपडेट का उपयोग करें।", "გამოიყენეთ ROW განახლება.", "Gebruik ROW-update.", "استخدم تحديث ROW.", "Use la actualización ROW."),
        "PRC(중국 내수롬) 업데이트는 지원하지 않습니다." => lpm_lang_text(lang, "PRC ROM update is not supported.", "Обновление PRC ROM не поддерживается.", "PRC ROMの更新はサポートされていません。", "不支援 PRC ROM 更新。", "Không hỗ trợ cập nhật PRC ROM.", "Δεν υποστηρίζεται ενημέρωση PRC ROM.", "PRC ROM अपडेट समर्थित नहीं है।", "PRC ROM-ის განახლება მხარდაჭერილი არ არის.", "PRC-ROM-update wordt niet ondersteund.", "تحديث PRC ROM غير مدعوم.", "La actualización de ROM PRC no es compatible."),
        "ROM 타입을 확인할 수 없어 설치 실행을 보류합니다." => lpm_lang_text(lang, "Installation is on hold because the ROM type cannot be verified.", "Установка отложена, потому что тип ROM не удалось проверить.", "ROMタイプを確認できないため、インストールを保留します。", "無法確認 ROM 類型，因此暫停安裝。", "Tạm dừng cài đặt vì không thể xác minh loại ROM.", "Η εγκατάσταση έχει τεθεί σε αναμονή επειδή δεν μπορεί να επαληθευτεί ο τύπος ROM.", "ROM प्रकार सत्यापित नहीं हो सका, इसलिए इंस्टॉल रोक दिया गया है।", "ინსტალაცია შეჩერებულია, რადგან ROM-ის ტიპი ვერ მოწმდება.", "Installatie is gepauzeerd omdat het ROM-type niet kan worden gecontroleerd.", "تم تعليق التثبيت لأنه لا يمكن التحقق من نوع ROM.", "La instalación está en espera porque no se puede verificar el tipo de ROM."),
        "기기가 켜지지 않거나, 무한 재부팅 등 다양한 오류를 고칩니다." => lpm_lang_text(lang, "Fixes issues such as a device that will not turn on or endless rebooting.", "Исправляет проблемы, например устройство не включается или бесконечно перезагружается.", "起動しない・無限再起動などの問題を修復します。", "修復無法開機、循環重啟等各種問題。", "Sửa các lỗi như không bật được máy hoặc lặp khởi động.", "Διορθώνει προβλήματα όπως συσκευή που δεν ανοίγει ή κάνει συνεχείς επανεκκινήσεις.", "डिवाइस चालू न होना या अनंत रीबूट जैसी समस्याएं ठीक करें।", "ასწორებს პრობლემებს, როგორიცაა არ ჩართვა ან უსასრულო გადატვირთვა.", "Lost problemen op zoals niet inschakelen of eindeloos herstarten.", "يصلح مشكلات مثل عدم تشغيل الجهاز أو إعادة التشغيل المستمرة.", "Corrige fallos como que el dispositivo no encienda o se reinicie sin parar."),
        "PRC ↔ ROW 설치 루틴" => lpm_lang_text(lang, "PRC ↔ ROW Install", "Установка PRC ↔ ROW", "PRC ↔ ROWインストール", "PRC ↔ ROW 安裝", "Cài đặt PRC ↔ ROW", "Εγκατάσταση PRC ↔ ROW", "PRC ↔ ROW इंस्टॉल", "PRC ↔ ROW ინსტალაცია", "PRC ↔ ROW installeren", "PRC ↔ ROW تثبيت", "Instalar PRC ↔ ROW"),
        "ROW(글로벌롬) 업데이트 루틴" => lpm_lang_text(lang, "ROW Update", "Обновление ROW", "ROWアップデート", "ROW 更新", "Cập nhật ROW", "Ενημέρωση ROW", "ROW अपडेट", "ROW განახლება", "ROW-update", "تحديث ROW", "Actualización ROW"),
        "기기 복구 루틴" => lpm_lang_text(lang, "Device Recovery", "Восстановление устройства", "デバイス復旧", "裝置修復", "Khôi phục thiết bị", "Ανάκτηση συσκευής", "डिवाइस रिकवरी", "მოწყობილობის აღდგენა", "Apparaatherstel", "استرداد الجهاز", "Recuperación"),
        "중국 내수롬과 글로벌롬을 자유롭게 변경 가능" => lpm_lang_text(lang, "Switch freely between PRC and ROW ROM.", "Свободное переключение между PRC и ROW ROM.", "PRC ROMとROW ROMを自由に切り替えられます。", "可自由切換 PRC 與 ROW ROM。", "Chuyển đổi tự do giữa PRC và ROW ROM.", "Ελεύθερη εναλλαγή μεταξύ PRC και ROW ROM.", "PRC और ROW ROM के बीच स्वतंत्र रूप से स्विच करें।", "თავისუფლად გადართეთ PRC და ROW ROM-ს შორის.", "Schakel vrij tussen PRC- en ROW-ROM.", "التبديل بحرية بين PRC و ROW ROM.", "Cambia libremente entre ROM PRC y ROW."),
        "데이터 초기화가 필수이기 때문에" => lpm_lang_text(lang, "Data wipe is required.", "Требуется сброс данных.", "データ初期化が必須です。", "必須恢復原廠設定。", "Bắt buộc xóa dữ liệu.", "Απαιτείται διαγραφή δεδομένων.", "डेटा वाइप आवश्यक है।", "მონაცემების წაშლა აუცილებელია.", "Gegevens wissen is vereist.", "يلزم مسح البيانات.", "Es obligatorio borrar los datos."),
        "시작하기 전 데이터 백업 후 진행해주세요." => lpm_lang_text(lang, "Back up your data before starting.", "Перед началом создайте резервную копию данных.", "開始前にデータをバックアップしてください。", "開始前請先備份資料。", "Sao lưu dữ liệu trước khi bắt đầu.", "Δημιουργήστε αντίγραφο πριν ξεκινήσετε.", "शुरू करने से पहले डेटा का बैकअप लें।", "დაწყებამდე შექმენით მონაცემების სარეზერვო ასლი.", "Maak een back-up voordat u begint.", "انسخ بياناتك احتياطيًا قبل البدء.", "Haz una copia de los datos antes de empezar."),
        "글로벌롬 펌웨어 버전을 업데이트합니다." => lpm_lang_text(lang, "Update the ROW firmware version.", "Обновить версию прошивки ROW.", "ROWファームウェアを更新します。", "更新 ROW 韌體版本。", "Cập nhật phiên bản firmware ROW.", "Ενημέρωση έκδοσης firmware ROW.", "ROW firmware संस्करण अपडेट करें।", "განაახლეთ ROW firmware-ის ვერსია.", "Werk de ROW-firmware bij.", "تحديث إصدار firmware ROW.", "Actualiza el firmware ROW."),
        "기기에 설치된 버전보다 낮을 경우/초기화 O." => lpm_lang_text(lang, "If lower than the installed version: wipe required.", "Если ниже установленной версии: требуется сброс.", "インストール済みより低い場合：初期化あり。", "低於已安裝版本時：需要清除資料。", "Nếu thấp hơn phiên bản đã cài: cần xóa dữ liệu.", "Αν είναι χαμηλότερη από την εγκατεστημένη έκδοση: απαιτείται διαγραφή.", "इंस्टॉल संस्करण से कम होने पर: वाइप आवश्यक।", "თუ დაყენებულ ვერსიაზე დაბალია: საჭიროა წაშლა.", "Als lager dan geïnstalleerd: wissen vereist.", "إذا كان أقل من الإصدار المثبت: يلزم المسح.", "Si es inferior a la versión instalada: requiere borrado."),
        "기기에 설치된 버전보다 높은 경우/초기화 X." => lpm_lang_text(lang, "If higher than the installed version: no wipe.", "Если выше установленной версии: сброс не нужен.", "インストール済みより高い場合：初期化なし。", "高於已安裝版本時：不清除資料。", "Nếu cao hơn phiên bản đã cài: không xóa dữ liệu.", "Αν είναι υψηλότερη από την εγκατεστημένη έκδοση: χωρίς διαγραφή.", "इंस्टॉल संस्करण से अधिक होने पर: वाइप नहीं।", "თუ დაყენებულ ვერსიაზე მაღალია: წაშლა არ არის საჭირო.", "Als hoger dan geïnstalleerd: niet wissen.", "إذا كان أعلى من الإصدار المثبت: لا يلزم المسح.", "Si es superior a la versión instalada: sin borrado."),
        "'펌웨어 설치 실패' 등 기기를 복구합니다." => lpm_lang_text(lang, "Recover from issues such as firmware installation failure.", "Восстановление после ошибок, например сбоя установки прошивки.", "ファームウェア導入失敗などから復旧します。", "修復韌體安裝失敗等問題。", "Khôi phục các lỗi như cài firmware thất bại.", "Ανάκτηση από προβλήματα όπως αποτυχία εγκατάστασης firmware.", "firmware इंस्टॉल विफलता जैसी समस्याओं से रिकवर करें।", "აღდგენა firmware-ის დაყენების შეცდომის მსგავსი პრობლემებისგან.", "Herstel van problemen zoals firmware-installatiefouten.", "الاسترداد من مشكلات مثل فشل تثبيت firmware.", "Recupera fallos como errores al instalar firmware."),
        _ => lpm_translate_exact_stage10(lang, key).unwrap_or(key),
    }
}

fn lpm_translate_exact_stage3(lang: LanguageOption, key: &str) -> Option<&'static str> {
    Some(match key {
        "proinfo 백업" => lpm_lang_text(lang, "proinfo Backup", "Резервная копия proinfo", "proinfoバックアップ", "proinfo 備份", "Sao lưu proinfo", "Αντίγραφο proinfo", "proinfo बैकअप", "proinfo სარეზერვო ასლი", "proinfo-back-up", "نسخ proinfo احتياطيًا", "Copia de proinfo"),
        "proinfo 파티션을 백업합니다." => lpm_lang_text(lang, "Back up the proinfo partition.", "Создать резервную копию раздела proinfo.", "proinfoパーティションをバックアップします。", "備份 proinfo 分割區。", "Sao lưu phân vùng proinfo.", "Δημιουργήστε αντίγραφο του διαμερίσματος proinfo.", "proinfo partition का बैकअप लें।", "შექმენით proinfo დანაყოფის სარეზერვო ასლი.", "Maak een back-up van de proinfo-partitie.", "انسخ قسم proinfo احتياطيًا.", "Haz una copia de la partición proinfo."),
        "샤오신패드 펌웨어 다운로드 페이지로 이동합니다." => lpm_lang_text(lang, "Open the Xiaoxin Pad firmware download page.", "Открыть страницу загрузки прошивки Xiaoxin Pad.", "Xiaoxin Padファームウェアダウンロードページを開きます。", "開啟 Xiaoxin Pad 韌體下載頁面。", "Mở trang tải firmware Xiaoxin Pad.", "Ανοίξτε τη σελίδα λήψης firmware Xiaoxin Pad.", "Xiaoxin Pad firmware download page खोलें।", "გახსენით Xiaoxin Pad firmware-ის ჩამოტვირთვის გვერდი.", "Open de firmwaredownloadpagina van de Xiaoxin Pad.", "افتح صفحة تنزيل firmware لجهاز Xiaoxin Pad.", "Abre la página de descarga de firmware de Xiaoxin Pad."),
        "PRC/ROW 설치 안내" => lpm_lang_text(lang, "PRC/ROW installation guide", "Руководство по установке PRC/ROW", "PRC/ROWインストール案内", "PRC/ROW 安裝指南", "Hướng dẫn cài đặt PRC/ROW", "Οδηγός εγκατάστασης PRC/ROW", "PRC/ROW installation guide", "PRC/ROW ინსტალაციის გზამკვლევი", "PRC/ROW-installatiegids", "دليل تثبيت PRC/ROW", "Guía de instalación PRC/ROW"),
        "PRC ↔ ROW 전환 작업입니다." => lpm_lang_text(lang, "This switches between PRC and ROW.", "Это переключение между PRC и ROW.", "PRCとROWを切り替える作業です。", "這是 PRC 與 ROW 之間的切換作業。", "Đây là thao tác chuyển đổi giữa PRC và ROW.", "Αυτή η εργασία αλλάζει μεταξύ PRC και ROW.", "यह PRC और ROW के बीच स्विच करता है।", "ეს PRC და ROW-ს შორის გადართვის მოქმედებაა.", "Dit schakelt tussen PRC en ROW.", "هذه عملية تبديل بين PRC و ROW.", "Esta operación cambia entre PRC y ROW."),
        "데이터가 초기화되므로 백업 후 진행해주세요." => lpm_lang_text(lang, "Your data will be wiped, so back it up before continuing.", "Данные будут удалены, поэтому сначала создайте резервную копию.", "データが初期化されるため、バックアップ後に進めてください。", "資料會被清除，請先備份再繼續。", "Dữ liệu sẽ bị xóa, hãy sao lưu trước khi tiếp tục.", "Τα δεδομένα θα διαγραφούν, δημιουργήστε πρώτα αντίγραφο.", "डेटा मिट जाएगा, आगे बढ़ने से पहले बैकअप लें।", "მონაცემები წაიშლება, გაგრძელებამდე შექმენით სარეზერვო ასლი.", "Gegevens worden gewist; maak eerst een back-up.", "سيتم مسح البيانات، لذلك انسخها احتياطيًا قبل المتابعة.", "Se borrarán los datos; haz una copia antes de continuar."),
        "ROW(글로벌롬) 업데이트 안내" => lpm_lang_text(lang, "ROW (Global ROM) update guide", "Руководство по обновлению ROW (глобальная ROM)", "ROW（グローバルROM）アップデート案内", "ROW（全球版 ROM）更新指南", "Hướng dẫn cập nhật ROW (ROM quốc tế)", "Οδηγός ενημέρωσης ROW (Global ROM)", "ROW (Global ROM) update guide", "ROW (გლობალური ROM) განახლების გზამკვლევი", "ROW (Global ROM)-updategids", "دليل تحديث ROW (الروم العالمي)", "Guía de actualización ROW (ROM global)"),
        "ROW 글로벌롬 업데이트 작업입니다." => lpm_lang_text(lang, "This updates the ROW global ROM.", "Это обновляет глобальную ROM ROW.", "ROWグローバルROMを更新する作業です。", "這是更新 ROW 全球版 ROM 的作業。", "Đây là thao tác cập nhật ROM quốc tế ROW.", "Αυτή η εργασία ενημερώνει το ROW Global ROM.", "यह ROW global ROM को update करता है।", "ეს ROW გლობალური ROM-ის განახლებაა.", "Dit werkt de ROW Global ROM bij.", "هذه عملية تحديث ROW Global ROM.", "Esta operación actualiza la ROM global ROW."),
        "데이터 삭제 없이 버전을 업데이트합니다." => lpm_lang_text(lang, "Update the version without deleting data.", "Обновить версию без удаления данных.", "データを削除せずにバージョンを更新します。", "在不刪除資料的情況下更新版本。", "Cập nhật phiên bản mà không xóa dữ liệu.", "Ενημερώστε την έκδοση χωρίς διαγραφή δεδομένων.", "डेटा हटाए बिना version update करें।", "განაახლეთ ვერსია მონაცემების წაშლის გარეშე.", "Werk de versie bij zonder gegevens te verwijderen.", "حدّث الإصدار بدون حذف البيانات.", "Actualiza la versión sin eliminar datos."),
        "기기 복구 안내" => lpm_lang_text(lang, "Device recovery guide", "Руководство по восстановлению устройства", "デバイス復旧案内", "裝置修復指南", "Hướng dẫn khôi phục thiết bị", "Οδηγός ανάκτησης συσκευής", "Device recovery guide", "მოწყობილობის აღდგენის გზამკვლევი", "Apparaatherstelgids", "دليل استرداد الجهاز", "Guía de recuperación del dispositivo"),
        "기기가 켜지지 않거나 무한 재부팅 상태일 때 사용하는 복구 작업입니다." => lpm_lang_text(lang, "Use this when the device will not turn on or keeps rebooting endlessly.", "Используйте это, если устройство не включается или бесконечно перезагружается.", "デバイスが起動しない、または無限再起動する場合に使用します。", "裝置無法開機或無限重啟時使用。", "Dùng khi thiết bị không bật hoặc khởi động lại liên tục.", "Χρησιμοποιήστε το όταν η συσκευή δεν ανοίγει ή επανεκκινείται συνεχώς.", "जब device चालू न हो या बार-बार reboot हो, इसका उपयोग करें।", "გამოიყენეთ, როცა მოწყობილობა არ ირთვება ან უსასრულოდ იტვირთება.", "Gebruik dit als het apparaat niet opstart of eindeloos herstart.", "استخدمه عندما لا يعمل الجهاز أو يعيد التشغيل بلا توقف.", "Úsalo cuando el dispositivo no enciende o se reinicia sin parar."),
        "현재 단계에서는 UI만 준비하며, 실제 복구 루틴 연결은 추후 진행합니다." => lpm_lang_text(lang, "At this stage, only the UI is prepared; the actual recovery routine will be connected later.", "На этом этапе подготовлен только UI; фактическая процедура восстановления будет подключена позже.", "現段階ではUIのみ準備し、実際の復旧ルーチン接続は後で行います。", "目前只準備 UI，實際修復流程稍後連接。", "Ở bước này chỉ chuẩn bị UI; quy trình khôi phục thực tế sẽ được kết nối sau.", "Σε αυτό το στάδιο προετοιμάζεται μόνο το UI· η πραγματική ρουτίνα ανάκτησης θα συνδεθεί αργότερα.", "इस चरण में केवल UI तैयार है; वास्तविक recovery routine बाद में जोड़ा जाएगा।", "ამ ეტაპზე მზადდება მხოლოდ UI; აღდგენის რეალური რუტინა მოგვიანებით დაემატება.", "In deze fase is alleen de UI voorbereid; de herstelroutine wordt later gekoppeld.", "في هذه المرحلة تم إعداد الواجهة فقط؛ سيتم ربط روتين الاسترداد لاحقًا.", "En esta etapa solo se prepara la interfaz; la rutina real se conectará más tarde."),
        "SPFlashToolV6 readback으로 proinfo 파티션을 추출합니다." => lpm_lang_text(lang, "Extract the proinfo partition using SPFlashToolV6 readback.", "Извлечь раздел proinfo через readback SPFlashToolV6.", "SPFlashToolV6 readbackでproinfoパーティションを抽出します。", "使用 SPFlashToolV6 readback 擷取 proinfo 分割區。", "Trích xuất phân vùng proinfo bằng SPFlashToolV6 readback.", "Εξαγωγή του διαμερίσματος proinfo με readback SPFlashToolV6.", "SPFlashToolV6 readback से proinfo partition निकालें।", "ამოიღეთ proinfo დანაყოფი SPFlashToolV6 readback-ით.", "Extraheer de proinfo-partitie met SPFlashToolV6 readback.", "استخرج قسم proinfo باستخدام readback في SPFlashToolV6.", "Extrae la partición proinfo con readback de SPFlashToolV6."),
        "proinfo 백업 시작" => lpm_lang_text(lang, "Start proinfo backup", "Начать резервное копирование proinfo", "proinfoバックアップ開始", "開始 proinfo 備份", "Bắt đầu sao lưu proinfo", "Έναρξη αντιγράφου proinfo", "proinfo backup शुरू करें", "proinfo სარეზერვოს დაწყება", "proinfo-back-up starten", "بدء نسخ proinfo احتياطيًا", "Iniciar copia de proinfo"),
        "ADB / Fastboot / PreLoader / SPFlashToolV6 진행 상태" => lpm_lang_text(lang, "ADB / Fastboot / PreLoader / SPFlashToolV6 progress status", "Состояние ADB / Fastboot / PreLoader / SPFlashToolV6", "ADB / Fastboot / PreLoader / SPFlashToolV6 進行状況", "ADB / Fastboot / PreLoader / SPFlashToolV6 進度狀態", "Trạng thái ADB / Fastboot / PreLoader / SPFlashToolV6", "Κατάσταση ADB / Fastboot / PreLoader / SPFlashToolV6", "ADB / Fastboot / PreLoader / SPFlashToolV6 progress status", "ADB / Fastboot / PreLoader / SPFlashToolV6 მიმდინარეობა", "ADB / Fastboot / PreLoader / SPFlashToolV6-voortgang", "حالة تقدم ADB / Fastboot / PreLoader / SPFlashToolV6", "Estado de progreso de ADB / Fastboot / PreLoader / SPFlashToolV6"),
        "설치 실패 / 무한 재부팅 / Red State 복구" => lpm_lang_text(lang, "Recover from install failure / endless reboot / Red State", "Восстановление после ошибки установки / бесконечной перезагрузки / Red State", "インストール失敗／無限再起動／Red Stateを復旧", "修復安裝失敗／無限重啟／Red State", "Khôi phục lỗi cài đặt / khởi động lại liên tục / Red State", "Ανάκτηση από αποτυχία εγκατάστασης / ατελείωτη επανεκκίνηση / Red State", "Install failure / endless reboot / Red State से recover करें", "აღდგენა ინსტალაციის შეცდომიდან / უსასრულო გადატვირთვიდან / Red State-დან", "Herstel installatiefout / eindeloze herstart / Red State", "استرداد فشل التثبيت / إعادة التشغيل المستمرة / Red State", "Recuperar fallo de instalación / reinicio infinito / Red State"),
        "현재 버전의 문제를\n해결하고 업그레이드한 파일을\n감지했습니다." => lpm_lang_text(lang, "A file that fixes issues in the current version and upgrades it was detected.", "Обнаружен файл, который исправляет проблемы текущей версии и обновляет её.", "現在のバージョンの問題を修正し、アップグレードするファイルを検出しました。", "已偵測到可修正目前版本問題並升級的檔案。", "Đã phát hiện tệp khắc phục lỗi phiên bản hiện tại và nâng cấp chương trình.", "Εντοπίστηκε αρχείο που διορθώνει προβλήματα της τρέχουσας έκδοσης και την αναβαθμίζει.", "Current version की समस्याएँ ठीक करके upgrade करने वाली file मिली है।", "ნაპოვნია ფაილი, რომელიც აგვარებს მიმდინარე ვერსიის პრობლემებს და ახორციელებს განახლებას.", "Er is een bestand gevonden dat problemen in de huidige versie oplost en deze bijwerkt.", "تم اكتشاف ملف يصلح مشاكل الإصدار الحالي ويقوم بترقيته.", "Se detectó un archivo que corrige problemas de la versión actual y la actualiza."),
        _ => return None,
    })
}

fn lpm_translate_stage3_phrasewise(lang: LanguageOption, content: String) -> String {
    if lang.is_korean() || !content.chars().any(|c| ('가'..='힣').contains(&c)) {
        return content;
    }

    let mut out = content;
    for (from, to) in lpm_stage3_phrase_pairs(lang) {
        out = out.replace(from, to);
    }
    out
}

fn lpm_stage3_phrase_pairs(lang: LanguageOption) -> &'static [(&'static str, &'static str)] {
    use LanguageOption::*;
    match lang {
        English => &[
            ("펌웨어 정보 및 설치 환경 검사를 시작합니다.", "Starting firmware information and installation environment check."),
            ("image 폴더 선택이 취소되었습니다.", "Image folder selection was cancelled."),
            ("image 폴더 선택됨", "Image folder selected"),
            ("이미 작업이 진행 중입니다.", "A task is already in progress."),
            ("작업이 비정상적으로 종료되었습니다.", "The task ended abnormally."),
            ("작업 로그 자동 저장 실패", "Failed to auto-save the task log"),
            ("텍스트 파일 저장 실패", "Failed to save text file"),
            ("USB 디버깅 활성화 설정 후 다시 시도해주세요.", "Enable USB debugging and try again."),
            ("기기에 설치된 버전보다 낮을 경우/초기화 O.", "If lower than the installed version: wipe required."),
            ("기기에 설치된 버전보다 높은 경우/초기화 X.", "If higher than the installed version: no wipe."),
            ("실제 값과 다를 수 있음", "may differ from the actual value"),
            ("펌웨어 버전, 플랫폼, 모델명, 필수 partition 유효성, MTK 드라이버 설치 유/무를 검사합니다.", "Checks firmware version, platform, model, required partition validity, and MTK driver installation."),
            ("LPMBOX에서 지원하지 않는 image 폴더입니다.", "This image folder is not supported by LPMBOX."),
            ("선택한 image 폴더에 펌웨어 버전은 심각한 버그가 있습니다.", "The firmware version in the selected image folder has a serious bug."),
            ("기기가 연결 되어있지 않거나, ADB가 감지되지 않습니다.", "The device is not connected or ADB was not detected."),
            ("연결한 기기", "Connected device"),
            ("호환되지 않습니다", "is not compatible"),
            ("MTK 드라이버 설치가 필요합니다.", "MTK driver installation is required."),
            ("다음 단계로 진행해주세요.", "Proceed to the next step."),
            ("펌웨어 유형", "Firmware type"),
            ("가동 시간", "Uptime"),
            ("올바른 image 폴더를", "the correct image folder"),
            ("선택해주세요", "Please select"),
            ("시작 전 확인", "Before starting"),
            ("image 폴더 롬", "Image folder ROM"),
            ("기기에 설치된 롬", "ROM installed on device"),
            ("현재", "Current"),
            ("최신", "Latest"),
            ("검색 결과가 없습니다.", "No search results."),
            ("LPMBOX를 사용하기 위해선", "To use LPMBOX,"),
            ("반드시 설치가 필요합니다", "installation is required"),
            ("드라이버 설치를 해주세요.", "Please install the driver."),
            ("기기 연결이 끊겼습니다.", "The device was disconnected."),
            ("선택한 image 폴더 정보 화면으로 돌아갑니다.", "Returning to the selected image folder information screen."),
            ("PC 후면 연결", "Connect to a rear USB port on the PC"),
            ("노트북은 상관 없음", "not required for laptops"),
            ("올바른 데이터 케이블", "correct data cable"),
            ("국가 코드 재설정 플래싱 준비 실패", "Country code reset flashing preparation failed"),
            ("proinfo 내부에서 KRXX, JPXX, USXX 같은 국가 코드 토큰을 찾지 못했습니다.", "Could not find country-code tokens such as KRXX, JPXX, or USXX inside proinfo."),
            ("기기에 맞는 image 폴더가 아닙니다", "This image folder does not match the device"),
            ("올바른 파일을 선택해서 다시 시도해주세요.", "Select the correct file and try again."),
            ("작업 스레드 오류", "Worker thread error"),
            ("대시보드 갱신", "Dashboard refresh"),
            ("초기화", "wipe"),
            ("유지", "keep"),
            ("검사", "check"),
            ("진행 상태", "progress status"),
            ("출력", "output"),
            ("메세지", "message"),
            ("설치되어 있지 않습니다", "is not installed"),
            ("선택한", "selected"),
            ("감지된", "detected"),
            ("재시도", "retry"),
            ("재선택", "reselect"),
            ("지원하지 않는", "unsupported"),
            ("심각한 버그", "serious bug"),
        ],
        Russian => lpm_stage3_common_pairs_ru(),
        Japanese => lpm_stage3_common_pairs_ja(),
        TraditionalChinese => lpm_stage3_common_pairs_zh(),
        Vietnamese => lpm_stage3_common_pairs_vi(),
        Greek => lpm_stage3_common_pairs_el(),
        Hindi => lpm_stage3_common_pairs_hi(),
        Georgian => lpm_stage3_common_pairs_ka(),
        Dutch => lpm_stage3_common_pairs_nl(),
        Arabic => lpm_stage3_common_pairs_ar(),
        Spanish => lpm_stage3_common_pairs_es(),
        Korean => &[],
    }
}

fn lpm_stage3_common_pairs_ru() -> &'static [(&'static str, &'static str)] {
    &[("이미 작업이 진행 중입니다.", "Задача уже выполняется."), ("선택해주세요", "Выберите"), ("펌웨어", "прошивка"), ("검사", "проверка"), ("설치 환경", "среда установки"), ("기기 연결", "подключение устройства"), ("드라이버", "драйвер"), ("필요합니다", "требуется"), ("작업이", "операция"), ("시작합니다", "запускается"), ("진행 중", "выполняется"), ("정보", "информация"), ("선택된", "выбрано"), ("감지된", "обнаружено"), ("재선택", "выбрать снова"), ("재시도", "повторить"), ("초기화", "сброс"), ("유지", "сохранить"), ("설명", "описание"), ("화면", "экран"), ("버튼", "кнопка"), ("가운데", "по центру"), ("텍스트", "текст"), ("값", "значение"), ("최신", "последняя"), ("현재", "текущая"), ("새로운", "новая"), ("업그레이드", "обновление"), ("파일", "файл"), ("오류", "ошибка"), ("실패", "сбой"), ("성공", "успех"), ("완료", "готово"), ("경로", "путь"), ("폴더", "папка"), ("작업", "операция"), ("로그", "журнал"), ("저장", "сохранить"), ("열기", "открыть"), ("링크", "ссылка")]
}
fn lpm_stage3_common_pairs_ja() -> &'static [(&'static str, &'static str)] {
    &[("이미 작업이 진행 중입니다.", "すでに作業が進行中です。"), ("선택해주세요", "選択してください"), ("펌웨어", "ファームウェア"), ("검사", "検査"), ("설치 환경", "インストール環境"), ("기기 연결", "デバイス接続"), ("드라이버", "ドライバー"), ("필요합니다", "必要です"), ("작업이", "作業"), ("시작합니다", "開始します"), ("진행 중", "進行中"), ("정보", "情報"), ("선택된", "選択済み"), ("감지된", "検出済み"), ("재선택", "再選択"), ("재시도", "再試行"), ("초기화", "初期化"), ("유지", "保持"), ("설명", "説明"), ("화면", "画面"), ("버튼", "ボタン"), ("가운데", "中央"), ("텍스트", "テキスト"), ("값", "値"), ("최신", "最新"), ("현재", "現在"), ("새로운", "新しい"), ("업그레이드", "アップグレード"), ("파일", "ファイル"), ("오류", "エラー"), ("실패", "失敗"), ("성공", "成功"), ("완료", "完了"), ("경로", "パス"), ("폴더", "フォルダー"), ("작업", "作業"), ("로그", "ログ"), ("저장", "保存"), ("열기", "開く"), ("링크", "リンク")]
}
fn lpm_stage3_common_pairs_zh() -> &'static [(&'static str, &'static str)] {
    &[("이미 작업이 진행 중입니다.", "工作已在進行中。"), ("선택해주세요", "請選擇"), ("펌웨어", "韌體"), ("검사", "檢查"), ("설치 환경", "安裝環境"), ("기기 연결", "裝置連線"), ("드라이버", "驅動程式"), ("필요합니다", "需要"), ("작업이", "工作"), ("시작합니다", "開始"), ("진행 중", "進行中"), ("정보", "資訊"), ("선택된", "已選擇"), ("감지된", "已偵測"), ("재선택", "重新選擇"), ("재시도", "重試"), ("초기화", "初始化"), ("유지", "保留"), ("설명", "說明"), ("화면", "畫面"), ("버튼", "按鈕"), ("가운데", "置中"), ("텍스트", "文字"), ("값", "值"), ("최신", "最新"), ("현재", "目前"), ("새로운", "新的"), ("업그레이드", "升級"), ("파일", "檔案"), ("오류", "錯誤"), ("실패", "失敗"), ("성공", "成功"), ("완료", "完成"), ("경로", "路徑"), ("폴더", "資料夾"), ("작업", "工作"), ("로그", "日誌"), ("저장", "儲存"), ("열기", "開啟"), ("링크", "連結")]
}
fn lpm_stage3_common_pairs_vi() -> &'static [(&'static str, &'static str)] {
    &[("이미 작업이 진행 중입니다.", "Một tác vụ đang chạy."), ("선택해주세요", "Vui lòng chọn"), ("펌웨어", "firmware"), ("검사", "kiểm tra"), ("설치 환경", "môi trường cài đặt"), ("기기 연결", "kết nối thiết bị"), ("드라이버", "trình điều khiển"), ("필요합니다", "cần thiết"), ("작업이", "tác vụ"), ("시작합니다", "bắt đầu"), ("진행 중", "đang chạy"), ("정보", "thông tin"), ("선택된", "đã chọn"), ("감지된", "đã phát hiện"), ("재선택", "chọn lại"), ("재시도", "thử lại"), ("초기화", "xóa dữ liệu"), ("유지", "giữ lại"), ("설명", "mô tả"), ("화면", "màn hình"), ("버튼", "nút"), ("가운데", "căn giữa"), ("텍스트", "văn bản"), ("값", "giá trị"), ("최신", "mới nhất"), ("현재", "hiện tại"), ("새로운", "mới"), ("업그레이드", "nâng cấp"), ("파일", "tệp"), ("오류", "lỗi"), ("실패", "thất bại"), ("성공", "thành công"), ("완료", "hoàn tất"), ("경로", "đường dẫn"), ("폴더", "thư mục"), ("작업", "tác vụ"), ("로그", "nhật ký"), ("저장", "lưu"), ("열기", "mở"), ("링크", "liên kết")]
}
fn lpm_stage3_common_pairs_el() -> &'static [(&'static str, &'static str)] {
    &[("이미 작업이 진행 중입니다.", "Μια εργασία εκτελείται ήδη."), ("선택해주세요", "Επιλέξτε"), ("펌웨어", "firmware"), ("검사", "έλεγχος"), ("설치 환경", "περιβάλλον εγκατάστασης"), ("기기 연결", "σύνδεση συσκευής"), ("드라이버", "οδηγός"), ("필요합니다", "απαιτείται"), ("작업이", "εργασία"), ("시작합니다", "ξεκινά"), ("진행 중", "σε εξέλιξη"), ("정보", "πληροφορίες"), ("선택된", "επιλεγμένο"), ("감지된", "εντοπίστηκε"), ("재선택", "επιλογή ξανά"), ("재시도", "δοκιμή ξανά"), ("초기화", "επαναφορά"), ("유지", "διατήρηση"), ("설명", "περιγραφή"), ("화면", "οθόνη"), ("버튼", "κουμπί"), ("가운데", "κέντρο"), ("텍스트", "κείμενο"), ("값", "τιμή"), ("최신", "τελευταίο"), ("현재", "τρέχον"), ("새로운", "νέο"), ("업그레이드", "αναβάθμιση"), ("파일", "αρχείο"), ("오류", "σφάλμα"), ("실패", "αποτυχία"), ("성공", "επιτυχία"), ("완료", "ολοκληρώθηκε"), ("경로", "διαδρομή"), ("폴더", "φάκελος"), ("작업", "εργασία"), ("로그", "αρχείο καταγραφής"), ("저장", "αποθήκευση"), ("열기", "άνοιγμα"), ("링크", "σύνδεσμος")]
}
fn lpm_stage3_common_pairs_hi() -> &'static [(&'static str, &'static str)] {
    &[("이미 작업이 진행 중입니다.", "एक कार्य पहले से चल रहा है।"), ("선택해주세요", "कृपया चुनें"), ("펌웨어", "firmware"), ("검사", "जाँच"), ("설치 환경", "installation environment"), ("기기 연결", "device connection"), ("드라이버", "driver"), ("필요합니다", "आवश्यक है"), ("작업이", "कार्य"), ("시작합니다", "शुरू होता है"), ("진행 중", "चल रहा है"), ("정보", "जानकारी"), ("선택된", "चयनित"), ("감지된", "detected"), ("재선택", "फिर से चुनें"), ("재시도", "फिर से प्रयास करें"), ("초기화", "reset"), ("유지", "keep"), ("설명", "विवरण"), ("화면", "screen"), ("버튼", "button"), ("가운데", "center"), ("텍스트", "text"), ("값", "value"), ("최신", "latest"), ("현재", "current"), ("새로운", "new"), ("업그레이드", "upgrade"), ("파일", "file"), ("오류", "error"), ("실패", "failed"), ("성공", "success"), ("완료", "completed"), ("경로", "path"), ("폴더", "folder"), ("작업", "task"), ("로그", "log"), ("저장", "save"), ("열기", "open"), ("링크", "link")]
}
fn lpm_stage3_common_pairs_ka() -> &'static [(&'static str, &'static str)] {
    &[("이미 작업이 진행 중입니다.", "ამოცანა უკვე მიმდინარეობს."), ("선택해주세요", "გთხოვთ აირჩიოთ"), ("펌웨어", "firmware"), ("검사", "შემოწმება"), ("설치 환경", "ინსტალაციის გარემო"), ("기기 연결", "მოწყობილობის დაკავშირება"), ("드라이버", "დრაივერი"), ("필요합니다", "საჭიროა"), ("작업이", "ამოცანა"), ("시작합니다", "იწყება"), ("진행 중", "მიმდინარეობს"), ("정보", "ინფორმაცია"), ("선택된", "არჩეული"), ("감지된", "აღმოჩენილი"), ("재선택", "ხელახლა არჩევა"), ("재시도", "ხელახლა ცდა"), ("초기화", "გადატვირთვა"), ("유지", "შენარჩუნება"), ("설명", "აღწერა"), ("화면", "ეკრანი"), ("버튼", "ღილაკი"), ("가운데", "ცენტრი"), ("텍스트", "ტექსტი"), ("값", "მნიშვნელობა"), ("최신", "უახლესი"), ("현재", "მიმდინარე"), ("새로운", "ახალი"), ("업그레이드", "განახლება"), ("파일", "ფაილი"), ("오류", "შეცდომა"), ("실패", "ვერ შესრულდა"), ("성공", "წარმატება"), ("완료", "დასრულდა"), ("경로", "ბილიკი"), ("폴더", "საქაღალდე"), ("작업", "ამოცანა"), ("로그", "ჟურნალი"), ("저장", "შენახვა"), ("열기", "გახსნა"), ("링크", "ბმული")]
}
fn lpm_stage3_common_pairs_nl() -> &'static [(&'static str, &'static str)] {
    &[("이미 작업이 진행 중입니다.", "Er is al een taak bezig."), ("선택해주세요", "Selecteer"), ("펌웨어", "firmware"), ("검사", "controle"), ("설치 환경", "installatieomgeving"), ("기기 연결", "apparaatverbinding"), ("드라이버", "stuurprogramma"), ("필요합니다", "vereist"), ("작업이", "taak"), ("시작합니다", "wordt gestart"), ("진행 중", "bezig"), ("정보", "informatie"), ("선택된", "geselecteerd"), ("감지된", "gedetecteerd"), ("재선택", "opnieuw selecteren"), ("재시도", "opnieuw proberen"), ("초기화", "wissen"), ("유지", "behouden"), ("설명", "beschrijving"), ("화면", "scherm"), ("버튼", "knop"), ("가운데", "midden"), ("텍스트", "tekst"), ("값", "waarde"), ("최신", "nieuwste"), ("현재", "huidige"), ("새로운", "nieuwe"), ("업그레이드", "upgrade"), ("파일", "bestand"), ("오류", "fout"), ("실패", "mislukt"), ("성공", "geslaagd"), ("완료", "voltooid"), ("경로", "pad"), ("폴더", "map"), ("작업", "taak"), ("로그", "log"), ("저장", "opslaan"), ("열기", "openen"), ("링크", "link")]
}
fn lpm_stage3_common_pairs_ar() -> &'static [(&'static str, &'static str)] {
    &[("이미 작업이 진행 중입니다.", "توجد مهمة قيد التنفيذ بالفعل."), ("선택해주세요", "يرجى الاختيار"), ("펌웨어", "firmware"), ("검사", "فحص"), ("설치 환경", "بيئة التثبيت"), ("기기 연결", "اتصال الجهاز"), ("드라이버", "برنامج التشغيل"), ("필요합니다", "مطلوب"), ("작업이", "المهمة"), ("시작합니다", "تبدأ"), ("진행 중", "قيد التنفيذ"), ("정보", "معلومات"), ("선택된", "محدد"), ("감지된", "تم اكتشافه"), ("재선택", "إعادة الاختيار"), ("재시도", "إعادة المحاولة"), ("초기화", "مسح البيانات"), ("유지", "احتفاظ"), ("설명", "وصف"), ("화면", "الشاشة"), ("버튼", "زر"), ("가운데", "الوسط"), ("텍스트", "نص"), ("값", "قيمة"), ("최신", "الأحدث"), ("현재", "الحالي"), ("새로운", "جديد"), ("업그레이드", "ترقية"), ("파일", "ملف"), ("오류", "خطأ"), ("실패", "فشل"), ("성공", "نجاح"), ("완료", "اكتمل"), ("경로", "مسار"), ("폴더", "مجلد"), ("작업", "مهمة"), ("로그", "سجل"), ("저장", "حفظ"), ("열기", "فتح"), ("링크", "رابط")]
}
fn lpm_stage3_common_pairs_es() -> &'static [(&'static str, &'static str)] {
    &[("이미 작업이 진행 중입니다.", "Ya hay una tarea en curso."), ("선택해주세요", "Selecciona"), ("펌웨어", "firmware"), ("검사", "comprobación"), ("설치 환경", "entorno de instalación"), ("기기 연결", "conexión del dispositivo"), ("드라이버", "controlador"), ("필요합니다", "es necesario"), ("작업이", "tarea"), ("시작합니다", "se inicia"), ("진행 중", "en curso"), ("정보", "información"), ("선택된", "seleccionado"), ("감지된", "detectado"), ("재선택", "volver a seleccionar"), ("재시도", "reintentar"), ("초기화", "borrar datos"), ("유지", "mantener"), ("설명", "descripción"), ("화면", "pantalla"), ("버튼", "botón"), ("가운데", "centro"), ("텍스트", "texto"), ("값", "valor"), ("최신", "más reciente"), ("현재", "actual"), ("새로운", "nuevo"), ("업그레이드", "actualización"), ("파일", "archivo"), ("오류", "error"), ("실패", "fallo"), ("성공", "éxito"), ("완료", "completado"), ("경로", "ruta"), ("폴더", "carpeta"), ("작업", "tarea"), ("로그", "registro"), ("저장", "guardar"), ("열기", "abrir"), ("링크", "enlace")]
}

fn lpm_translate_exact(lang: LanguageOption, key: &str) -> Option<&'static str> {
    use LanguageOption::*;
    let en = |s: &'static str| -> Option<&'static str> { Some(s) };
    match key {
        "대시보드" | "대시 보드" => match lang { English => en("Dashboard"), Russian => en("Панель"), Japanese => en("ダッシュボード"), TraditionalChinese => en("儀表板"), Vietnamese => en("Bảng điều khiển"), Greek => en("Πίνακας ελέγχου"), Hindi => en("डैशबोर्ड"), Georgian => en("დაფა"), Dutch => en("Dashboard"), Arabic => en("لوحة التحكم"), Spanish => en("Panel"), Korean => None },
        "ROM 작업" => match lang { English => en("ROM Tasks"), Russian => en("Операции ROM"), Japanese => en("ROM作業"), TraditionalChinese => en("ROM 工作"), Vietnamese => en("Tác vụ ROM"), Greek => en("Εργασίες ROM"), Hindi => en("ROM कार्य"), Georgian => en("ROM სამუშაოები"), Dutch => en("ROM-taken"), Arabic => en("مهام ROM"), Spanish => en("Tareas ROM"), Korean => None },
        "추가 옵션" => match lang { English => en("Additional Options"), Russian => en("Дополнительные параметры"), Japanese => en("追加オプション"), TraditionalChinese => en("其他選項"), Vietnamese => en("Tùy chọn bổ sung"), Greek => en("Πρόσθετες επιλογές"), Hindi => en("अतिरिक्त विकल्प"), Georgian => en("დამატებითი პარამეტრები"), Dutch => en("Extra opties"), Arabic => en("خيارات إضافية"), Spanish => en("Opciones adicionales"), Korean => None },
        "펌웨어 다운로드" => match lang { English => en("Firmware Download"), Russian => en("Загрузка прошивки"), Japanese => en("ファームウェアダウンロード"), TraditionalChinese => en("韌體下載"), Vietnamese => en("Tải firmware"), Greek => en("Λήψη firmware"), Hindi => en("फर्मवेयर डाउनलोड"), Georgian => en("Firmware ჩამოტვირთვა"), Dutch => en("Firmware downloaden"), Arabic => en("تنزيل البرنامج الثابت"), Spanish => en("Descargar firmware"), Korean => None },
        "로그 관리" => match lang { English => en("Log Management"), Russian => en("Управление журналами"), Japanese => en("ログ管理"), TraditionalChinese => en("日誌管理"), Vietnamese => en("Quản lý nhật ký"), Greek => en("Διαχείριση αρχείων καταγραφής"), Hindi => en("लॉग प्रबंधन"), Georgian => en("ჟურნალის მართვა"), Dutch => en("Logbeheer"), Arabic => en("إدارة السجلات"), Spanish => en("Gestión de registros"), Korean => None },
        "설정" => match lang { English => en("Settings"), Russian => en("Настройки"), Japanese => en("設定"), TraditionalChinese => en("設定"), Vietnamese => en("Cài đặt"), Greek => en("Ρυθμίσεις"), Hindi => en("सेटिंग्स"), Georgian => en("პარამეტრები"), Dutch => en("Instellingen"), Arabic => en("الإعدادات"), Spanish => en("Configuración"), Korean => None },
        "기기 관리" => match lang { English => en("Device Management"), Russian => en("Управление устройством"), Japanese => en("デバイス管理"), TraditionalChinese => en("裝置管理"), Vietnamese => en("Quản lý thiết bị"), Greek => en("Διαχείριση συσκευής"), Hindi => en("डिवाइस प्रबंधन"), Georgian => en("მოწყობილობის მართვა"), Dutch => en("Apparaatbeheer"), Arabic => en("إدارة الجهاز"), Spanish => en("Gestión del dispositivo"), Korean => None },
        "프로그램" => match lang { English => en("Program"), Russian => en("Программа"), Japanese => en("プログラム"), TraditionalChinese => en("程式"), Vietnamese => en("Chương trình"), Greek => en("Πρόγραμμα"), Hindi => en("प्रोग्राम"), Georgian => en("პროგრამა"), Dutch => en("Programma"), Arabic => en("البرنامج"), Spanish => en("Programa"), Korean => None },
        "LPMBox 작업 상태와 주요 기능을 한 화면에서 관리합니다." => match lang { English => en("Manage LPMBox status and main features in one place."), Russian => en("Управляйте состоянием LPMBox и основными функциями на одном экране."), Japanese => en("LPMBoxの状態と主要機能を1つの画面で管理します。"), TraditionalChinese => en("在同一畫面管理 LPMBox 狀態與主要功能。"), Vietnamese => en("Quản lý trạng thái LPMBox và các chức năng chính trên một màn hình."), Greek => en("Διαχειριστείτε την κατάσταση και τις βασικές λειτουργίες του LPMBox σε μία οθόνη."), Hindi => en("LPMBox की स्थिति और मुख्य सुविधाओं को एक ही स्क्रीन पर प्रबंधित करें।"), Georgian => en("მართეთ LPMBox-ის სტატუსი და ძირითადი ფუნქციები ერთ ეკრანზე."), Dutch => en("Beheer de LPMBox-status en hoofdfuncties op één scherm."), Arabic => en("أدر حالة LPMBox والميزات الرئيسية من شاشة واحدة."), Spanish => en("Gestiona el estado de LPMBox y sus funciones principales en una sola pantalla."), Korean => None },
        "image 폴더를 선택해주세요." => match lang { English => en("Please select the image folder."), Russian => en("Выберите папку image."), Japanese => en("imageフォルダーを選択してください。"), TraditionalChinese => en("請選擇 image 資料夾。"), Vietnamese => en("Vui lòng chọn thư mục image."), Greek => en("Επιλέξτε τον φάκελο image."), Hindi => en("कृपया image फ़ोल्डर चुनें।"), Georgian => en("გთხოვთ აირჩიოთ image საქაღალდე."), Dutch => en("Selecteer de image-map."), Arabic => en("يرجى اختيار مجلد image."), Spanish => en("Selecciona la carpeta image."), Korean => None },
        "기기를 추가적으로 설정합니다." => match lang { English => en("Configure additional device options."), Russian => en("Настройте дополнительные параметры устройства."), Japanese => en("デバイスの追加設定を行います。"), TraditionalChinese => en("設定其他裝置選項。"), Vietnamese => en("Cấu hình thêm các tùy chọn thiết bị."), Greek => en("Ρυθμίστε πρόσθετες επιλογές συσκευής."), Hindi => en("अतिरिक्त डिवाइस विकल्प कॉन्फ़िगर करें।"), Georgian => en("დააყენეთ მოწყობილობის დამატებითი პარამეტრები."), Dutch => en("Configureer extra apparaatopties."), Arabic => en("قم بتكوين خيارات إضافية للجهاز."), Spanish => en("Configura opciones adicionales del dispositivo."), Korean => None },
        "작업 로그를 확인하고 텍스트 파일로 저장합니다." => match lang { English => en("View work logs and save them as a text file."), Russian => en("Просматривайте журналы и сохраняйте их в текстовый файл."), Japanese => en("作業ログを確認し、テキストファイルとして保存します。"), TraditionalChinese => en("查看工作日誌並儲存為文字檔。"), Vietnamese => en("Xem nhật ký công việc và lưu dưới dạng tệp văn bản."), Greek => en("Προβάλετε τα αρχεία καταγραφής και αποθηκεύστε τα ως αρχείο κειμένου."), Hindi => en("कार्य लॉग देखें और उन्हें टेक्स्ट फ़ाइल में सहेजें।"), Georgian => en("ნახეთ სამუშაო ჟურნალები და შეინახეთ ტექსტურ ფაილად."), Dutch => en("Bekijk werklogs en sla ze op als tekstbestand."), Arabic => en("اعرض سجلات العمل واحفظها كملف نصي."), Spanish => en("Consulta los registros y guárdalos como archivo de texto."), Korean => None },
        "LPMBOX 프로그램을 설정합니다." => match lang { English => en("Configure the LPMBox program."), Russian => en("Настройте программу LPMBox."), Japanese => en("LPMBoxプログラムを設定します。"), TraditionalChinese => en("設定 LPMBox 程式。"), Vietnamese => en("Cấu hình chương trình LPMBox."), Greek => en("Ρυθμίστε το πρόγραμμα LPMBox."), Hindi => en("LPMBox प्रोग्राम कॉन्फ़िगर करें।"), Georgian => en("დააყენეთ LPMBox პროგრამა."), Dutch => en("Configureer het LPMBox-programma."), Arabic => en("قم بإعداد برنامج LPMBox."), Spanish => en("Configura el programa LPMBox."), Korean => None },
        "PRC ↔ ROW 설치" => match lang { English => en("PRC ↔ ROW Install"), Russian => en("Установка PRC ↔ ROW"), Japanese => en("PRC ↔ ROW インストール"), TraditionalChinese => en("PRC ↔ ROW 安裝"), Vietnamese => en("Cài đặt PRC ↔ ROW"), Greek => en("Εγκατάσταση PRC ↔ ROW"), Hindi => en("PRC ↔ ROW इंस्टॉल"), Georgian => en("PRC ↔ ROW ინსტალაცია"), Dutch => en("PRC ↔ ROW-installatie"), Arabic => en("PRC ↔ ROW تثبيت"), Spanish => en("Instalar PRC ↔ ROW"), Korean => None },
        "ROW(글로벌롬) 업데이트" => match lang { English => en("ROW (Global ROM) Update"), Russian => en("Обновление ROW (глобальная ROM)"), Japanese => en("ROW（グローバルROM）アップデート"), TraditionalChinese => en("ROW（全球版 ROM）更新"), Vietnamese => en("Cập nhật ROW (ROM quốc tế)"), Greek => en("Ενημέρωση ROW (Global ROM)"), Hindi => en("ROW (ग्लोबल ROM) अपडेट"), Georgian => en("ROW (გლობალური ROM) განახლება"), Dutch => en("ROW (Global ROM) update"), Arabic => en("تحديث ROW (الروم العالمي)"), Spanish => en("Actualizar ROW (ROM global)"), Korean => None },
        "기기 복구" => match lang { English => en("Device Recovery"), Russian => en("Восстановление устройства"), Japanese => en("デバイス復旧"), TraditionalChinese => en("裝置修復"), Vietnamese => en("Khôi phục thiết bị"), Greek => en("Ανάκτηση συσκευής"), Hindi => en("डिवाइस रिकवरी"), Georgian => en("მოწყობილობის აღდგენა"), Dutch => en("Apparaatherstel"), Arabic => en("استرداد الجهاز"), Spanish => en("Recuperar dispositivo"), Korean => None },
        "데이터 초기화" => match lang { English => en("Factory reset"), Russian => en("Сброс данных"), Japanese => en("データ初期化"), TraditionalChinese => en("資料初始化"), Vietnamese => en("Xóa dữ liệu"), Greek => en("Επαναφορά δεδομένων"), Hindi => en("डेटा रीसेट"), Georgian => en("მონაცემების გადატვირთვა"), Dutch => en("Gegevens wissen"), Arabic => en("إعادة ضبط البيانات"), Spanish => en("Restablecer datos"), Korean => None },
        "데이터 유지" => match lang { English => en("Keep data"), Russian => en("Сохранить данные"), Japanese => en("データ保持"), TraditionalChinese => en("保留資料"), Vietnamese => en("Giữ dữ liệu"), Greek => en("Διατήρηση δεδομένων"), Hindi => en("डेटा रखें"), Georgian => en("მონაცემების შენარჩუნება"), Dutch => en("Gegevens behouden"), Arabic => en("الاحتفاظ بالبيانات"), Spanish => en("Conservar datos"), Korean => None },
        "설치 시작" => match lang { English => en("Start Install"), Russian => en("Начать установку"), Japanese => en("インストール開始"), TraditionalChinese => en("開始安裝"), Vietnamese => en("Bắt đầu cài đặt"), Greek => en("Έναρξη εγκατάστασης"), Hindi => en("इंस्टॉल शुरू करें"), Georgian => en("ინსტალაციის დაწყება"), Dutch => en("Installatie starten"), Arabic => en("بدء التثبيت"), Spanish => en("Iniciar instalación"), Korean => None },
        "업데이트 시작" => match lang { English => en("Start Update"), Russian => en("Начать обновление"), Japanese => en("アップデート開始"), TraditionalChinese => en("開始更新"), Vietnamese => en("Bắt đầu cập nhật"), Greek => en("Έναρξη ενημέρωσης"), Hindi => en("अपडेट शुरू करें"), Georgian => en("განახლების დაწყება"), Dutch => en("Update starten"), Arabic => en("بدء التحديث"), Spanish => en("Iniciar actualización"), Korean => None },
        "복구 시작" => match lang { English => en("Start Recovery"), Russian => en("Начать восстановление"), Japanese => en("復旧開始"), TraditionalChinese => en("開始修復"), Vietnamese => en("Bắt đầu khôi phục"), Greek => en("Έναρξη ανάκτησης"), Hindi => en("रिकवरी शुरू करें"), Georgian => en("აღდგენის დაწყება"), Dutch => en("Herstel starten"), Arabic => en("بدء الاسترداد"), Spanish => en("Iniciar recuperación"), Korean => None },
        "작업 선택" => match lang { English => en("Select Task"), Russian => en("Выбор операции"), Japanese => en("作業選択"), TraditionalChinese => en("選擇工作"), Vietnamese => en("Chọn tác vụ"), Greek => en("Επιλογή εργασίας"), Hindi => en("कार्य चुनें"), Georgian => en("სამუშაოს არჩევა"), Dutch => en("Taak selecteren"), Arabic => en("اختيار المهمة"), Spanish => en("Seleccionar tarea"), Korean => None },
        "옵션 선택" => match lang { English => en("Select Options"), Russian => en("Выбор параметров"), Japanese => en("オプション選択"), TraditionalChinese => en("選擇選項"), Vietnamese => en("Chọn tùy chọn"), Greek => en("Επιλογή επιλογών"), Hindi => en("विकल्प चुनें"), Georgian => en("პარამეტრების არჩევა"), Dutch => en("Opties selecteren"), Arabic => en("اختيار الخيارات"), Spanish => en("Seleccionar opciones"), Korean => None },
        "선택한 image 폴더 정보" => match lang { English => en("Selected image folder information"), Russian => en("Информация о выбранной папке image"), Japanese => en("選択したimageフォルダー情報"), TraditionalChinese => en("已選取 image 資料夾資訊"), Vietnamese => en("Thông tin thư mục image đã chọn"), Greek => en("Πληροφορίες επιλεγμένου φακέλου image"), Hindi => en("चुने गए image फ़ोल्डर की जानकारी"), Georgian => en("არჩეული image საქაღალდის ინფორმაცია"), Dutch => en("Informatie over geselecteerde image-map"), Arabic => en("معلومات مجلد image المحدد"), Spanish => en("Información de la carpeta image seleccionada"), Korean => None },
        "아래 작업을 선택하여 기기에 적용합니다." => match lang { English => en("Select a task below and apply it to the device."), Russian => en("Выберите операцию ниже и примените её к устройству."), Japanese => en("下の作業を選択してデバイスに適用します。"), TraditionalChinese => en("選擇下方工作並套用到裝置。"), Vietnamese => en("Chọn tác vụ bên dưới và áp dụng cho thiết bị."), Greek => en("Επιλέξτε μια εργασία παρακάτω και εφαρμόστε τη στη συσκευή."), Hindi => en("नीचे कार्य चुनें और उसे डिवाइस पर लागू करें।"), Georgian => en("აირჩიეთ ქვემოთ მოცემული სამუშაო და გამოიყენეთ მოწყობილობაზე."), Dutch => en("Selecteer hieronder een taak en pas deze toe op het apparaat."), Arabic => en("حدد مهمة أدناه وطبقها على الجهاز."), Spanish => en("Selecciona una tarea y aplícala al dispositivo."), Korean => None },
        "ROM 작업을 진행하기 전 세부 옵션을 설정합니다." => match lang { English => en("Configure detailed options before starting the ROM task."), Russian => en("Настройте параметры перед запуском операции ROM."), Japanese => en("ROM作業を始める前に詳細オプションを設定します。"), TraditionalChinese => en("開始 ROM 工作前先設定詳細選項。"), Vietnamese => en("Cấu hình tùy chọn chi tiết trước khi chạy tác vụ ROM."), Greek => en("Ρυθμίστε τις λεπτομερείς επιλογές πριν την εργασία ROM."), Hindi => en("ROM कार्य शुरू करने से पहले विस्तृत विकल्प सेट करें।"), Georgian => en("ROM სამუშაოს დაწყებამდე დააყენეთ დეტალური პარამეტრები."), Dutch => en("Configureer gedetailleerde opties voordat de ROM-taak start."), Arabic => en("قم بتكوين الخيارات التفصيلية قبل بدء مهمة ROM."), Spanish => en("Configura las opciones antes de iniciar la tarea ROM."), Korean => None },
        "Image 폴더 정보를 확인합니다." | "image 폴더 정보를 확인합니다." => match lang { English => en("Check the image folder information."), Russian => en("Проверьте информацию папки image."), Japanese => en("imageフォルダー情報を確認します。"), TraditionalChinese => en("檢查 image 資料夾資訊。"), Vietnamese => en("Kiểm tra thông tin thư mục image."), Greek => en("Ελέγξτε τις πληροφορίες του φακέλου image."), Hindi => en("image फ़ोल्डर की जानकारी जांचें।"), Georgian => en("შეამოწმეთ image საქაღალდის ინფორმაცია."), Dutch => en("Controleer de informatie van de image-map."), Arabic => en("تحقق من معلومات مجلد image."), Spanish => en("Comprueba la información de la carpeta image."), Korean => None },
        "현재 image 폴더" => match lang { English => en("Current image folder"), Russian => en("Текущая папка image"), Japanese => en("現在のimageフォルダー"), TraditionalChinese => en("目前 image 資料夾"), Vietnamese => en("Thư mục image hiện tại"), Greek => en("Τρέχων φάκελος image"), Hindi => en("वर्तमान image फ़ोल्डर"), Georgian => en("მიმდინარე image საქაღალდე"), Dutch => en("Huidige image-map"), Arabic => en("مجلد image الحالي"), Spanish => en("Carpeta image actual"), Korean => None },
        "폴더 선택" => match lang { English => en("Select folder"), Russian => en("Выбрать папку"), Japanese => en("フォルダー選択"), TraditionalChinese => en("選擇資料夾"), Vietnamese => en("Chọn thư mục"), Greek => en("Επιλογή φακέλου"), Hindi => en("फ़ोल्डर चुनें"), Georgian => en("საქაღალდის არჩევა"), Dutch => en("Map selecteren"), Arabic => en("اختيار مجلد"), Spanish => en("Seleccionar carpeta"), Korean => None },
        "폴더 재선택" | "재선택" => match lang { English => en("Reselect"), Russian => en("Выбрать снова"), Japanese => en("再選択"), TraditionalChinese => en("重新選擇"), Vietnamese => en("Chọn lại"), Greek => en("Επανεπιλογή"), Hindi => en("फिर से चुनें"), Georgian => en("ხელახლა არჩევა"), Dutch => en("Opnieuw selecteren"), Arabic => en("إعادة الاختيار"), Spanish => en("Volver a seleccionar"), Korean => None },
        "다시 검사" => match lang { English => en("Check again"), Russian => en("Проверить снова"), Japanese => en("再検査"), TraditionalChinese => en("重新檢查"), Vietnamese => en("Kiểm tra lại"), Greek => en("Έλεγχος ξανά"), Hindi => en("फिर से जांचें"), Georgian => en("ხელახლა შემოწმება"), Dutch => en("Opnieuw controleren"), Arabic => en("إعادة الفحص"), Spanish => en("Comprobar de nuevo"), Korean => None },
        "드라이버 설치" => match lang { English => en("Install driver"), Russian => en("Установить драйвер"), Japanese => en("ドライバーをインストール"), TraditionalChinese => en("安裝驅動程式"), Vietnamese => en("Cài đặt trình điều khiển"), Greek => en("Εγκατάσταση οδηγού"), Hindi => en("ड्राइवर इंस्टॉल करें"), Georgian => en("დრაივერის დაყენება"), Dutch => en("Driver installeren"), Arabic => en("تثبيت برنامج التشغيل"), Spanish => en("Instalar controlador"), Korean => None },
        "다음 단계로 이동" => match lang { English => en("Go to next step"), Russian => en("Перейти к следующему шагу"), Japanese => en("次のステップへ"), TraditionalChinese => en("前往下一步"), Vietnamese => en("Chuyển sang bước tiếp theo"), Greek => en("Μετάβαση στο επόμενο βήμα"), Hindi => en("अगले चरण पर जाएं"), Georgian => en("შემდეგ ეტაპზე გადასვლა"), Dutch => en("Naar de volgende stap"), Arabic => en("الانتقال إلى الخطوة التالية"), Spanish => en("Ir al siguiente paso"), Korean => None },
        "이전 메뉴로 이동" => match lang { English => en("Back to previous menu"), Russian => en("Вернуться в предыдущее меню"), Japanese => en("前のメニューへ"), TraditionalChinese => en("返回上一個選單"), Vietnamese => en("Quay lại menu trước"), Greek => en("Πίσω στο προηγούμενο μενού"), Hindi => en("पिछले मेनू पर जाएं"), Georgian => en("წინა მენიუში დაბრუნება"), Dutch => en("Terug naar vorig menu"), Arabic => en("العودة إلى القائمة السابقة"), Spanish => en("Volver al menú anterior"), Korean => None },
        "계속" => match lang { English => en("Continue"), Russian => en("Продолжить"), Japanese => en("続行"), TraditionalChinese => en("繼續"), Vietnamese => en("Tiếp tục"), Greek => en("Συνέχεια"), Hindi => en("जारी रखें"), Georgian => en("გაგრძელება"), Dutch => en("Doorgaan"), Arabic => en("متابعة"), Spanish => en("Continuar"), Korean => None },
        "활성화" => match lang { English => en("Enable"), Russian => en("Включить"), Japanese => en("有効化"), TraditionalChinese => en("啟用"), Vietnamese => en("Bật"), Greek => en("Ενεργοποίηση"), Hindi => en("सक्षम करें"), Georgian => en("ჩართვა"), Dutch => en("Inschakelen"), Arabic => en("تفعيل"), Spanish => en("Activar"), Korean => None },
        "비활성화" => match lang { English => en("Disable"), Russian => en("Отключить"), Japanese => en("無効化"), TraditionalChinese => en("停用"), Vietnamese => en("Tắt"), Greek => en("Απενεργοποίηση"), Hindi => en("अक्षम करें"), Georgian => en("გამორთვა"), Dutch => en("Uitschakelen"), Arabic => en("تعطيل"), Spanish => en("Desactivar"), Korean => None },
        "재설정" => match lang { English => en("Reset"), Russian => en("Сбросить"), Japanese => en("再設定"), TraditionalChinese => en("重設"), Vietnamese => en("Đặt lại"), Greek => en("Επαναφορά"), Hindi => en("रीसेट"), Georgian => en("გადაყენება"), Dutch => en("Resetten"), Arabic => en("إعادة ضبط"), Spanish => en("Restablecer"), Korean => None },
        "선택" => match lang { English => en("Select"), Russian => en("Выбрать"), Japanese => en("選択"), TraditionalChinese => en("選擇"), Vietnamese => en("Chọn"), Greek => en("Επιλογή"), Hindi => en("चुनें"), Georgian => en("არჩევა"), Dutch => en("Selecteren"), Arabic => en("اختيار"), Spanish => en("Seleccionar"), Korean => None },
        "취소" => match lang { English => en("Cancel"), Russian => en("Отмена"), Japanese => en("キャンセル"), TraditionalChinese => en("取消"), Vietnamese => en("Hủy"), Greek => en("Άκυρο"), Hindi => en("रद्द करें"), Georgian => en("გაუქმება"), Dutch => en("Annuleren"), Arabic => en("إلغاء"), Spanish => en("Cancelar"), Korean => None },
        "확인" => match lang { English => en("Check"), Russian => en("Проверить"), Japanese => en("確認"), TraditionalChinese => en("檢查"), Vietnamese => en("Kiểm tra"), Greek => en("Έλεγχος"), Hindi => en("जांचें"), Georgian => en("შემოწმება"), Dutch => en("Controleren"), Arabic => en("تحقق"), Spanish => en("Comprobar"), Korean => None },
        "확인 중" => match lang { English => en("Checking"), Russian => en("Проверка"), Japanese => en("確認中"), TraditionalChinese => en("檢查中"), Vietnamese => en("Đang kiểm tra"), Greek => en("Έλεγχος"), Hindi => en("जांच हो रही है"), Georgian => en("მოწმდება"), Dutch => en("Bezig met controleren"), Arabic => en("جارٍ التحقق"), Spanish => en("Comprobando"), Korean => None },
        "이동" => match lang { English => en("Open"), Russian => en("Открыть"), Japanese => en("移動"), TraditionalChinese => en("前往"), Vietnamese => en("Mở"), Greek => en("Άνοιγμα"), Hindi => en("खोलें"), Georgian => en("გახსნა"), Dutch => en("Openen"), Arabic => en("فتح"), Spanish => en("Abrir"), Korean => None },
        "언어 변경" => match lang { English => en("Language"), Russian => en("Язык"), Japanese => en("言語変更"), TraditionalChinese => en("語言變更"), Vietnamese => en("Ngôn ngữ"), Greek => en("Γλώσσα"), Hindi => en("भाषा"), Georgian => en("ენა"), Dutch => en("Taal"), Arabic => en("اللغة"), Spanish => en("Idioma"), Korean => None },
        "프로그램 언어를 변경합니다." => match lang { English => en("Change the program language."), Russian => en("Изменить язык программы."), Japanese => en("プログラムの言語を変更します。"), TraditionalChinese => en("變更程式語言。"), Vietnamese => en("Thay đổi ngôn ngữ chương trình."), Greek => en("Αλλάξτε τη γλώσσα του προγράμματος."), Hindi => en("प्रोग्राम की भाषा बदलें।"), Georgian => en("შეცვალეთ პროგრამის ენა."), Dutch => en("Wijzig de programmataal."), Arabic => en("تغيير لغة البرنامج."), Spanish => en("Cambiar el idioma del programa."), Korean => None },
        "개발자 유튜브" => match lang { English => en("Developer YouTube"), Russian => en("YouTube разработчика"), Japanese => en("開発者YouTube"), TraditionalChinese => en("開發者 YouTube"), Vietnamese => en("YouTube của nhà phát triển"), Greek => en("YouTube προγραμματιστή"), Hindi => en("डेवलपर YouTube"), Georgian => en("დეველოპერის YouTube"), Dutch => en("YouTube van ontwikkelaar"), Arabic => en("يوتيوب المطور"), Spanish => en("YouTube del desarrollador"), Korean => None },
        "샤오신패드에 유용한 프로그램을 확인하실 수 있습니다." => match lang { English => en("Check useful programs for Xiaoxin Pad."), Russian => en("Посмотрите полезные программы для Xiaoxin Pad."), Japanese => en("Xiaoxin Padに役立つプログラムを確認できます。"), TraditionalChinese => en("查看適用於 Xiaoxin Pad 的實用程式。"), Vietnamese => en("Xem các chương trình hữu ích cho Xiaoxin Pad."), Greek => en("Δείτε χρήσιμα προγράμματα για το Xiaoxin Pad."), Hindi => en("Xiaoxin Pad के लिए उपयोगी प्रोग्राम देखें।"), Georgian => en("ნახეთ Xiaoxin Pad-ისთვის სასარგებლო პროგრამები."), Dutch => en("Bekijk nuttige programma's voor de Xiaoxin Pad."), Arabic => en("تحقق من البرامج المفيدة لجهاز Xiaoxin Pad."), Spanish => en("Consulta programas útiles para Xiaoxin Pad."), Korean => None },
        "후원하기" => match lang { English => en("Support"), Russian => en("Поддержать"), Japanese => en("支援する"), TraditionalChinese => en("贊助"), Vietnamese => en("Ủng hộ"), Greek => en("Υποστήριξη"), Hindi => en("समर्थन करें"), Georgian => en("მხარდაჭერა"), Dutch => en("Ondersteunen"), Arabic => en("الدعم"), Spanish => en("Apoyar"), Korean => None },
        "개발자에게 큰 힘과 응원이 됩니다." => match lang { English => en("Your support greatly helps the developer."), Russian => en("Ваша поддержка очень помогает разработчику."), Japanese => en("開発者の大きな励みになります。"), TraditionalChinese => en("您的支持是開發者很大的鼓勵。"), Vietnamese => en("Sự ủng hộ của bạn là động lực lớn cho nhà phát triển."), Greek => en("Η υποστήριξή σας βοηθά πολύ τον προγραμματιστή."), Hindi => en("आपका समर्थन डेवलपर के लिए बहुत मददगार है।"), Georgian => en("თქვენი მხარდაჭერა ძალიან ეხმარება დეველოპერს."), Dutch => en("Uw steun helpt de ontwikkelaar enorm."), Arabic => en("دعمك يساعد المطور كثيرًا."), Spanish => en("Tu apoyo ayuda mucho al desarrollador."), Korean => None },
        "프로그램 업데이트" => match lang { English => en("Program Update"), Russian => en("Обновление программы"), Japanese => en("プログラム更新"), TraditionalChinese => en("程式更新"), Vietnamese => en("Cập nhật chương trình"), Greek => en("Ενημέρωση προγράμματος"), Hindi => en("प्रोग्राम अपडेट"), Georgian => en("პროგრამის განახლება"), Dutch => en("Programma-update"), Arabic => en("تحديث البرنامج"), Spanish => en("Actualización del programa"), Korean => None },
        "LPMBox 최신 릴리즈 버전을 확인합니다." => match lang { English => en("Check the latest LPMBox release."), Russian => en("Проверить последнюю версию LPMBox."), Japanese => en("LPMBoxの最新リリースを確認します。"), TraditionalChinese => en("檢查最新的 LPMBox 版本。"), Vietnamese => en("Kiểm tra bản phát hành LPMBox mới nhất."), Greek => en("Ελέγξτε την πιο πρόσφατη έκδοση του LPMBox."), Hindi => en("नवीनतम LPMBox रिलीज़ जांचें।"), Georgian => en("შეამოწმეთ LPMBox-ის უახლესი ვერსია."), Dutch => en("Controleer de nieuwste LPMBox-release."), Arabic => en("تحقق من أحدث إصدار من LPMBox."), Spanish => en("Comprobar la última versión de LPMBox."), Korean => None },
        "피드백" => match lang { English => en("Feedback"), Russian => en("Обратная связь"), Japanese => en("フィードバック"), TraditionalChinese => en("意見回饋"), Vietnamese => en("Phản hồi"), Greek => en("Σχόλια"), Hindi => en("प्रतिक्रिया"), Georgian => en("უკუკავშირი"), Dutch => en("Feedback"), Arabic => en("ملاحظات"), Spanish => en("Comentarios"), Korean => None },
        "의견을 주시면 프로그램이 완벽해질 수 있습니다." => match lang { English => en("Your feedback helps improve the program."), Russian => en("Ваш отзыв поможет улучшить программу."), Japanese => en("ご意見によりプログラムを改善できます。"), TraditionalChinese => en("您的意見可協助改善程式。"), Vietnamese => en("Phản hồi của bạn giúp chương trình hoàn thiện hơn."), Greek => en("Τα σχόλιά σας βοηθούν στη βελτίωση του προγράμματος."), Hindi => en("आपकी प्रतिक्रिया प्रोग्राम को बेहतर बनाने में मदद करती है।"), Georgian => en("თქვენი უკუკავშირი ეხმარება პროგრამის გაუმჯობესებას."), Dutch => en("Uw feedback helpt het programma te verbeteren."), Arabic => en("تساعد ملاحظاتك على تحسين البرنامج."), Spanish => en("Tus comentarios ayudan a mejorar el programa."), Korean => None },
        "OTA(업데이트)" => match lang { English => en("OTA (Update)"), Russian => en("OTA (обновление)"), Japanese => en("OTA（アップデート）"), TraditionalChinese => en("OTA（更新）"), Vietnamese => en("OTA (Cập nhật)"), Greek => en("OTA (Ενημέρωση)"), Hindi => en("OTA (अपडेट)"), Georgian => en("OTA (განახლება)"), Dutch => en("OTA (update)"), Arabic => en("OTA (تحديث)"), Spanish => en("OTA (actualización)"), Korean => None },
        "업데이트 기능을 활성화 또는 비활성화로 설정합니다." => match lang { English => en("Enable or disable the update feature."), Russian => en("Включите или отключите функцию обновления."), Japanese => en("更新機能を有効または無効に設定します。"), TraditionalChinese => en("啟用或停用更新功能。"), Vietnamese => en("Bật hoặc tắt chức năng cập nhật."), Greek => en("Ενεργοποιήστε ή απενεργοποιήστε τη λειτουργία ενημέρωσης."), Hindi => en("अपडेट सुविधा को सक्षम या अक्षम करें।"), Georgian => en("ჩართეთ ან გამორთეთ განახლების ფუნქცია."), Dutch => en("Schakel de updatefunctie in of uit."), Arabic => en("قم بتفعيل أو تعطيل ميزة التحديث."), Spanish => en("Activa o desactiva la función de actualización."), Korean => None },
        "국가 코드 재설정" => match lang { English => en("Country Code Reset"), Russian => en("Сброс кода страны"), Japanese => en("国コード再設定"), TraditionalChinese => en("國家代碼重設"), Vietnamese => en("Đặt lại mã quốc gia"), Greek => en("Επαναφορά κωδικού χώρας"), Hindi => en("देश कोड रीसेट"), Georgian => en("ქვეყნის კოდის გადაყენება"), Dutch => en("Landcode resetten"), Arabic => en("إعادة ضبط رمز البلد"), Spanish => en("Restablecer código de país"), Korean => None },
        "기기에 설정된 국가 코드를 변경합니다." => match lang { English => en("Change the country code set on the device."), Russian => en("Изменить код страны, заданный на устройстве."), Japanese => en("デバイスに設定された国コードを変更します。"), TraditionalChinese => en("變更裝置上設定的國家代碼。"), Vietnamese => en("Thay đổi mã quốc gia đã đặt trên thiết bị."), Greek => en("Αλλάξτε τον κωδικό χώρας που έχει οριστεί στη συσκευή."), Hindi => en("डिवाइस पर सेट देश कोड बदलें।"), Georgian => en("შეცვალეთ მოწყობილობაზე დაყენებული ქვეყნის კოდი."), Dutch => en("Wijzig de landcode die op het apparaat is ingesteld."), Arabic => en("تغيير رمز البلد المحدد على الجهاز."), Spanish => en("Cambia el código de país configurado en el dispositivo."), Korean => None },
        "작업 로그" => match lang { English => en("Work Log"), Russian => en("Журнал работы"), Japanese => en("作業ログ"), TraditionalChinese => en("工作日誌"), Vietnamese => en("Nhật ký công việc"), Greek => en("Αρχείο εργασιών"), Hindi => en("कार्य लॉग"), Georgian => en("სამუშაო ჟურნალი"), Dutch => en("Werklog"), Arabic => en("سجل العمل"), Spanish => en("Registro de trabajo"), Korean => None },
        "로그 내보내기" => match lang { English => en("Export log"), Russian => en("Экспорт журнала"), Japanese => en("ログをエクスポート"), TraditionalChinese => en("匯出日誌"), Vietnamese => en("Xuất nhật ký"), Greek => en("Εξαγωγή αρχείου"), Hindi => en("लॉग निर्यात करें"), Georgian => en("ჟურნალის ექსპორტი"), Dutch => en("Log exporteren"), Arabic => en("تصدير السجل"), Spanish => en("Exportar registro"), Korean => None },
        "로그 지우기" => match lang { English => en("Clear log"), Russian => en("Очистить журнал"), Japanese => en("ログを消去"), TraditionalChinese => en("清除日誌"), Vietnamese => en("Xóa nhật ký"), Greek => en("Εκκαθάριση αρχείου"), Hindi => en("लॉग साफ़ करें"), Georgian => en("ჟურნალის გასუფთავება"), Dutch => en("Log wissen"), Arabic => en("مسح السجل"), Spanish => en("Borrar registro"), Korean => None },
        "국가 코드" => match lang { English => en("Country Code"), Russian => en("Код страны"), Japanese => en("国コード"), TraditionalChinese => en("國家代碼"), Vietnamese => en("Mã quốc gia"), Greek => en("Κωδικός χώρας"), Hindi => en("देश कोड"), Georgian => en("ქვეყნის კოდი"), Dutch => en("Landcode"), Arabic => en("رمز البلد"), Spanish => en("Código de país"), Korean => None },
        "펌웨어 버전" => match lang { English => en("Firmware version"), Russian => en("Версия прошивки"), Japanese => en("ファームウェアバージョン"), TraditionalChinese => en("韌體版本"), Vietnamese => en("Phiên bản firmware"), Greek => en("Έκδοση firmware"), Hindi => en("फर्मवेयर संस्करण"), Georgian => en("Firmware ვერსია"), Dutch => en("Firmwareversie"), Arabic => en("إصدار البرنامج الثابت"), Spanish => en("Versión de firmware"), Korean => None },
        "설정 언어" => match lang { English => en("System language"), Russian => en("Язык системы"), Japanese => en("設定言語"), TraditionalChinese => en("系統語言"), Vietnamese => en("Ngôn ngữ hệ thống"), Greek => en("Γλώσσα συστήματος"), Hindi => en("सिस्टम भाषा"), Georgian => en("სისტემის ენა"), Dutch => en("Systeemtaal"), Arabic => en("لغة النظام"), Spanish => en("Idioma del sistema"), Korean => None },
        "와이드바인 레벨" => match lang { English => en("Widevine level"), Russian => en("Уровень Widevine"), Japanese => en("Widevineレベル"), TraditionalChinese => en("Widevine 等級"), Vietnamese => en("Cấp Widevine"), Greek => en("Επίπεδο Widevine"), Hindi => en("Widevine स्तर"), Georgian => en("Widevine დონე"), Dutch => en("Widevine-niveau"), Arabic => en("مستوى Widevine"), Spanish => en("Nivel Widevine"), Korean => None },
        "기기에 원본 롬" => match lang { English => en("Original ROM"), Russian => en("Исходная ROM"), Japanese => en("デバイスの元ROM"), TraditionalChinese => en("裝置原始 ROM"), Vietnamese => en("ROM gốc trên thiết bị"), Greek => en("Αρχική ROM συσκευής"), Hindi => en("मूल ROM"), Georgian => en("ორიგინალი ROM"), Dutch => en("Originele ROM"), Arabic => en("الروم الأصلي"), Spanish => en("ROM original"), Korean => None },
        "기기에 설치한 롬" => match lang { English => en("Installed ROM"), Russian => en("Установленная ROM"), Japanese => en("インストール済みROM"), TraditionalChinese => en("已安裝 ROM"), Vietnamese => en("ROM đã cài"), Greek => en("Εγκατεστημένη ROM"), Hindi => en("इंस्टॉल किया गया ROM"), Georgian => en("დაყენებული ROM"), Dutch => en("Geïnstalleerde ROM"), Arabic => en("الروم المثبت"), Spanish => en("ROM instalada"), Korean => None },
        "설정된 슬롯 값" => match lang { English => en("Active slot"), Russian => en("Активный слот"), Japanese => en("設定スロット値"), TraditionalChinese => en("已設定 Slot 值"), Vietnamese => en("Slot đã đặt"), Greek => en("Ενεργό slot"), Hindi => en("सेट स्लॉट मान"), Georgian => en("დაყენებული სლოტი"), Dutch => en("Actieve slot"), Arabic => en("قيمة الفتحة النشطة"), Spanish => en("Slot activo"), Korean => None },
        "하드웨어 정보" => match lang { English => en("Hardware info"), Russian => en("Информация об оборудовании"), Japanese => en("ハードウェア情報"), TraditionalChinese => en("硬體資訊"), Vietnamese => en("Thông tin phần cứng"), Greek => en("Πληροφορίες υλικού"), Hindi => en("हार्डवेयर जानकारी"), Georgian => en("ჰარდვერის ინფორმაცია"), Dutch => en("Hardware-info"), Arabic => en("معلومات العتاد"), Spanish => en("Información de hardware"), Korean => None },
        "AP 칩셋" => match lang { English => en("AP chipset"), Russian => en("AP чипсет"), Japanese => en("APチップセット"), TraditionalChinese => en("AP 晶片組"), Vietnamese => en("Chipset AP"), Greek => en("Chipset AP"), Hindi => en("AP चिपसेट"), Georgian => en("AP ჩიპსეტი"), Dutch => en("AP-chipset"), Arabic => en("مجموعة شرائح AP"), Spanish => en("Chipset AP"), Korean => None },
        "플랫폼" => match lang { English => en("Platform"), Russian => en("Платформа"), Japanese => en("プラットフォーム"), TraditionalChinese => en("平台"), Vietnamese => en("Nền tảng"), Greek => en("Πλατφόρμα"), Hindi => en("प्लेटफ़ॉर्म"), Georgian => en("პლატფორმა"), Dutch => en("Platform"), Arabic => en("المنصة"), Spanish => en("Plataforma"), Korean => None },
        "시스템 업데이트(OTA)" => match lang { English => en("System update (OTA)"), Russian => en("Системное обновление (OTA)"), Japanese => en("システム更新（OTA）"), TraditionalChinese => en("系統更新（OTA）"), Vietnamese => en("Cập nhật hệ thống (OTA)"), Greek => en("Ενημέρωση συστήματος (OTA)"), Hindi => en("सिस्टम अपडेट (OTA)"), Georgian => en("სისტემის განახლება (OTA)"), Dutch => en("Systeemupdate (OTA)"), Arabic => en("تحديث النظام (OTA)"), Spanish => en("Actualización del sistema (OTA)"), Korean => None },
        "시리얼 넘버" => match lang { English => en("Serial number"), Russian => en("Серийный номер"), Japanese => en("シリアル番号"), TraditionalChinese => en("序號"), Vietnamese => en("Số sê-ri"), Greek => en("Σειριακός αριθμός"), Hindi => en("सीरियल नंबर"), Georgian => en("სერიული ნომერი"), Dutch => en("Serienummer"), Arabic => en("الرقم التسلسلي"), Spanish => en("Número de serie"), Korean => None },
        "알 수 없음" => match lang { English => en("Unknown"), Russian => en("Неизвестно"), Japanese => en("不明"), TraditionalChinese => en("未知"), Vietnamese => en("Không rõ"), Greek => en("Άγνωστο"), Hindi => en("अज्ञात"), Georgian => en("უცნობია"), Dutch => en("Onbekend"), Arabic => en("غير معروف"), Spanish => en("Desconocido"), Korean => None },
        "감지 전" => match lang { English => en("Not detected yet"), Russian => en("Еще не обнаружено"), Japanese => en("未検出"), TraditionalChinese => en("尚未偵測"), Vietnamese => en("Chưa phát hiện"), Greek => en("Δεν εντοπίστηκε ακόμη"), Hindi => en("अभी पता नहीं चला"), Georgian => en("ჯერ არ არის აღმოჩენილი"), Dutch => en("Nog niet gedetecteerd"), Arabic => en("لم يتم الاكتشاف بعد"), Spanish => en("Aún no detectado"), Korean => None },
        "작업 중" => match lang { English => en("Working"), Russian => en("Выполняется"), Japanese => en("作業中"), TraditionalChinese => en("工作中"), Vietnamese => en("Đang xử lý"), Greek => en("Σε εξέλιξη"), Hindi => en("कार्य चल रहा है"), Georgian => en("მიმდინარეობს"), Dutch => en("Bezig"), Arabic => en("جارٍ العمل"), Spanish => en("Trabajando"), Korean => None },
        "대기 중" => match lang { English => en("Idle"), Russian => en("Ожидание"), Japanese => en("待機中"), TraditionalChinese => en("待機中"), Vietnamese => en("Đang chờ"), Greek => en("Σε αναμονή"), Hindi => en("प्रतीक्षा में"), Georgian => en("მოლოდინში"), Dutch => en("Inactief"), Arabic => en("في الانتظار"), Spanish => en("En espera"), Korean => None },
        "선택 안 됨" => match lang { English => en("Not selected"), Russian => en("Не выбрано"), Japanese => en("未選択"), TraditionalChinese => en("未選擇"), Vietnamese => en("Chưa chọn"), Greek => en("Δεν επιλέχθηκε"), Hindi => en("चयनित नहीं"), Georgian => en("არჩეული არ არის"), Dutch => en("Niet geselecteerd"), Arabic => en("غير محدد"), Spanish => en("No seleccionado"), Korean => None },
        "선택 불가" => match lang { English => en("Unavailable"), Russian => en("Недоступно"), Japanese => en("選択不可"), TraditionalChinese => en("無法選擇"), Vietnamese => en("Không khả dụng"), Greek => en("Μη διαθέσιμο"), Hindi => en("उपलब्ध नहीं"), Georgian => en("მიუწვდომელია"), Dutch => en("Niet beschikbaar"), Arabic => en("غير متاح"), Spanish => en("No disponible"), Korean => None },
        "시작 전 확인" => match lang { English => en("Before you start"), Russian => en("Перед началом"), Japanese => en("開始前の確認"), TraditionalChinese => en("開始前確認"), Vietnamese => en("Kiểm tra trước khi bắt đầu"), Greek => en("Πριν ξεκινήσετε"), Hindi => en("शुरू करने से पहले"), Georgian => en("დაწყებამდე"), Dutch => en("Vóór starten"), Arabic => en("قبل البدء"), Spanish => en("Antes de empezar"), Korean => None },
        "새로운 업데이트가 있습니다." => match lang { English => en("A new update is available."), Russian => en("Доступно новое обновление."), Japanese => en("新しいアップデートがあります。"), TraditionalChinese => en("有新的更新。"), Vietnamese => en("Có bản cập nhật mới."), Greek => en("Υπάρχει νέα ενημέρωση."), Hindi => en("एक नया अपडेट उपलब्ध है।"), Georgian => en("ხელმისაწვდომია ახალი განახლება."), Dutch => en("Er is een nieuwe update beschikbaar."), Arabic => en("يتوفر تحديث جديد."), Spanish => en("Hay una nueva actualización disponible."), Korean => None },
        "현재 버전의 문제를\n해결하고 업그레이드한 파일을\n감지했습니다." => match lang { English => en("A file that fixes issues in the current version\nand upgrades the program\nwas detected."), Russian => en("Обнаружен файл, который исправляет проблемы\nтекущей версии и обновляет программу."), Japanese => en("現在のバージョンの問題を解決し\nアップグレードしたファイルを\n検出しました。"), TraditionalChinese => en("已偵測到可修正目前版本問題\n並升級程式的檔案。"), Vietnamese => en("Đã phát hiện tệp khắc phục sự cố\ncủa phiên bản hiện tại và nâng cấp chương trình."), Greek => en("Εντοπίστηκε αρχείο που διορθώνει προβλήματα\nτης τρέχουσας έκδοσης και αναβαθμίζει το πρόγραμμα."), Hindi => en("वर्तमान संस्करण की समस्याएं ठीक करने\nऔर प्रोग्राम अपग्रेड करने वाली फ़ाइल मिली है।"), Georgian => en("აღმოჩენილია ფაილი, რომელიც აგვარებს მიმდინარე ვერსიის პრობლემებს\nდა აახლებს პროგრამას."), Dutch => en("Er is een bestand gevonden dat problemen\nin de huidige versie oplost en het programma bijwerkt."), Arabic => en("تم اكتشاف ملف يعالج مشاكل الإصدار الحالي\nويرقي البرنامج."), Spanish => en("Se detectó un archivo que corrige problemas\nde la versión actual y actualiza el programa."), Korean => None },
        "파일 업데이트 (권장)" => match lang { English => en("File update (recommended)"), Russian => en("Обновить файл (рекомендуется)"), Japanese => en("ファイル更新（推奨）"), TraditionalChinese => en("檔案更新（建議）"), Vietnamese => en("Cập nhật tệp (khuyến nghị)"), Greek => en("Ενημέρωση αρχείου (συνιστάται)"), Hindi => en("फ़ाइल अपडेट (अनुशंसित)"), Georgian => en("ფაილის განახლება (რეკომენდებულია)"), Dutch => en("Bestand bijwerken (aanbevolen)"), Arabic => en("تحديث الملف (موصى به)"), Spanish => en("Actualizar archivo (recomendado)"), Korean => None },
        "다음에 하기" => match lang { English => en("Later"), Russian => en("Позже"), Japanese => en("後で"), TraditionalChinese => en("稍後再說"), Vietnamese => en("Để sau"), Greek => en("Αργότερα"), Hindi => en("बाद में"), Georgian => en("მოგვიანებით"), Dutch => en("Later"), Arabic => en("لاحقًا"), Spanish => en("Más tarde"), Korean => None },
        "국가 코드 변경" => match lang { English => en("Change Country Code"), Russian => en("Изменить код страны"), Japanese => en("国コード変更"), TraditionalChinese => en("變更國家代碼"), Vietnamese => en("Đổi mã quốc gia"), Greek => en("Αλλαγή κωδικού χώρας"), Hindi => en("देश कोड बदलें"), Georgian => en("ქვეყნის კოდის შეცვლა"), Dutch => en("Landcode wijzigen"), Arabic => en("تغيير رمز البلد"), Spanish => en("Cambiar código de país"), Korean => None },
        "국가 코드 또는 국가명 검색" => match lang { English => en("Search country code or country name"), Russian => en("Поиск кода или названия страны"), Japanese => en("国コードまたは国名を検索"), TraditionalChinese => en("搜尋國家代碼或國家名稱"), Vietnamese => en("Tìm mã quốc gia hoặc tên quốc gia"), Greek => en("Αναζήτηση κωδικού ή ονόματος χώρας"), Hindi => en("देश कोड या देश का नाम खोजें"), Georgian => en("ქვეყნის კოდის ან სახელის ძებნა"), Dutch => en("Zoek landcode of landnaam"), Arabic => en("ابحث عن رمز البلد أو اسم البلد"), Spanish => en("Buscar código o nombre de país"), Korean => None },
        "검색 결과가 없습니다." => match lang { English => en("No results found."), Russian => en("Результаты не найдены."), Japanese => en("検索結果がありません。"), TraditionalChinese => en("沒有搜尋結果。"), Vietnamese => en("Không có kết quả."), Greek => en("Δεν βρέθηκαν αποτελέσματα."), Hindi => en("कोई परिणाम नहीं मिला।"), Georgian => en("შედეგები არ მოიძებნა."), Dutch => en("Geen resultaten gevonden."), Arabic => en("لم يتم العثور على نتائج."), Spanish => en("No se encontraron resultados."), Korean => None },
        "MTK 드라이버 설치가 필요합니다!" => match lang { English => en("MTK driver installation is required!"), Russian => en("Требуется установка драйвера MTK!"), Japanese => en("MTKドライバーのインストールが必要です！"), TraditionalChinese => en("需要安裝 MTK 驅動程式！"), Vietnamese => en("Cần cài đặt trình điều khiển MTK!"), Greek => en("Απαιτείται εγκατάσταση οδηγού MTK!"), Hindi => en("MTK ड्राइवर इंस्टॉल करना आवश्यक है!"), Georgian => en("საჭიროა MTK დრაივერის დაყენება!"), Dutch => en("Installatie van de MTK-driver is vereist!"), Arabic => en("يلزم تثبيت برنامج تشغيل MTK!"), Spanish => en("¡Se requiere instalar el controlador MTK!"), Korean => None },
        "LPMBOX를 사용하기 위해선\n반드시 설치가 필요합니다\n드라이버 설치를 해주세요." => match lang { English => en("To use LPMBOX,\nthe driver must be installed.\nPlease install the driver."), Russian => en("Для использования LPMBOX\nнеобходимо установить драйвер.\nУстановите драйвер."), Japanese => en("LPMBOXを使用するには\nドライバーのインストールが必要です。\nドライバーをインストールしてください。"), TraditionalChinese => en("若要使用 LPMBOX，\n必須安裝驅動程式。\n請安裝驅動程式。"), Vietnamese => en("Để sử dụng LPMBOX,\nbạn phải cài đặt trình điều khiển.\nVui lòng cài đặt trình điều khiển."), Greek => en("Για να χρησιμοποιήσετε το LPMBOX,\nπρέπει να εγκατασταθεί ο οδηγός.\nΕγκαταστήστε τον οδηγό."), Hindi => en("LPMBOX का उपयोग करने के लिए\nड्राइवर इंस्टॉल होना आवश्यक है।\nकृपया ड्राइवर इंस्टॉल करें।"), Georgian => en("LPMBOX-ის გამოსაყენებლად\nდრაივერის დაყენება აუცილებელია.\nგთხოვთ დააყენოთ დრაივერი."), Dutch => en("Om LPMBOX te gebruiken\nmoet de driver zijn geïnstalleerd.\nInstalleer de driver."), Arabic => en("لاستخدام LPMBOX،\nيجب تثبيت برنامج التشغيل.\nيرجى تثبيت برنامج التشغيل."), Spanish => en("Para usar LPMBOX,\nes necesario instalar el controlador.\nInstala el controlador."), Korean => None },
        _ => None,
    }
}

fn lpm_translate_phrasewise(lang: LanguageOption, content: String) -> String {
    if lang.is_korean() || !content.chars().any(|c| ('가'..='힣').contains(&c)) {
        return content;
    }

    let pairs = lpm_phrase_pairs(lang);
    let mut out = content;
    for (from, to) in pairs {
        out = out.replace(from, to);
    }
    out
}

fn lpm_phrase_pairs(lang: LanguageOption) -> &'static [(&'static str, &'static str)] {
    use LanguageOption::*;
    match lang {
        English => &[
            ("proinfo 파티션만 플래싱하려면 선택한 image 폴더의 flash.xml, scatter, DA 파일이 필요합니다.", "To flash only the proinfo partition, flash.xml, scatter, and DA files from the selected image folder are required."),
            ("image 폴더 안에 flash.xml, scatter, DA 파일이 올바르게 있는지 확인해주세요.", "Check that flash.xml, scatter, and DA files are correctly placed in the image folder."),
            ("화면 렌더링 성능을 위해 생략되었습니다. 전체 내용은 로그 저장으로 확인할 수 있습니다.", "were omitted for screen rendering performance. You can check the full contents by saving the log."),
            ("proinfo 전용 Flash Plan 준비 및 작업용 scatter/xml 생성", "proinfo-only Flash Plan preparation and work scatter/xml generation"),
            ("개발자 옵션에서 'USB 디버깅'을 활성화로 설정 한 다음 다시 시도해주세요.", "Enable 'USB debugging' in Developer options and try again."),
            ("기기 연결이 끊겼습니다. 선택한 image 폴더 정보 화면으로 돌아갑니다.", "The device was disconnected. Returning to the selected image folder information screen."),
            ("국가 코드 재설정용 재부팅 및 MediaTek PreLoader 포트 감지", "reboot for country code reset and MediaTek PreLoader port detection"),
            ("ROW(글로벌) 펌웨어 업데이트 [데이터 유지] 작업이 완료되었습니다.", "ROW (global) firmware update [keep data] completed."),
            ("download_agent 폴더의 기존 proinfo 파일 제거 실패", "failed to remove existing proinfo file in download_agent folder"),
            ("image 폴더 유형이 PRC(중국 내수롬)이므로 불가능 합니다.", "Unavailable because the image folder type is PRC (China ROM)."),
            ("외부 adb.exe server가 USB를 점유 중일 수 있습니다", "An external adb.exe server may be occupying USB"),
            ("기기가 켜지지 않거나, 무한 재부팅 등 다양한 오류를 고칩니다.", "Fixes issues such as a device that will not turn on or endless rebooting."),
            ("Flash Plan 준비 및 작업용 scatter/xml 생성", "Flash Plan preparation and work scatter/xml generation"),
            ("proinfo 파일에 국가 코드가 설정 되었는지 확인합니다.", "Checking whether the country code was set in the proinfo file."),
            ("기기가 연결 되어있지 않거나, ADB가 감지되지 않습니다.", "The device is not connected or ADB was not detected."),
            ("ROM 타입을 확인할 수 없어 업데이트 실행을 보류합니다.", "Update is on hold because the ROM type cannot be verified."),
            ("ROW(글로벌롬)기기에 PRC(중국 내수롬)을 설치합니다.", "Installs PRC (China ROM) on a ROW (global ROM) device."),
            ("PRC(중국 내수롬)기기에 ROW(글로벌롬)을 설치합니다.", "Installs ROW (global ROM) on a PRC (China ROM) device."),
            ("proinfo 파티션은 기기 복구 루틴에서 비활성화합니다.", "The proinfo partition is disabled in the device recovery routine."),
            ("국가 코드 변경을 위해 proinfo 파티션을 백업합니다.", "Backing up the proinfo partition to change the country code."),
            ("수정한 proinfo 파일을 image 폴더로 이동합니다.", "Moving the modified proinfo file to the image folder."),
            ("PRC(중국 내수롬) 설치는 데이터 초기화가 필수입니다.", "Data wipe is required when installing PRC (China ROM)."),
            ("PRC(중국 내수롬)은 국가 코드를 변경할 수 없습니다.", "Country code cannot be changed for PRC (China ROM)."),
            ("확인 가능한 LPMBox 릴리즈 버전을 찾지 못했습니다.", "No verifiable LPMBox release version was found."),
            ("케이블을 PC 후면(노트북은 상관 없음)에 꽂아주세요.", "Connect the cable to a rear USB port on the PC. (Not required for laptops.)"),
            ("기기가 연결되어 있지 않아 설치를 실행할 수 없습니다.", "Cannot install because the device is not connected."),
            ("ROM 타입을 확인할 수 없어 설치 실행을 보류합니다.", "Installation is on hold because the ROM type cannot be verified."),
            ("SPFlashToolV6 proinfo 파티션만 플래싱", "SPFlashToolV6 flashing proinfo partition only"),
            ("올바른 데이터 케이블을 사용해주세요. (QnA 참고)", "Use the correct data cable. (See Q&A.)"),
            ("안정성을 위해 데이터 초기화를 필수적으로 해야합니다.", "Data wipe is required for stability."),
            ("먼저 ROM 작업에서 image 폴더를 선택해주세요.", "Please select an image folder from ROM Tasks first."),
            ("설치/업데이트 실행 단계에서 이 펌웨어는 차단됩니다.", "This firmware will be blocked during install/update execution."),
            ("옵션 선택 페이지와 국가 코드 변경 없이 진행합니다.", "Proceeding without the options page or country code change."),
            ("image 폴더의 기존 proinfo 파일 제거 실패", "failed to remove existing proinfo file in the image folder"),
            ("PRC/ROW 설치 [데이터 초기화]를 시도해주세요.", "Try PRC/ROW install [data wipe]."),
            ("PRC(중국 내수롬) 업데이트는 지원하지 않습니다.", "PRC (China ROM) update is not supported."),
            ("안정성을 위해 데이터 초기화를 필수적으로 해야합니다", "Data wipe is required for stability"),
            ("기기 복구 [데이터 초기화] 작업이 완료되었습니다.", "Device recovery [data wipe] completed."),
            ("current slot A 강제 단계를 시작합니다.", "Starting the force current slot A step."),
            ("기기가 PRC(중국 내수롬)이므로 불가능 합니다.", "Unavailable because the device is PRC (China ROM)."),
            ("USB 디버깅 활성화 설정 후 다시 시도해주세요.", "Enable USB debugging and try again."),
            ("PRC/ROW 펌웨어 설치 작업이 완료되었습니다.", "PRC/ROW firmware installation completed."),
            ("OTA(업데이트) 비활성화 작업이 완료되었습니다.", "OTA update disable task completed."),
            ("MediaTek 드라이버가 설치 되어있지 않습니다", "MediaTek driver is not installed"),
            ("기기에 ROW(글로벌롬) 버전을 업데이트 합니다.", "Updating the device to a ROW (global ROM) version."),
            ("GitHub 릴리즈 응답 형식이 예상과 다릅니다.", "GitHub release response format is unexpected."),
            ("기기가 연결되어 있지 않아 실행할 수 없습니다.", "Cannot run because the device is not connected."),
            ("proinfo 백업 및 선택한 국가 코드로 수정", "proinfo backup and modification to the selected country code"),
            ("OTA(업데이트) 활성화 작업이 완료되었습니다.", "OTA update enable task completed."),
            ("기기에 설치된 버전보다 낮을 경우/초기화 O.", "If it is lower than the installed version: wipe required."),
            ("기기에 설치된 버전보다 높은 경우/초기화 X.", "If it is higher than the installed version: no wipe."),
            ("MTK 드라이버 설치 파일 준비를 시작합니다.", "Preparing the MTK driver installer."),
            ("GitHub 릴리즈 정보를 가져오지 못했습니다", "Failed to fetch GitHub release information"),
            ("올바른 image 폴더로 다시 시도해 주세요.", "Try again with the correct image folder."),
            ("ROW(글로벌롬) 업데이트로 진행해 주세요.", "Please proceed with ROW (global ROM) update."),
            ("중국 내수롬과 글로벌롬을 자유롭게 변경 가능", "Switch freely between China ROM and global ROM"),
            ("'펌웨어 설치 실패' 등 기기를 복구합니다.", "Recovers the device from issues such as firmware installation failure."),
            ("MTK 드라이버 설치 작업이 완료되었습니다.", "MTK driver installation task completed."),
            ("MediaTek PreLoader 포트 감지", "MediaTek PreLoader port detection"),
            ("MTK 드라이버 설치 후 다시 시도해주세요.", "Install the MTK driver and try again."),
            ("current slot stage 필요 여부", "current slot stage required"),
            ("시작하기 전 데이터 백업 후 진행해주세요.", "Back up your data before starting."),
            ("proinfo 파티션 백업을 성공했습니다.", "proinfo partition backup succeeded."),
            ("PC(노트북)와 연결한 태블릿에 잠금 해제", "Unlock the tablet connected to the PC/laptop"),
            ("ROW(글로벌롬) 버전을 업데이트합니다.", "Updates the ROW (global ROM) version."),
            ("USB ADB 기기가 감지되지 않았습니다", "USB ADB device was not detected"),
            ("기존 proinfo 백업 파일 제거 실패", "failed to remove existing proinfo backup file"),
            ("proinfo에 국가 코드를 변경합니다.", "Changing the country code in proinfo."),
            ("최신 릴리즈 버전을 해석하지 못했습니다.", "Failed to parse latest release version."),
            ("글로벌롬 펌웨어 버전을 업데이트합니다.", "Updates the global ROM firmware version."),
            ("활성화 할 경우 기기를 초기화 합니다.", "If enabled, the device will be wiped."),
            ("국가 코드 재설정 작업이 완료되었습니다", "Country code reset task completed"),
            ("Fastboot 기기 감지 시간 초과", "Fastboot device detection timed out"),
            ("ADB 기기 감지 및 기기 정보 확인", "ADB device detection and device information check"),
            ("image 폴더 검사 및 플래싱 준비", "image folder check and flashing preparation"),
            ("current slot A 설정 실패", "failed to set current slot A"),
            ("SPFlashToolV6 ROM 설치", "SPFlashToolV6 ROM installation"),
            ("수정한 proinfo 플래싱용 재부팅", "reboot for flashing the modified proinfo"),
            ("MTK 드라이버 설치 여부 확인 실패", "Failed to check MTK driver installation"),
            ("메세지 창 왼쪽 중간 체크 박스 체크", "Check the checkbox in the middle-left of the message window"),
            ("GitHub 응답 JSON 파싱 실패", "Failed to parse GitHub response JSON"),
            ("LPMBOX를 사용할 수 없습니다.", "cannot use LPMBOX."),
            ("기기에 맞는 image 폴더를 선택", "Select an image folder that matches the device"),
            ("국가 코드 재설정 작업을 시작합니다", "Starting the country code reset task"),
            ("SPFlashToolV6 작업 완료", "SPFlashToolV6 task completed"),
            ("spft_log 폴더를 제거했습니다", "removed the spft_log folder"),
            ("proinfo 국가 코드 변경 실패", "Failed to change proinfo country code"),
            ("proinfo 국가 코드 변경 완료", "proinfo country code change completed"),
            ("외부 adb server가 감지되어", "External adb server was detected"),
            ("ADB 명령어로 Slot 설정 완료", "Slot setting completed with ADB command"),
            ("GitHub 응답을 읽지 못했습니다", "Failed to read GitHub response"),
            ("현재 버전 값을 해석하지 못했습니다", "Failed to parse current version"),
            ("partition 상세 정보 읽기", "partition detail read"),
            ("PreLoader 포트 감지 완료", "PreLoader port detected"),
            ("PreLoader 포트 감지 실패", "PreLoader port detection failed"),
            ("작업 scatter XML 재파싱", "work scatter XML reparsing"),
            ("ADB 명령어로 Slot 설정 중", "Setting slot with ADB command"),
            ("25% 이상 충전 후 다시 시도", "Charge to at least 25% and try again"),
            ("데이터 초기화가 필수이기 때문에", "Because data wipe is required"),
            ("기기에 국가 코드를 변경합니다.", "Changes the country code on the device."),
            ("MTK 드라이버 설치 감지 완료", "MTK driver installation detected"),
            ("Fastboot 기기 감지 실패", "Fastboot device detection failed"),
            ("current slot A 설정", "set current slot A"),
            ("PreLoader 포트 감지 중", "Detecting PreLoader port"),
            ("OTA(업데이트) 비활성화 실패", "OTA update disable failed"),
            ("spft_log 폴더 제거 실패", "failed to remove spft_log folder"),
            ("Fastboot 기기 재감지 중", "Re-detecting Fastboot device"),
            ("올바른 image 폴더 재선택", "Reselect the correct image folder"),
            ("올바르지 않은 국가 코드입니다", "invalid country code"),
            ("OTA(업데이트) 활성화 실패", "OTA update enable failed"),
            ("proinfo 패치 필요 여부", "proinfo patch required"),
            ("ADB USB 직접 연결 실패", "Direct ADB USB connection failed"),
            ("bootloader 모드 설정", "bootloader mode setting"),
            ("Fastboot 기기 감지 중", "Detecting Fastboot device"),
            ("기기에 배터리가 부족합니다.", "The device battery is too low."),
            ("새로운 버전을 감지했습니다.", "A new version was detected."),
            ("기기가 다시 감지되었습니다.", "The device was detected again."),
            ("partition 목록 읽기", "partition list read"),
            ("필수 partition 검사", "required partition check"),
            ("backup 폴더 생성 실패", "failed to create backup folder"),
            ("기기 ADB 단계 필요 여부", "device ADB stage required"),
            ("PreLoader 감지 방식", "PreLoader detection method"),
            ("올바른 기기를 연결해주세요.", "Connect the correct device."),
            ("태블릿 확인에 실패했습니다", "Tablet verification failed"),
            ("scatter XML 파싱", "scatter XML parsing"),
            ("image 폴더 기본 검사", "basic image folder check"),
            ("image 폴더 검사 실패", "image folder check failed"),
            ("사용자가 선택한 국가 코드", "User-selected country code"),
            ("작업 scatter.xml", "Work scatter.xml"),
            ("다른 버전 파일로 재시도", "Try again with a different version file"),
            ("patch plan 생성", "patch plan creation"),
            ("patch plan 적용", "patch plan application"),
            ("데이터 유지/초기화 정책", "data keep/wipe policy"),
            ("안정성을 위해 5초 대기", "Waiting 5 seconds for stability"),
            ("선택한 image 폴더", "selected image folder"),
            ("patch 결과 재검증", "patch result recheck"),
            ("다운로드 및 감지 완료", "download and detection completed"),
            ("ADB 기기 확인 실패", "ADB device check failed"),
            ("설치 금지 펌웨어입니다", "This firmware is blocked from installation"),
            ("국가 코드 재설정 실패", "country code reset failed"),
            ("ADB shell 실패", "ADB shell failed"),
            ("원본 flash.xml", "Original flash.xml"),
            ("작업 flash.xml", "Work flash.xml"),
            ("MTK 드라이버 확인", "MTK driver check"),
            ("DA 모드 진입 시작", "Starting DA mode entry"),
            ("국가 코드 확인 완료", "country code verification completed"),
            ("PRC(중국 내수롬)", "PRC (China ROM)"),
            ("호환되지 않습니다.", "is not compatible."),
            ("image 폴더 롬", "Image folder ROM"),
            ("기기를 재시작합니다", "Restarting the device"),
            ("patch 변경 수", "patch change count"),
            ("원본 scatter", "Original scatter"),
            ("올바른 기기 연결", "Connect the correct device"),
            ("펌웨어 검사 실패", "Firmware check failed"),
            ("펌웨어 검사 완료", "Firmware check completed"),
            ("감지된 국가 코드", "Detected country code"),
            ("선택한 국가 코드", "Selected country code"),
            ("기기에 설치된 롬", "Installed ROM on device"),
            ("차단 펌웨어 감지", "Blocked firmware detected"),
            ("파티션 추출 성공", "partition extraction succeeded"),
            ("태블릿 재부팅 중", "Rebooting tablet"),
            ("플래싱 준비 실패", "flashing preparation failed"),
            ("데이터 유지 여부", "keep data"),
            ("로그가 없습니다.", "No logs."),
            ("Allow(허용)", "Allow"),
            ("ROW(글로벌롬)", "ROW (Global ROM)"),
            ("사전 파일 준비", "Pre-file preparation"),
            ("기기 복구 준비", "device recovery preparation"),
            ("기기 복구 실패", "device recovery failed"),
            ("포트가 감지되면", "When the port is detected"),
            ("텍스트 파일을", "Saving the text file to"),
            ("다운로드 실패", "download failed"),
            ("연결한 기기", "Connected device"),
            ("유효성 검사", "Validation"),
            ("작업 로그를", "Saving the task log to"),
            ("ROM 설치", "ROM installation"),
            ("기기 연결", "Connect the device"),
            ("확인 필요", "Needs check"),
            ("확인 불가", "Cannot verify"),
            ("저장합니다", "saving"),
            ("작업 오류", "Task error"),
            ("감지 실패", "Detection failed"),
            ("검증 오류", "validation error"),
            ("검증 경고", "validation warning"),
            ("출력 경로", "output path"),
            ("백업 실패", "backup failed"),
            ("이전 로그", "previous log lines"),
            ("DA 파일", "DA file"),
            ("실행 시작", "execution started"),
            ("종료 코드", "exit code"),
            ("예정 명령", "planned command"),
            ("사용 가능", "available"),
            ("사용 불가", "unavailable"),
            ("모두 통과", "all passed"),
            ("누락 있음", "missing items"),
            ("이상 충전", "or more charged"),
            ("모델명", "Model"),
            ("플랫폼", "Platform"),
            ("글로벌", "Global"),
            ("적용됨", "applied"),
            ("미적용", "not applied"),
            ("통과", "Passed"),
            ("버전", "Version"),
            ("폴더", "folder"),
            ("경로", "path"),
            ("실행", "execute"),
            ("완료", "completed"),
            ("주의", "Caution"),
            ("중국", "China"),
            ("내수", "China ROM"),
            ("경고", "warning"),
            ("있음", "Yes"),
            ("없음", "No"),
        ],
        Russian => &[
            ("데이터 초기화", "Сброс данных"),
            ("중국 내수롬", "китайская ROM"),
            ("국가 코드", "Код страны"),
            ("다시 시도", "Повторите попытку"),
            ("감지 실패", "Не удалось обнаружить"),
            ("감지 완료", "Обнаружение завершено"),
            ("업데이트", "Обновление"),
            ("감지 중", "Обнаружение"),
            ("글로벌롬", "глобальная ROM"),
            ("재부팅", "Перезагрузка"),
            ("플래싱", "Прошивка"),
            ("모델명", "Модель"),
            ("플랫폼", "Платформа"),
            ("오류", "Ошибка"),
            ("완료", "Готово"),
            ("경고", "Предупреждение"),
            ("안내", "Информация"),
            ("작업", "Операция"),
            ("기기", "Устройство"),
            ("설치", "Установка"),
            ("복구", "Восстановление"),
            ("로그", "Журнал"),
            ("선택", "Выбрать"),
            ("취소", "Отмена"),
            ("확인", "Проверка"),
            ("백업", "Резервное копирование"),
            ("실패", "Сбой"),
            ("성공", "Успешно"),
            ("버전", "Версия"),
            ("있음", "Есть"),
            ("없음", "Нет"),
        ],
        Japanese => &[
            ("데이터 초기화", "データ初期化"),
            ("중국 내수롬", "中国版ROM"),
            ("국가 코드", "国コード"),
            ("다시 시도", "再試行"),
            ("감지 실패", "検出失敗"),
            ("감지 완료", "検出完了"),
            ("업데이트", "更新"),
            ("감지 중", "検出中"),
            ("글로벌롬", "グローバルROM"),
            ("재부팅", "再起動"),
            ("플래싱", "フラッシュ"),
            ("모델명", "モデル名"),
            ("플랫폼", "プラットフォーム"),
            ("오류", "エラー"),
            ("완료", "完了"),
            ("경고", "警告"),
            ("안내", "案内"),
            ("작업", "作業"),
            ("기기", "端末"),
            ("설치", "インストール"),
            ("복구", "復旧"),
            ("로그", "ログ"),
            ("선택", "選択"),
            ("취소", "キャンセル"),
            ("확인", "確認"),
            ("백업", "バックアップ"),
            ("실패", "失敗"),
            ("성공", "成功"),
            ("버전", "バージョン"),
            ("있음", "あり"),
            ("없음", "なし"),
        ],
        TraditionalChinese => &[
            ("데이터 초기화", "資料初始化"),
            ("중국 내수롬", "中國版 ROM"),
            ("국가 코드", "國家代碼"),
            ("다시 시도", "重試"),
            ("감지 실패", "偵測失敗"),
            ("감지 완료", "偵測完成"),
            ("업데이트", "更新"),
            ("감지 중", "偵測中"),
            ("글로벌롬", "全球版 ROM"),
            ("재부팅", "重新啟動"),
            ("플래싱", "刷入"),
            ("모델명", "型號"),
            ("플랫폼", "平台"),
            ("오류", "錯誤"),
            ("완료", "完成"),
            ("경고", "警告"),
            ("안내", "提示"),
            ("작업", "工作"),
            ("기기", "裝置"),
            ("설치", "安裝"),
            ("복구", "修復"),
            ("로그", "日誌"),
            ("선택", "選擇"),
            ("취소", "取消"),
            ("확인", "確認"),
            ("백업", "備份"),
            ("실패", "失敗"),
            ("성공", "成功"),
            ("버전", "版本"),
            ("있음", "有"),
            ("없음", "無"),
        ],
        Vietnamese => &[
            ("데이터 초기화", "Xóa dữ liệu"),
            ("중국 내수롬", "ROM Trung Quốc"),
            ("국가 코드", "Mã quốc gia"),
            ("다시 시도", "Thử lại"),
            ("감지 실패", "Không phát hiện được"),
            ("감지 완료", "Đã phát hiện"),
            ("업데이트", "Cập nhật"),
            ("감지 중", "Đang phát hiện"),
            ("글로벌롬", "ROM toàn cầu"),
            ("재부팅", "Khởi động lại"),
            ("플래싱", "Flash"),
            ("모델명", "Tên mẫu"),
            ("플랫폼", "Nền tảng"),
            ("오류", "Lỗi"),
            ("완료", "Hoàn tất"),
            ("경고", "Cảnh báo"),
            ("안내", "Hướng dẫn"),
            ("작업", "Tác vụ"),
            ("기기", "Thiết bị"),
            ("설치", "Cài đặt"),
            ("복구", "Khôi phục"),
            ("로그", "Nhật ký"),
            ("선택", "Chọn"),
            ("취소", "Hủy"),
            ("확인", "Kiểm tra"),
            ("백업", "Sao lưu"),
            ("실패", "Thất bại"),
            ("성공", "Thành công"),
            ("버전", "Phiên bản"),
            ("있음", "Có"),
            ("없음", "Không"),
        ],
        Greek => &[
            ("데이터 초기화", "Διαγραφή δεδομένων"),
            ("중국 내수롬", "κινεζική ROM"),
            ("국가 코드", "Κωδικός χώρας"),
            ("다시 시도", "Δοκιμάστε ξανά"),
            ("감지 실패", "Αποτυχία εντοπισμού"),
            ("감지 완료", "Εντοπισμός ολοκληρώθηκε"),
            ("업데이트", "Ενημέρωση"),
            ("감지 중", "Εντοπισμός"),
            ("글로벌롬", "παγκόσμια ROM"),
            ("재부팅", "Επανεκκίνηση"),
            ("플래싱", "Flash"),
            ("모델명", "Μοντέλο"),
            ("플랫폼", "Πλατφόρμα"),
            ("오류", "Σφάλμα"),
            ("완료", "Ολοκληρώθηκε"),
            ("경고", "Προειδοποίηση"),
            ("안내", "Οδηγίες"),
            ("작업", "Εργασία"),
            ("기기", "Συσκευή"),
            ("설치", "Εγκατάσταση"),
            ("복구", "Ανάκτηση"),
            ("로그", "Αρχείο καταγραφής"),
            ("선택", "Επιλογή"),
            ("취소", "Ακύρωση"),
            ("확인", "Έλεγχος"),
            ("백업", "Αντίγραφο ασφαλείας"),
            ("실패", "Αποτυχία"),
            ("성공", "Επιτυχία"),
            ("버전", "Έκδοση"),
            ("있음", "Ναι"),
            ("없음", "Όχι"),
        ],
        Hindi => &[
            ("데이터 초기화", "डेटा रीसेट"),
            ("중국 내수롬", "चाइना ROM"),
            ("국가 코드", "देश कोड"),
            ("다시 시도", "फिर कोशिश करें"),
            ("감지 실패", "पता लगाने में विफल"),
            ("감지 완료", "पता लग गया"),
            ("업데이트", "अपडेट"),
            ("감지 중", "पता लगाया जा रहा है"),
            ("글로벌롬", "ग्लोबल ROM"),
            ("재부팅", "रीबूट"),
            ("플래싱", "फ्लैशिंग"),
            ("모델명", "मॉडल"),
            ("플랫폼", "प्लेटफ़ॉर्म"),
            ("오류", "त्रुटि"),
            ("완료", "पूर्ण"),
            ("경고", "चेतावनी"),
            ("안내", "मार्गदर्शन"),
            ("작업", "कार्य"),
            ("기기", "डिवाइस"),
            ("설치", "इंस्टॉल"),
            ("복구", "रिकवरी"),
            ("로그", "लॉग"),
            ("선택", "चयन"),
            ("취소", "रद्द"),
            ("확인", "जाँच"),
            ("백업", "बैकअप"),
            ("실패", "विफल"),
            ("성공", "सफल"),
            ("버전", "संस्करण"),
            ("있음", "है"),
            ("없음", "नहीं"),
        ],
        Georgian => &[
            ("데이터 초기화", "მონაცემების წაშლა"),
            ("중국 내수롬", "ჩინური ROM"),
            ("국가 코드", "ქვეყნის კოდი"),
            ("다시 시도", "სცადეთ ხელახლა"),
            ("감지 실패", "ვერ მოიძებნა"),
            ("감지 완료", "აღმოჩენა დასრულდა"),
            ("업데이트", "განახლება"),
            ("감지 중", "მიმდინარეობს აღმოჩენა"),
            ("글로벌롬", "გლობალური ROM"),
            ("재부팅", "გადატვირთვა"),
            ("플래싱", "ფლეშინგი"),
            ("모델명", "მოდელი"),
            ("플랫폼", "პლატფორმა"),
            ("오류", "შეცდომა"),
            ("완료", "დასრულდა"),
            ("경고", "გაფრთხილება"),
            ("안내", "მითითება"),
            ("작업", "სამუშაო"),
            ("기기", "მოწყობილობა"),
            ("설치", "დაყენება"),
            ("복구", "აღდგენა"),
            ("로그", "ჟურნალი"),
            ("선택", "არჩევა"),
            ("취소", "გაუქმება"),
            ("확인", "შემოწმება"),
            ("백업", "სარეზერვო ასლი"),
            ("실패", "ვერ შესრულდა"),
            ("성공", "წარმატება"),
            ("버전", "ვერსია"),
            ("있음", "არის"),
            ("없음", "არ არის"),
        ],
        Dutch => &[
            ("데이터 초기화", "Gegevens wissen"),
            ("중국 내수롬", "China-ROM"),
            ("국가 코드", "Landcode"),
            ("다시 시도", "Probeer opnieuw"),
            ("감지 실패", "Detectie mislukt"),
            ("감지 완료", "Detectie voltooid"),
            ("업데이트", "Update"),
            ("감지 중", "Bezig met detecteren"),
            ("글로벌롬", "Global ROM"),
            ("재부팅", "Opnieuw opstarten"),
            ("플래싱", "Flashen"),
            ("모델명", "Model"),
            ("플랫폼", "Platform"),
            ("오류", "Fout"),
            ("완료", "Voltooid"),
            ("경고", "Waarschuwing"),
            ("안내", "Informatie"),
            ("작업", "Taak"),
            ("기기", "Apparaat"),
            ("설치", "Installatie"),
            ("복구", "Herstel"),
            ("로그", "Log"),
            ("선택", "Selecteren"),
            ("취소", "Annuleren"),
            ("확인", "Controleren"),
            ("백업", "Back-up"),
            ("실패", "Mislukt"),
            ("성공", "Geslaagd"),
            ("버전", "Versie"),
            ("있음", "Ja"),
            ("없음", "Nee"),
        ],
        Arabic => &[
            ("데이터 초기화", "مسح البيانات"),
            ("중국 내수롬", "روم الصين"),
            ("국가 코드", "رمز البلد"),
            ("다시 시도", "حاول مرة أخرى"),
            ("감지 실패", "فشل الاكتشاف"),
            ("감지 완료", "اكتمل الاكتشاف"),
            ("업데이트", "تحديث"),
            ("감지 중", "جارٍ الاكتشاف"),
            ("글로벌롬", "روم عالمي"),
            ("재부팅", "إعادة التشغيل"),
            ("플래싱", "تفليش"),
            ("모델명", "الطراز"),
            ("플랫폼", "النظام الأساسي"),
            ("오류", "خطأ"),
            ("완료", "اكتمل"),
            ("경고", "تحذير"),
            ("안내", "إرشاد"),
            ("작업", "مهمة"),
            ("기기", "الجهاز"),
            ("설치", "تثبيت"),
            ("복구", "استرداد"),
            ("로그", "سجل"),
            ("선택", "تحديد"),
            ("취소", "إلغاء"),
            ("확인", "تحقق"),
            ("백업", "نسخ احتياطي"),
            ("실패", "فشل"),
            ("성공", "نجاح"),
            ("버전", "الإصدار"),
            ("있음", "موجود"),
            ("없음", "غير موجود"),
        ],
        Spanish => &[
            ("데이터 초기화", "Borrar datos"),
            ("중국 내수롬", "ROM china"),
            ("국가 코드", "Código de país"),
            ("다시 시도", "Inténtalo de nuevo"),
            ("감지 실패", "Error de detección"),
            ("감지 완료", "Detección completada"),
            ("업데이트", "Actualización"),
            ("감지 중", "Detectando"),
            ("글로벌롬", "ROM global"),
            ("재부팅", "Reinicio"),
            ("플래싱", "Flasheo"),
            ("모델명", "Modelo"),
            ("플랫폼", "Plataforma"),
            ("오류", "Error"),
            ("완료", "Completado"),
            ("경고", "Advertencia"),
            ("안내", "Guía"),
            ("작업", "Tarea"),
            ("기기", "Dispositivo"),
            ("설치", "Instalación"),
            ("복구", "Recuperación"),
            ("로그", "Registro"),
            ("선택", "Seleccionar"),
            ("취소", "Cancelar"),
            ("확인", "Comprobar"),
            ("백업", "Copia de seguridad"),
            ("실패", "Error"),
            ("성공", "Correcto"),
            ("버전", "Versión"),
            ("있음", "Sí"),
            ("없음", "No"),
        ],
        Korean => &[],
    }
}

const LOG_VISIBLE_MAX_LINES: usize = 500;
const DASHBOARD_REFRESH_INTERVAL: Duration = Duration::from_secs(3);
const DASHBOARD_MODEL_TIMEOUT_INTERVAL: Duration = Duration::from_millis(500);
const LIVE_EVENT_DRAIN_INTERVAL: Duration = Duration::from_millis(250);
const ROM_CHECK_LOADING_INTERVAL: Duration = Duration::from_millis(140);
const SIDEBAR_ANIM_INTERVAL: Duration = Duration::from_millis(16);
const ROM_SLIDE_ANIM_INTERVAL: Duration = Duration::from_millis(16);
const SIDEBAR_ANIM_FACTOR: f32 = 0.15;
const SIDEBAR_ANIM_THRESHOLD: f32 = 0.004;
const ROM_SLIDE_ANIM_FACTOR: f32 = 0.15;
const ROM_SLIDE_ANIM_THRESHOLD: f32 = 0.75;

const ROM_FOLDER_INFO_TITLE_FONT: u32 = 15;
const ROM_FOLDER_INFO_VALUE_FONT: u32 = 12;
const ROM_FOLDER_INFO_VALUE_MAX_CHARS: usize = 34;

const ROM_RIGHT_INFO_TITLE_FONT: u32 = 12;
const ROM_RIGHT_INFO_VALUE_FONT: u32 = 11;
const ROM_RIGHT_INFO_VALUE_MAX_CHARS: usize = 28;

const ROM_BOTTOM_BUTTON_FONT: u32 = 12;


const LPMBOX_MIN_BATTERY_LEVEL: u8 = 20;

fn main() -> iced::Result {
    #[cfg(target_os = "windows")]
    {
        let backend_chosen = std::env::var_os("WGPU_BACKEND").is_some_and(|value| !value.is_empty());
        if !backend_chosen {
            unsafe {
                std::env::set_var("WGPU_BACKEND", "dx12");
            }
        }
    }

    let _ = lpmbox_core::app_paths::ensure_runtime_directories();

    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .title("LPMBox")
        .font(INTER_FONT_BYTES)
        .default_font(lpm_font())
        .window(window::Settings {
            size: Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            resizable: false,
            icon: lpm_window_icon(),
            ..window::Settings::default()
        })
        .centered()
        .run()
}

fn lpm_translate_exact_stage6(lang: LanguageOption, key: &str) -> Option<&'static str> {
    Some(match key {
        "image 폴더 선택" => lpm_lang_text(lang, "Select image folder", "Выбор папки image", "imageフォルダー選択", "選擇 image 資料夾", "Chọn thư mục image", "Επιλογή φακέλου image", "image फ़ोल्डर चुनें", "image საქაღალდის არჩევა", "image-map selecteren", "اختيار مجلد image", "Seleccionar carpeta image"),
        "image 폴더 선택됨: {}" => lpm_lang_text(lang, "Image folder selected: {}", "Папка image выбрана: {}", "imageフォルダーを選択しました: {}", "已選擇 image 資料夾：{}", "Đã chọn thư mục image: {}", "Επιλέχθηκε φάκελος image: {}", "image फ़ोल्डर चुना गया: {}", "არჩეულია image საქაღალდე: {}", "image-map geselecteerd: {}", "تم اختيار مجلد image: {}", "Carpeta image seleccionada: {}"),
        "image 폴더 선택이 취소되었습니다." => lpm_lang_text(lang, "Image folder selection was canceled.", "Выбор папки image отменён.", "imageフォルダーの選択がキャンセルされました。", "已取消選擇 image 資料夾。", "Đã hủy chọn thư mục image.", "Η επιλογή φακέλου image ακυρώθηκε.", "image फ़ोल्डर चयन रद्द किया गया।", "image საქაღალდის არჩევა გაუქმდა.", "Selectie van image-map geannuleerd.", "تم إلغاء اختيار مجلد image.", "Se canceló la selección de la carpeta image."),
        "flash.xml / scatter / DA 파일이 포함된 image 폴더를 선택해주세요." => lpm_lang_text(lang, "Select the image folder that contains flash.xml, scatter, and DA files.", "Выберите папку image, содержащую flash.xml, scatter и DA-файлы.", "flash.xml、scatter、DAファイルを含むimageフォルダーを選択してください。", "請選擇包含 flash.xml、scatter、DA 檔案的 image 資料夾。", "Chọn thư mục image có flash.xml, scatter và tệp DA.", "Επιλέξτε τον φάκελο image που περιέχει flash.xml, scatter και αρχεία DA.", "flash.xml, scatter और DA फ़ाइलों वाला image फ़ोल्डर चुनें।", "აირჩიეთ image საქაღალდე, რომელიც შეიცავს flash.xml, scatter და DA ფაილებს.", "Selecteer de image-map met flash.xml-, scatter- en DA-bestanden.", "اختر مجلد image الذي يحتوي على flash.xml وscatter وملفات DA.", "Seleccione la carpeta image que contiene flash.xml, scatter y archivos DA."),
        "MTK 드라이버 설치" => lpm_lang_text(lang, "Install MTK driver", "Установка драйвера MTK", "MTKドライバーをインストール", "安裝 MTK 驅動程式", "Cài đặt driver MTK", "Εγκατάσταση οδηγού MTK", "MTK ड्राइवर इंस्टॉल करें", "MTK დრაივერის ინსტალაცია", "MTK-stuurprogramma installeren", "تثبيت برنامج تشغيل MTK", "Instalar controlador MTK"),
        "MTK 드라이버 설치 실패: {err}" => lpm_lang_text(lang, "MTK driver installation failed: {err}", "Ошибка установки драйвера MTK: {err}", "MTKドライバーのインストールに失敗しました: {err}", "MTK 驅動程式安裝失敗：{err}", "Cài đặt driver MTK thất bại: {err}", "Η εγκατάσταση οδηγού MTK απέτυχε: {err}", "MTK ड्राइवर इंस्टॉल विफल: {err}", "MTK დრაივერის ინსტალაცია ვერ მოხერხდა: {err}", "Installatie van MTK-stuurprogramma mislukt: {err}", "فشل تثبيت برنامج تشغيل MTK: {err}", "Error al instalar el controlador MTK: {err}"),
        "MTK 드라이버 설치 파일 준비 실패: {err}" => lpm_lang_text(lang, "Failed to prepare the MTK driver installer: {err}", "Не удалось подготовить установщик драйвера MTK: {err}", "MTKドライバーインストーラーの準備に失敗しました: {err}", "MTK 驅動程式安裝檔準備失敗：{err}", "Chuẩn bị tệp cài đặt driver MTK thất bại: {err}", "Αποτυχία προετοιμασίας εγκαταστάτη οδηγού MTK: {err}", "MTK ड्राइवर इंस्टॉलर तैयार करने में विफल: {err}", "MTK დრაივერის ინსტალერის მომზადება ვერ მოხერხდა: {err}", "Voorbereiden van MTK-driverinstallatie mislukt: {err}", "فشل تجهيز مثبت برنامج تشغيل MTK: {err}", "Error al preparar el instalador del controlador MTK: {err}"),
        "OTA 비활성화" => lpm_lang_text(lang, "OTA disable", "Отключение OTA", "OTA無効化", "停用 OTA", "Tắt OTA", "Απενεργοποίηση OTA", "OTA निष्क्रिय करें", "OTA გამორთვა", "OTA uitschakelen", "تعطيل OTA", "Desactivar OTA"),
        "OTA 활성화" => lpm_lang_text(lang, "OTA enable", "Включение OTA", "OTA有効化", "啟用 OTA", "Bật OTA", "Ενεργοποίηση OTA", "OTA सक्षम करें", "OTA ჩართვა", "OTA inschakelen", "تفعيل OTA", "Activar OTA"),
        "ROW 업데이트" => lpm_lang_text(lang, "ROW update", "Обновление ROW", "ROWアップデート", "ROW 更新", "Cập nhật ROW", "Ενημέρωση ROW", "ROW अपडेट", "ROW განახლება", "ROW-update", "تحديث ROW", "Actualización ROW"),
        "ROM 타입" => lpm_lang_text(lang, "ROM type", "Тип ROM", "ROMタイプ", "ROM 類型", "Loại ROM", "Τύπος ROM", "ROM प्रकार", "ROM ტიპი", "ROM-type", "نوع الروم", "Tipo de ROM"),
        "LPMBOX에서 지원하지 않는 image 폴더입니다." => lpm_lang_text(lang, "This image folder is not supported by LPMBOX.", "Эта папка image не поддерживается LPMBOX.", "このimageフォルダーはLPMBOXでサポートされていません。", "LPMBOX 不支援此 image 資料夾。", "Thư mục image này không được LPMBOX hỗ trợ.", "Αυτός ο φάκελος image δεν υποστηρίζεται από το LPMBOX.", "यह image फ़ोल्डर LPMBOX द्वारा समर्थित नहीं है।", "ეს image საქაღალდე LPMBOX-ის მიერ არ არის მხარდაჭერილი.", "Deze image-map wordt niet ondersteund door LPMBOX.", "مجلد image هذا غير مدعوم من LPMBOX.", "Esta carpeta image no es compatible con LPMBOX."),
        "[Error] 1) 데이터 케이블을 PC 후면에 연결한 뒤 다시 시도해주세요. (노트북은 상관 없음)" => lpm_lang_text(lang, "[Error] 1) Connect the data cable to a rear USB port on the PC and try again. (Not required for laptops)", "[Error] 1) Подключите кабель данных к заднему USB-порту ПК и повторите. (Для ноутбуков не требуется)", "[Error] 1) データケーブルをPC背面のUSBポートに接続してから再試行してください。（ノートPCは不要）", "[Error] 1) 請將資料線連接到桌機後方 USB 連接埠後重試。（筆電不需要）", "[Error] 1) Cắm cáp dữ liệu vào cổng USB phía sau PC rồi thử lại. (Laptop không cần)", "[Error] 1) Συνδέστε το καλώδιο δεδομένων στην πίσω θύρα USB του PC και δοκιμάστε ξανά. (Δεν χρειάζεται για laptop)", "[Error] 1) डेटा केबल को PC के पीछे वाले USB पोर्ट में लगाकर फिर प्रयास करें। (लैपटॉप के लिए आवश्यक नहीं)", "[Error] 1) მონაცემთა კაბელი შეაერთეთ PC-ის უკანა USB პორტში და სცადეთ თავიდან. (ლეპტოპისთვის საჭირო არ არის)", "[Error] 1) Sluit de datakabel aan op een USB-poort aan de achterkant van de pc en probeer opnieuw. (Niet nodig voor laptops)", "[Error] 1) وصّل كابل البيانات بمنفذ USB الخلفي في الكمبيوتر ثم حاول مجددًا. (غير مطلوب للحواسيب المحمولة)", "[Error] 1) Conecte el cable de datos al puerto USB trasero del PC e inténtelo de nuevo. (No aplica para portátiles)"),
        "[Error] 2) LPMBOX QnA에 설명드린 케이블로 시도해주세요." => lpm_lang_text(lang, "[Error] 2) Try using the cable recommended in the LPMBOX Q&A.", "[Error] 2) Попробуйте кабель, рекомендованный в Q&A LPMBOX.", "[Error] 2) LPMBOX Q&Aで案内したケーブルで試してください。", "[Error] 2) 請使用 LPMBOX Q&A 中建議的線材嘗試。", "[Error] 2) Hãy thử bằng cáp được khuyến nghị trong Q&A LPMBOX.", "[Error] 2) Δοκιμάστε με το καλώδιο που προτείνεται στο LPMBOX Q&A.", "[Error] 2) LPMBOX Q&A में बताए गए केबल से प्रयास करें।", "[Error] 2) სცადეთ LPMBOX Q&A-ში რეკომენდებული კაბელით.", "[Error] 2) Probeer het met de kabel die in de LPMBOX Q&A wordt aanbevolen.", "[Error] 2) جرّب الكابل الموصى به في أسئلة وأجوبة LPMBOX.", "[Error] 2) Pruebe con el cable recomendado en las preguntas frecuentes de LPMBOX."),
        "[Error] 3) 케이블 PC 후면 연결, 설명한 케이블로 시도했음에도 실패할 경우 다른 PC 또는 노트북으로 시도해주세요." => lpm_lang_text(lang, "[Error] 3) If it still fails after using the rear USB port and the recommended cable, try another PC or laptop.", "[Error] 3) Если ошибка остаётся после заднего USB-порта и рекомендованного кабеля, попробуйте другой ПК или ноутбук.", "[Error] 3) 背面USBポートと推奨ケーブルでも失敗する場合は、別のPCまたはノートPCで試してください。", "[Error] 3) 若已使用後方 USB 與建議線材仍失敗，請改用其他 PC 或筆電。", "[Error] 3) Nếu vẫn lỗi sau khi dùng cổng USB phía sau và cáp khuyến nghị, hãy thử PC hoặc laptop khác.", "[Error] 3) Αν αποτύχει ακόμα με πίσω USB και προτεινόμενο καλώδιο, δοκιμάστε άλλο PC ή laptop.", "[Error] 3) पीछे वाले USB पोर्ट और बताए गए केबल के बाद भी विफल हो, तो दूसरे PC या laptop से प्रयास करें।", "[Error] 3) თუ უკანა USB პორტით და რეკომენდებული კაბელითაც ვერ მოხერხდა, სცადეთ სხვა PC ან ლეპტოპი.", "[Error] 3) Als het nog steeds mislukt met de achterste USB-poort en aanbevolen kabel, probeer een andere pc of laptop.", "[Error] 3) إذا استمر الفشل بعد استخدام منفذ USB الخلفي والكابل الموصى به، جرّب كمبيوترًا أو حاسوبًا محمولًا آخر.", "[Error] 3) Si sigue fallando con el puerto USB trasero y el cable recomendado, pruebe otro PC o portátil."),
        "[OTA] OTA(업데이트) 비활성화 작업을 시작합니다." => lpm_lang_text(lang, "[OTA] Starting OTA update disable task.", "[OTA] Запуск отключения OTA-обновлений.", "[OTA] OTA（アップデート）無効化作業を開始します。", "[OTA] 開始停用 OTA 更新工作。", "[OTA] Bắt đầu tác vụ tắt cập nhật OTA.", "[OTA] Εκκίνηση εργασίας απενεργοποίησης OTA.", "[OTA] OTA अपडेट निष्क्रिय करने का कार्य शुरू हो रहा है।", "[OTA] OTA განახლების გამორთვის ამოცანა იწყება.", "[OTA] OTA-update uitschakelen wordt gestart.", "[OTA] بدء مهمة تعطيل تحديث OTA.", "[OTA] Iniciando tarea para desactivar OTA."),
        "[OTA] OTA(업데이트) 활성화 작업을 시작합니다." => lpm_lang_text(lang, "[OTA] Starting OTA update enable task.", "[OTA] Запуск включения OTA-обновлений.", "[OTA] OTA（アップデート）有効化作業を開始します。", "[OTA] 開始啟用 OTA 更新工作。", "[OTA] Bắt đầu tác vụ bật cập nhật OTA.", "[OTA] Εκκίνηση εργασίας ενεργοποίησης OTA.", "[OTA] OTA अपडेट सक्षम करने का कार्य शुरू हो रहा है।", "[OTA] OTA განახლების ჩართვის ამოცანა იწყება.", "[OTA] OTA-update inschakelen wordt gestart.", "[OTA] بدء مهمة تفعيل تحديث OTA.", "[OTA] Iniciando tarea para activar OTA."),
        "[Update] 새로운 업데이트 파일을 감지했습니다." => lpm_lang_text(lang, "[Update] A new update file was detected.", "[Update] Обнаружен новый файл обновления.", "[Update] 新しいアップデートファイルを検出しました。", "[Update] 已偵測到新的更新檔案。", "[Update] Đã phát hiện tệp cập nhật mới.", "[Update] Εντοπίστηκε νέο αρχείο ενημέρωσης.", "[Update] नई अपडेट फ़ाइल मिली है।", "[Update] აღმოჩენილია ახალი განახლების ფაილი.", "[Update] Nieuw updatebestand gedetecteerd.", "[Update] تم اكتشاف ملف تحديث جديد.", "[Update] Se detectó un nuevo archivo de actualización."),
        "[Update] 업데이트 확인 실패: {err}" => lpm_lang_text(lang, "[Update] Update check failed: {err}", "[Update] Ошибка проверки обновлений: {err}", "[Update] アップデート確認に失敗しました: {err}", "[Update] 更新檢查失敗：{err}", "[Update] Kiểm tra cập nhật thất bại: {err}", "[Update] Ο έλεγχος ενημέρωσης απέτυχε: {err}", "[Update] अपडेट जांच विफल: {err}", "[Update] განახლების შემოწმება ვერ მოხერხდა: {err}", "[Update] Controleren op updates mislukt: {err}", "[Update] فشل التحقق من التحديث: {err}", "[Update] Error al buscar actualizaciones: {err}"),
        "[Update] 이미 최신 버전을 사용 중입니다: LPMBox {}" => lpm_lang_text(lang, "[Update] You are already using the latest version: LPMBox {}", "[Update] Вы уже используете последнюю версию: LPMBox {}", "[Update] すでに最新バージョンを使用しています: LPMBox {}", "[Update] 您已使用最新版本：LPMBox {}", "[Update] Bạn đang dùng phiên bản mới nhất: LPMBox {}", "[Update] Χρησιμοποιείτε ήδη την πιο πρόσφατη έκδοση: LPMBox {}", "[Update] आप पहले से नवीनतम संस्करण उपयोग कर रहे हैं: LPMBox {}", "[Update] თქვენ უკვე იყენებთ უახლეს ვერსიას: LPMBox {}", "[Update] U gebruikt al de nieuwste versie: LPMBox {}", "[Update] أنت تستخدم أحدث إصدار بالفعل: LPMBox {}", "[Update] Ya está usando la última versión: LPMBox {}"),
        "[설정] 프로그램 언어를 선택했습니다: {language}" => lpm_lang_text(lang, "[Settings] Program language selected: {language}", "[Настройки] Выбран язык программы: {language}", "[設定] プログラム言語を選択しました: {language}", "[設定] 已選擇程式語言：{language}", "[Cài đặt] Đã chọn ngôn ngữ chương trình: {language}", "[Ρυθμίσεις] Επιλέχθηκε γλώσσα προγράμματος: {language}", "[सेटिंग्स] प्रोग्राम भाषा चुनी गई: {language}", "[პარამეტრები] არჩეულია პროგრამის ენა: {language}", "[Instellingen] Programmataal geselecteerd: {language}", "[الإعدادات] تم اختيار لغة البرنامج: {language}", "[Configuración] Idioma del programa seleccionado: {language}"),
        "[설정] 언어 설정 파일 경로: {}" => lpm_lang_text(lang, "[Settings] Language setting file path: {}", "[Настройки] Путь к файлу настройки языка: {}", "[設定] 言語設定ファイルのパス: {}", "[設定] 語言設定檔路徑：{}", "[Cài đặt] Đường dẫn tệp cài đặt ngôn ngữ: {}", "[Ρυθμίσεις] Διαδρομή αρχείου ρύθμισης γλώσσας: {}", "[सेटिंग्स] भाषा सेटिंग फ़ाइल पथ: {}", "[პარამეტრები] ენის პარამეტრის ფაილის ბილიკი: {}", "[Instellingen] Pad van taalinstellingenbestand: {}", "[الإعدادات] مسار ملف إعداد اللغة: {}", "[Configuración] Ruta del archivo de idioma: {}"),
        "proinfo 백업을 시작합니다." => lpm_lang_text(lang, "Starting proinfo backup.", "Запуск резервного копирования proinfo.", "proinfoバックアップを開始します。", "開始 proinfo 備份。", "Bắt đầu sao lưu proinfo.", "Έναρξη αντιγράφου proinfo.", "proinfo बैकअप शुरू हो रहा है।", "proinfo სარეზერვოს შექმნა იწყება.", "proinfo-back-up wordt gestart.", "بدء نسخ proinfo احتياطيًا.", "Iniciando copia de proinfo."),
        "proinfo 파티션 추출 중..." => lpm_lang_text(lang, "Extracting the proinfo partition...", "Извлечение раздела proinfo...", "proinfoパーティションを抽出中...", "正在擷取 proinfo 分割區...", "Đang trích xuất phân vùng proinfo...", "Εξαγωγή διαμερίσματος proinfo...", "proinfo partition निकाला जा रहा है...", "მიმდინარეობს proinfo დანაყოფის ამოღება...", "proinfo-partitie wordt geëxtraheerd...", "جارٍ استخراج قسم proinfo...", "Extrayendo la partición proinfo..."),
        "scatter XML 파싱: 성공 / root: {} / size: {} bytes" => lpm_lang_text(lang, "scatter XML parsing: success / root: {} / size: {} bytes", "Анализ scatter XML: успешно / root: {} / размер: {} bytes", "scatter XML解析: 成功 / root: {} / size: {} bytes", "scatter XML 解析：成功 / root：{} / 大小：{} bytes", "Phân tích scatter XML: thành công / root: {} / size: {} bytes", "Ανάλυση scatter XML: επιτυχία / root: {} / μέγεθος: {} bytes", "scatter XML parsing: सफल / root: {} / size: {} bytes", "scatter XML parsing: წარმატება / root: {} / size: {} bytes", "scatter XML parseren: geslaagd / root: {} / grootte: {} bytes", "تحليل scatter XML: نجاح / root: {} / size: {} bytes", "Análisis de scatter XML: correcto / root: {} / tamaño: {} bytes"),
        "partition 목록 읽기: 성공 / {}개" => lpm_lang_text(lang, "partition list read: success / {} items", "Чтение списка partition: успешно / {} шт.", "partition一覧読み取り: 成功 / {}件", "讀取 partition 清單：成功 / {} 個", "Đọc danh sách partition: thành công / {} mục", "Ανάγνωση λίστας partition: επιτυχία / {} στοιχεία", "partition सूची पढ़ना: सफल / {} आइटम", "partition სიის წაკითხვა: წარმატება / {} ელემენტი", "partition-lijst lezen: geslaagd / {} items", "قراءة قائمة partition: نجاح / {} عناصر", "Lectura de lista partition: correcta / {} elementos"),
        "partition 상세 정보 읽기: 성공 / {}개" => lpm_lang_text(lang, "partition detail read: success / {} items", "Чтение сведений partition: успешно / {} шт.", "partition詳細読み取り: 成功 / {}件", "讀取 partition 詳細：成功 / {} 個", "Đọc chi tiết partition: thành công / {} mục", "Ανάγνωση λεπτομερειών partition: επιτυχία / {} στοιχεία", "partition विवरण पढ़ना: सफल / {} आइटम", "partition დეტალების წაკითხვა: წარმატება / {} ელემენტი", "partitiondetails lezen: geslaagd / {} items", "قراءة تفاصيل partition: نجاح / {} عناصر", "Lectura de detalles partition: correcta / {} elementos"),
        "patch plan 생성: 성공 / {}개" => lpm_lang_text(lang, "patch plan creation: success / {} items", "Создание patch plan: успешно / {} шт.", "patch plan作成: 成功 / {}件", "建立 patch plan：成功 / {} 個", "Tạo patch plan: thành công / {} mục", "Δημιουργία patch plan: επιτυχία / {} στοιχεία", "patch plan बनाना: सफल / {} आइटम", "patch plan-ის შექმნა: წარმატება / {} ელემენტი", "patch plan maken: geslaagd / {} items", "إنشاء patch plan: نجاح / {} عناصر", "Creación de patch plan: correcta / {} elementos"),
        "patch plan 적용: 성공 / {}개" => lpm_lang_text(lang, "patch plan application: success / {} items", "Применение patch plan: успешно / {} шт.", "patch plan適用: 成功 / {}件", "套用 patch plan：成功 / {} 個", "Áp dụng patch plan: thành công / {} mục", "Εφαρμογή patch plan: επιτυχία / {} στοιχεία", "patch plan लागू करना: सफल / {} आइटम", "patch plan-ის გამოყენება: წარმატება / {} ელემენტი", "patch plan toepassen: geslaagd / {} items", "تطبيق patch plan: نجاح / {} عناصر", "Aplicación de patch plan: correcta / {} elementos"),
        "다른 버전 파일로 다시 시도해 주세요." => lpm_lang_text(lang, "Try again with a different version file.", "Повторите с файлом другой версии.", "別のバージョンのファイルで再試行してください。", "請使用其他版本檔案重試。", "Thử lại với tệp phiên bản khác.", "Δοκιμάστε ξανά με αρχείο άλλης έκδοσης.", "दूसरे संस्करण की फ़ाइल से फिर प्रयास करें।", "სხვა ვერსიის ფაილით სცადეთ თავიდან.", "Probeer opnieuw met een ander versiebestand.", "حاول مرة أخرى بملف إصدار آخر.", "Inténtelo con un archivo de otra versión."),
        "{} 후 다시 시도해주세요." => lpm_lang_text(lang, "Try again after {}.", "Повторите после: {}.", "{}の後に再試行してください。", "請在 {} 後重試。", "Thử lại sau khi {}.", "Δοκιμάστε ξανά μετά από: {}.", "{} के बाद फिर प्रयास करें।", "სცადეთ თავიდან ამის შემდეგ: {}.", "Probeer opnieuw na {}.", "حاول مرة أخرى بعد {}.", "Inténtelo de nuevo después de {}."),
        "성공 / {}개 모두 통과" => lpm_lang_text(lang, "Success / all {} passed", "Успешно / все {} пройдены", "成功 / {}件すべて通過", "成功 / {} 個全部通過", "Thành công / tất cả {} mục đã qua", "Επιτυχία / πέρασαν και τα {}", "सफल / सभी {} पास", "წარმატება / ყველა {} გავიდა", "Geslaagd / alle {} geslaagd", "نجاح / اجتازت كل العناصر {}", "Correcto / todos los {} aprobados"),
        "실패 / 통과 {}개 / 실패 {}개" => lpm_lang_text(lang, "Failed / passed {} / failed {}", "Ошибка / пройдено {} / ошибок {}", "失敗 / 通過 {}件 / 失敗 {}件", "失敗 / 通過 {} 個 / 失敗 {} 個", "Thất bại / đạt {} / lỗi {}", "Αποτυχία / πέρασαν {} / απέτυχαν {}", "विफल / पास {} / विफल {}", "ვერ შესრულდა / გავიდა {} / ვერ გავიდა {}", "Mislukt / geslaagd {} / mislukt {}", "فشل / نجح {} / فشل {}", "Error / aprobados {} / fallidos {}"),
        "선택된 image 폴더" => lpm_lang_text(lang, "Selected image folder", "Выбранная папка image", "選択されたimageフォルダー", "已選擇的 image 資料夾", "Thư mục image đã chọn", "Επιλεγμένος φάκελος image", "चुना गया image फ़ोल्डर", "არჩეული image საქაღალდე", "Geselecteerde image-map", "مجلد image المحدد", "Carpeta image seleccionada"),
        "올바른 image 폴더를\n선택해주세요." => lpm_lang_text(lang, "Please select a valid image folder.", "Выберите правильную папку image.", "正しいimageフォルダーを選択してください。", "請選擇正確的 image 資料夾。", "Vui lòng chọn đúng thư mục image.", "Επιλέξτε σωστό φάκελο image.", "कृपया सही image फ़ोल्डर चुनें।", "გთხოვთ აირჩიოთ სწორი image საქაღალდე.", "Selecteer een geldige image-map.", "يرجى اختيار مجلد image صحيح.", "Seleccione una carpeta image válida."),
        "펌웨어 검사 작업 스레드 오류: {err}" => lpm_lang_text(lang, "Firmware check worker thread error: {err}", "Ошибка потока проверки прошивки: {err}", "ファームウェア検査ワーカースレッドエラー: {err}", "韌體檢查工作執行緒錯誤：{err}", "Lỗi luồng kiểm tra firmware: {err}", "Σφάλμα νήματος ελέγχου firmware: {err}", "फर्मवेयर जांच worker thread error: {err}", "Firmware შემოწმების worker thread შეცდომა: {err}", "Fout in firmwarecontrolethread: {err}", "خطأ في مؤشر ترابط فحص firmware: {err}", "Error del hilo de comprobación de firmware: {err}"),
        "펌웨어 버전, 플랫폼, 모델명, 필수 partition 유효성, MTK 드라이버 설치 유/무를 검사합니다." => lpm_lang_text(lang, "Check firmware version, platform, model name, required partition validity, and MTK driver installation status.", "Проверка версии прошивки, платформы, модели, обязательных partition и установки драйвера MTK.", "ファームウェアバージョン、プラットフォーム、モデル名、必須partitionの有効性、MTKドライバーの有無を確認します。", "檢查韌體版本、平台、型號、必要 partition 有效性與 MTK 驅動程式安裝狀態。", "Kiểm tra phiên bản firmware, nền tảng, model, partition bắt buộc và trạng thái cài driver MTK.", "Έλεγχος έκδοσης firmware, πλατφόρμας, μοντέλου, υποχρεωτικών partition και εγκατάστασης οδηγού MTK.", "फर्मवेयर संस्करण, प्लेटफ़ॉर्म, मॉडल नाम, आवश्यक partition और MTK driver स्थिति जांचें।", "შემოწმდება firmware ვერსია, პლატფორმა, მოდელი, აუცილებელი partition და MTK დრაივერი.", "Controleert firmwareversie, platform, modelnaam, vereiste partition en MTK-driverstatus.", "التحقق من إصدار firmware والمنصة والطراز وصلاحية partition المطلوبة وحالة تثبيت برنامج تشغيل MTK.", "Comprueba versión de firmware, plataforma, modelo, partition obligatorias y estado del controlador MTK."),
        "필수 partition 상세: {}" => lpm_lang_text(lang, "Required partition details: {}", "Сведения об обязательных partition: {}", "必須partition詳細: {}", "必要 partition 詳細：{}", "Chi tiết partition bắt buộc: {}", "Λεπτομέρειες απαιτούμενων partition: {}", "आवश्यक partition विवरण: {}", "აუცილებელი partition დეტალები: {}", "Details van vereiste partition: {}", "تفاصيل partition المطلوبة: {}", "Detalles de partition obligatorias: {}"),
        "{} (실제 값과 다를 수 있음)" => lpm_lang_text(lang, "{} (may differ from the actual value)", "{} (может отличаться от фактического значения)", "{}（実際の値と異なる場合があります）", "{}（可能與實際值不同）", "{} (có thể khác giá trị thực tế)", "{} (μπορεί να διαφέρει από την πραγματική τιμή)", "{} (वास्तविक मान से भिन्न हो सकता है)", "{} (შეიძლება განსხვავდებოდეს რეალური მნიშვნელობისგან)", "{} (kan afwijken van de werkelijke waarde)", "{} (قد يختلف عن القيمة الفعلية)", "{} (puede diferir del valor real)"),
        "PreLoader 포트 감지 실패: 30초 안에 MediaTek PreLoader USB VCOM 포트를 찾지 못했습니다. 감지 문자열: {}" => lpm_lang_text(lang, "PreLoader port detection failed: MediaTek PreLoader USB VCOM port was not found within 30 seconds. Detected strings: {}", "Ошибка обнаружения порта PreLoader: порт MediaTek PreLoader USB VCOM не найден за 30 секунд. Обнаруженные строки: {}", "PreLoaderポート検出失敗: 30秒以内にMediaTek PreLoader USB VCOMポートが見つかりませんでした。検出文字列: {}", "PreLoader 連接埠偵測失敗：30 秒內找不到 MediaTek PreLoader USB VCOM 連接埠。偵測字串：{}", "Phát hiện cổng PreLoader thất bại: không tìm thấy MediaTek PreLoader USB VCOM trong 30 giây. Chuỗi phát hiện: {}", "Αποτυχία εντοπισμού θύρας PreLoader: δεν βρέθηκε MediaTek PreLoader USB VCOM σε 30 δευτερόλεπτα. Συμβολοσειρές: {}", "PreLoader पोर्ट पता लगाने में विफल: 30 सेकंड में MediaTek PreLoader USB VCOM पोर्ट नहीं मिला। पता चले strings: {}", "PreLoader პორტის აღმოჩენა ვერ მოხერხდა: 30 წამში MediaTek PreLoader USB VCOM პორტი ვერ მოიძებნა. აღმოჩენილი სტრიქონები: {}", "PreLoader-poortdetectie mislukt: MediaTek PreLoader USB VCOM-poort niet gevonden binnen 30 seconden. Gedetecteerde strings: {}", "فشل اكتشاف منفذ PreLoader: لم يتم العثور على MediaTek PreLoader USB VCOM خلال 30 ثانية. السلاسل المكتشفة: {}", "Error al detectar puerto PreLoader: no se encontró MediaTek PreLoader USB VCOM en 30 segundos. Cadenas detectadas: {}"),
        _ => return None,
    })
}

fn lpm_translate_stage6_phrasewise(lang: LanguageOption, content: String) -> String {
    if lang.is_korean() || !content.chars().any(|c| ('가'..='힣').contains(&c)) {
        return content;
    }

    let mut out = content;
    for (from, to) in lpm_stage6_phrase_pairs(lang) {
        out = out.replace(from, to);
    }
    out
}

fn lpm_stage6_phrase_pairs(lang: LanguageOption) -> &'static [(&'static str, &'static str)] {
    use LanguageOption::*;
    match lang {
        English => &[("1단계", "Step 1"), ("2단계", "Step 2"), ("3단계", "Step 3"), ("4단계", "Step 4"), ("5단계", "Step 5"), ("6단계", "Step 6"), ("7단계", "Step 7"), ("8단계", "Step 8"), ("제한 시간 30초", "timeout 30 seconds"), ("30초", "30 seconds"), ("감지 문자열", "detected strings"), ("성공", "success"), ("실패", "failed"), ("완료", "completed"), ("개 모두 통과", " items passed"), ("개", " items"), ("경고", "warnings"), ("오류", "errors"), ("현재", "current"), ("최신", "latest"), ("기준", "source"), ("설정", "Settings"), ("국가 코드 재설정", "Country Code Reset"), ("ROM 옵션", "ROM Options"), ("OTA(업데이트)", "OTA update"), ("비활성화 작업을 시작합니다", "disable task is starting"), ("활성화 작업을 시작합니다", "enable task is starting"), ("펌웨어", "firmware"), ("업데이트", "update"), ("루틴", "routine"), ("버전", "version"), ("모델명", "model name"), ("플랫폼", "platform"), ("필수", "required"), ("유효성", "validity"), ("유/무", "status"), ("검사합니다", "check"), ("검사를 시작합니다", "check is starting"), ("확인", "check"), ("선택이", "selection"), ("취소되었습니다", "was canceled"), ("선택됨", "selected"), ("선택", "select"), ("폴더", "folder"), ("파일", "file"), ("작업", "task"), ("드라이버 설치", "driver installation"), ("드라이버", "driver"), ("포트 감지", "port detection"), ("감지 중", "detecting"), ("감지 완료", "detected"), ("감지 실패", "detection failed"), ("후면", "rear"), ("노트북", "laptop"), ("상관 없음", "not applicable"), ("데이터 케이블", "data cable"), ("연결한 뒤", "after connecting"), ("다시 시도해주세요", "try again"), ("설명드린 케이블", "recommended cable"), ("시도해주세요", "try"), ("다른 PC 또는 노트북", "another PC or laptop"), ("작업 로그", "task log"), ("저장합니다", "saving"), ("저장", "save"), ("경로", "path"), ("열기 실패", "open failed"), ("페이지", "page"), ("릴리즈", "release"), ("이전 로그", "previous logs"), ("화면 렌더링 성능", "screen rendering performance"), ("생략되었습니다", "omitted"), ("전체 내용", "full contents"), ("로그 저장", "saving the log"), ("백업", "backup"), ("수정한", "modified"), ("선택한", "selected"), ("감지된", "detected"), ("연결한 기기", "connected device"), ("올바른", "valid"), ("재선택", "select again"), ("재시도", "retry"), ("충전", "charge"), ("이상", "or more"), ("후", "after"), ("실제 값과 다를 수 있음", "may differ from the actual value"), ("내수롬", "China ROM"), ("글로벌롬", "Global ROM")],
        Russian => &[("1단계", "Step 1"), ("2단계", "Step 2"), ("3단계", "Step 3"), ("4단계", "Step 4"), ("5단계", "Step 5"), ("6단계", "Step 6"), ("7단계", "Step 7"), ("8단계", "Step 8"), ("제한 시간 30초", "timeout 30 seconds"), ("30초", "30 seconds"), ("감지 문자열", "detected strings"), ("성공", "success"), ("실패", "failed"), ("완료", "completed"), ("개 모두 통과", " items passed"), ("개", " items"), ("경고", "warnings"), ("오류", "errors"), ("현재", "current"), ("최신", "latest"), ("기준", "source"), ("설정", "Settings"), ("국가 코드 재설정", "Country Code Reset"), ("ROM 옵션", "ROM Options"), ("OTA(업데이트)", "OTA update"), ("비활성화 작업을 시작합니다", "disable task is starting"), ("활성화 작업을 시작합니다", "enable task is starting"), ("펌웨어", "firmware"), ("업데이트", "update"), ("루틴", "routine"), ("버전", "version"), ("모델명", "model name"), ("플랫폼", "platform"), ("필수", "required"), ("유효성", "validity"), ("유/무", "status"), ("검사합니다", "check"), ("검사를 시작합니다", "check is starting"), ("확인", "check"), ("선택이", "selection"), ("취소되었습니다", "was canceled"), ("선택됨", "selected"), ("선택", "select"), ("폴더", "folder"), ("파일", "file"), ("작업", "task"), ("드라이버 설치", "driver installation"), ("드라이버", "driver"), ("포트 감지", "port detection"), ("감지 중", "detecting"), ("감지 완료", "detected"), ("감지 실패", "detection failed"), ("후면", "rear"), ("노트북", "laptop"), ("상관 없음", "not applicable"), ("데이터 케이블", "data cable"), ("연결한 뒤", "after connecting"), ("다시 시도해주세요", "try again"), ("설명드린 케이블", "recommended cable"), ("시도해주세요", "try"), ("다른 PC 또는 노트북", "another PC or laptop"), ("작업 로그", "task log"), ("저장합니다", "saving"), ("저장", "save"), ("경로", "path"), ("열기 실패", "open failed"), ("페이지", "page"), ("릴리즈", "release"), ("이전 로그", "previous logs"), ("화면 렌더링 성능", "screen rendering performance"), ("생략되었습니다", "omitted"), ("전체 내용", "full contents"), ("로그 저장", "saving the log"), ("백업", "backup"), ("수정한", "modified"), ("선택한", "selected"), ("감지된", "detected"), ("연결한 기기", "connected device"), ("올바른", "valid"), ("재선택", "select again"), ("재시도", "retry"), ("충전", "charge"), ("이상", "or more"), ("후", "after"), ("실제 값과 다를 수 있음", "may differ from the actual value"), ("내수롬", "China ROM"), ("글로벌롬", "Global ROM")],
        Japanese => &[("1단계", "ステップ1"), ("2단계", "ステップ2"), ("3단계", "ステップ3"), ("4단계", "ステップ4"), ("5단계", "ステップ5"), ("6단계", "ステップ6"), ("7단계", "ステップ7"), ("8단계", "ステップ8"), ("제한 시간 30초", "制限時間30秒"), ("30초", "30秒"), ("감지 문자열", "検出文字列"), ("성공", "成功"), ("실패", "失敗"), ("완료", "完了"), ("개", "件"), ("경고", "警告"), ("오류", "エラー"), ("현재", "現在"), ("최신", "最新"), ("기준", "基準"), ("설정", "設定"), ("국가 코드 재설정", "国コード再設定"), ("ROM 옵션", "ROMオプション"), ("OTA(업데이트)", "OTAアップデート"), ("펌웨어", "ファームウェア"), ("업데이트", "アップデート"), ("루틴", "ルーチン"), ("버전", "バージョン"), ("모델명", "モデル名"), ("플랫폼", "プラットフォーム"), ("필수", "必須"), ("검사합니다", "確認します"), ("검사를 시작합니다", "確認を開始します"), ("확인", "確認"), ("선택이", "選択が"), ("취소되었습니다", "キャンセルされました"), ("선택됨", "選択済み"), ("선택", "選択"), ("폴더", "フォルダー"), ("파일", "ファイル"), ("작업", "作業"), ("드라이버 설치", "ドライバーインストール"), ("드라이버", "ドライバー"), ("포트 감지", "ポート検出"), ("감지 중", "検出中"), ("감지 완료", "検出完了"), ("감지 실패", "検出失敗"), ("다시 시도해주세요", "再試行してください"), ("시도해주세요", "試してください"), ("작업 로그", "作業ログ"), ("저장합니다", "保存します"), ("저장", "保存"), ("경로", "パス"), ("열기 실패", "開くのに失敗"), ("백업", "バックアップ"), ("수정한", "修正した"), ("선택한", "選択した"), ("감지된", "検出された"), ("올바른", "正しい"), ("재선택", "再選択"), ("재시도", "再試行"), ("충전", "充電"), ("이상", "以上"), ("후", "後"), ("실제 값과 다를 수 있음", "実際の値と異なる場合があります"), ("내수롬", "中国版ROM"), ("글로벌롬", "グローバルROM")],
        TraditionalChinese => &[("1단계", "步驟 1"), ("2단계", "步驟 2"), ("3단계", "步驟 3"), ("4단계", "步驟 4"), ("5단계", "步驟 5"), ("6단계", "步驟 6"), ("7단계", "步驟 7"), ("8단계", "步驟 8"), ("제한 시간 30초", "限制時間 30 秒"), ("30초", "30 秒"), ("감지 문자열", "偵測字串"), ("성공", "成功"), ("실패", "失敗"), ("완료", "完成"), ("개", "個"), ("경고", "警告"), ("오류", "錯誤"), ("현재", "目前"), ("최신", "最新"), ("기준", "依據"), ("설정", "設定"), ("국가 코드 재설정", "國家代碼重設"), ("ROM 옵션", "ROM 選項"), ("OTA(업데이트)", "OTA 更新"), ("펌웨어", "韌體"), ("업데이트", "更新"), ("루틴", "流程"), ("버전", "版本"), ("모델명", "型號"), ("플랫폼", "平台"), ("필수", "必要"), ("검사합니다", "檢查"), ("검사를 시작합니다", "開始檢查"), ("확인", "確認"), ("선택이", "選擇"), ("취소되었습니다", "已取消"), ("선택됨", "已選擇"), ("선택", "選擇"), ("폴더", "資料夾"), ("파일", "檔案"), ("작업", "工作"), ("드라이버 설치", "驅動程式安裝"), ("드라이버", "驅動程式"), ("포트 감지", "連接埠偵測"), ("감지 중", "偵測中"), ("감지 완료", "偵測完成"), ("감지 실패", "偵測失敗"), ("다시 시도해주세요", "請重試"), ("시도해주세요", "請嘗試"), ("작업 로그", "工作日誌"), ("저장합니다", "儲存"), ("저장", "儲存"), ("경로", "路徑"), ("열기 실패", "開啟失敗"), ("백업", "備份"), ("수정한", "已修改的"), ("선택한", "已選擇的"), ("감지된", "已偵測的"), ("올바른", "正確的"), ("재선택", "重新選擇"), ("재시도", "重試"), ("충전", "充電"), ("이상", "以上"), ("후", "後"), ("실제 값과 다를 수 있음", "可能與實際值不同"), ("내수롬", "中國版 ROM"), ("글로벌롬", "全球版 ROM")],
        Vietnamese => &[("1단계", "Step 1"), ("2단계", "Step 2"), ("3단계", "Step 3"), ("4단계", "Step 4"), ("5단계", "Step 5"), ("6단계", "Step 6"), ("7단계", "Step 7"), ("8단계", "Step 8"), ("제한 시간 30초", "timeout 30 seconds"), ("30초", "30 seconds"), ("감지 문자열", "detected strings"), ("성공", "success"), ("실패", "failed"), ("완료", "completed"), ("개 모두 통과", " items passed"), ("개", " items"), ("경고", "warnings"), ("오류", "errors"), ("현재", "current"), ("최신", "latest"), ("기준", "source"), ("설정", "Settings"), ("국가 코드 재설정", "Country Code Reset"), ("ROM 옵션", "ROM Options"), ("OTA(업데이트)", "OTA update"), ("비활성화 작업을 시작합니다", "disable task is starting"), ("활성화 작업을 시작합니다", "enable task is starting"), ("펌웨어", "firmware"), ("업데이트", "update"), ("루틴", "routine"), ("버전", "version"), ("모델명", "model name"), ("플랫폼", "platform"), ("필수", "required"), ("유효성", "validity"), ("유/무", "status"), ("검사합니다", "check"), ("검사를 시작합니다", "check is starting"), ("확인", "check"), ("선택이", "selection"), ("취소되었습니다", "was canceled"), ("선택됨", "selected"), ("선택", "select"), ("폴더", "folder"), ("파일", "file"), ("작업", "task"), ("드라이버 설치", "driver installation"), ("드라이버", "driver"), ("포트 감지", "port detection"), ("감지 중", "detecting"), ("감지 완료", "detected"), ("감지 실패", "detection failed"), ("후면", "rear"), ("노트북", "laptop"), ("상관 없음", "not applicable"), ("데이터 케이블", "data cable"), ("연결한 뒤", "after connecting"), ("다시 시도해주세요", "try again"), ("설명드린 케이블", "recommended cable"), ("시도해주세요", "try"), ("다른 PC 또는 노트북", "another PC or laptop"), ("작업 로그", "task log"), ("저장합니다", "saving"), ("저장", "save"), ("경로", "path"), ("열기 실패", "open failed"), ("페이지", "page"), ("릴리즈", "release"), ("이전 로그", "previous logs"), ("화면 렌더링 성능", "screen rendering performance"), ("생략되었습니다", "omitted"), ("전체 내용", "full contents"), ("로그 저장", "saving the log"), ("백업", "backup"), ("수정한", "modified"), ("선택한", "selected"), ("감지된", "detected"), ("연결한 기기", "connected device"), ("올바른", "valid"), ("재선택", "select again"), ("재시도", "retry"), ("충전", "charge"), ("이상", "or more"), ("후", "after"), ("실제 값과 다를 수 있음", "may differ from the actual value"), ("내수롬", "China ROM"), ("글로벌롬", "Global ROM")],
        Greek => &[("1단계", "Step 1"), ("2단계", "Step 2"), ("3단계", "Step 3"), ("4단계", "Step 4"), ("5단계", "Step 5"), ("6단계", "Step 6"), ("7단계", "Step 7"), ("8단계", "Step 8"), ("제한 시간 30초", "timeout 30 seconds"), ("30초", "30 seconds"), ("감지 문자열", "detected strings"), ("성공", "success"), ("실패", "failed"), ("완료", "completed"), ("개 모두 통과", " items passed"), ("개", " items"), ("경고", "warnings"), ("오류", "errors"), ("현재", "current"), ("최신", "latest"), ("기준", "source"), ("설정", "Settings"), ("국가 코드 재설정", "Country Code Reset"), ("ROM 옵션", "ROM Options"), ("OTA(업데이트)", "OTA update"), ("비활성화 작업을 시작합니다", "disable task is starting"), ("활성화 작업을 시작합니다", "enable task is starting"), ("펌웨어", "firmware"), ("업데이트", "update"), ("루틴", "routine"), ("버전", "version"), ("모델명", "model name"), ("플랫폼", "platform"), ("필수", "required"), ("유효성", "validity"), ("유/무", "status"), ("검사합니다", "check"), ("검사를 시작합니다", "check is starting"), ("확인", "check"), ("선택이", "selection"), ("취소되었습니다", "was canceled"), ("선택됨", "selected"), ("선택", "select"), ("폴더", "folder"), ("파일", "file"), ("작업", "task"), ("드라이버 설치", "driver installation"), ("드라이버", "driver"), ("포트 감지", "port detection"), ("감지 중", "detecting"), ("감지 완료", "detected"), ("감지 실패", "detection failed"), ("후면", "rear"), ("노트북", "laptop"), ("상관 없음", "not applicable"), ("데이터 케이블", "data cable"), ("연결한 뒤", "after connecting"), ("다시 시도해주세요", "try again"), ("설명드린 케이블", "recommended cable"), ("시도해주세요", "try"), ("다른 PC 또는 노트북", "another PC or laptop"), ("작업 로그", "task log"), ("저장합니다", "saving"), ("저장", "save"), ("경로", "path"), ("열기 실패", "open failed"), ("페이지", "page"), ("릴리즈", "release"), ("이전 로그", "previous logs"), ("화면 렌더링 성능", "screen rendering performance"), ("생략되었습니다", "omitted"), ("전체 내용", "full contents"), ("로그 저장", "saving the log"), ("백업", "backup"), ("수정한", "modified"), ("선택한", "selected"), ("감지된", "detected"), ("연결한 기기", "connected device"), ("올바른", "valid"), ("재선택", "select again"), ("재시도", "retry"), ("충전", "charge"), ("이상", "or more"), ("후", "after"), ("실제 값과 다를 수 있음", "may differ from the actual value"), ("내수롬", "China ROM"), ("글로벌롬", "Global ROM")],
        Hindi => &[("1단계", "Step 1"), ("2단계", "Step 2"), ("3단계", "Step 3"), ("4단계", "Step 4"), ("5단계", "Step 5"), ("6단계", "Step 6"), ("7단계", "Step 7"), ("8단계", "Step 8"), ("제한 시간 30초", "timeout 30 seconds"), ("30초", "30 seconds"), ("감지 문자열", "detected strings"), ("성공", "success"), ("실패", "failed"), ("완료", "completed"), ("개 모두 통과", " items passed"), ("개", " items"), ("경고", "warnings"), ("오류", "errors"), ("현재", "current"), ("최신", "latest"), ("기준", "source"), ("설정", "Settings"), ("국가 코드 재설정", "Country Code Reset"), ("ROM 옵션", "ROM Options"), ("OTA(업데이트)", "OTA update"), ("비활성화 작업을 시작합니다", "disable task is starting"), ("활성화 작업을 시작합니다", "enable task is starting"), ("펌웨어", "firmware"), ("업데이트", "update"), ("루틴", "routine"), ("버전", "version"), ("모델명", "model name"), ("플랫폼", "platform"), ("필수", "required"), ("유효성", "validity"), ("유/무", "status"), ("검사합니다", "check"), ("검사를 시작합니다", "check is starting"), ("확인", "check"), ("선택이", "selection"), ("취소되었습니다", "was canceled"), ("선택됨", "selected"), ("선택", "select"), ("폴더", "folder"), ("파일", "file"), ("작업", "task"), ("드라이버 설치", "driver installation"), ("드라이버", "driver"), ("포트 감지", "port detection"), ("감지 중", "detecting"), ("감지 완료", "detected"), ("감지 실패", "detection failed"), ("후면", "rear"), ("노트북", "laptop"), ("상관 없음", "not applicable"), ("데이터 케이블", "data cable"), ("연결한 뒤", "after connecting"), ("다시 시도해주세요", "try again"), ("설명드린 케이블", "recommended cable"), ("시도해주세요", "try"), ("다른 PC 또는 노트북", "another PC or laptop"), ("작업 로그", "task log"), ("저장합니다", "saving"), ("저장", "save"), ("경로", "path"), ("열기 실패", "open failed"), ("페이지", "page"), ("릴리즈", "release"), ("이전 로그", "previous logs"), ("화면 렌더링 성능", "screen rendering performance"), ("생략되었습니다", "omitted"), ("전체 내용", "full contents"), ("로그 저장", "saving the log"), ("백업", "backup"), ("수정한", "modified"), ("선택한", "selected"), ("감지된", "detected"), ("연결한 기기", "connected device"), ("올바른", "valid"), ("재선택", "select again"), ("재시도", "retry"), ("충전", "charge"), ("이상", "or more"), ("후", "after"), ("실제 값과 다를 수 있음", "may differ from the actual value"), ("내수롬", "China ROM"), ("글로벌롬", "Global ROM")],
        Georgian => &[("1단계", "Step 1"), ("2단계", "Step 2"), ("3단계", "Step 3"), ("4단계", "Step 4"), ("5단계", "Step 5"), ("6단계", "Step 6"), ("7단계", "Step 7"), ("8단계", "Step 8"), ("제한 시간 30초", "timeout 30 seconds"), ("30초", "30 seconds"), ("감지 문자열", "detected strings"), ("성공", "success"), ("실패", "failed"), ("완료", "completed"), ("개 모두 통과", " items passed"), ("개", " items"), ("경고", "warnings"), ("오류", "errors"), ("현재", "current"), ("최신", "latest"), ("기준", "source"), ("설정", "Settings"), ("국가 코드 재설정", "Country Code Reset"), ("ROM 옵션", "ROM Options"), ("OTA(업데이트)", "OTA update"), ("비활성화 작업을 시작합니다", "disable task is starting"), ("활성화 작업을 시작합니다", "enable task is starting"), ("펌웨어", "firmware"), ("업데이트", "update"), ("루틴", "routine"), ("버전", "version"), ("모델명", "model name"), ("플랫폼", "platform"), ("필수", "required"), ("유효성", "validity"), ("유/무", "status"), ("검사합니다", "check"), ("검사를 시작합니다", "check is starting"), ("확인", "check"), ("선택이", "selection"), ("취소되었습니다", "was canceled"), ("선택됨", "selected"), ("선택", "select"), ("폴더", "folder"), ("파일", "file"), ("작업", "task"), ("드라이버 설치", "driver installation"), ("드라이버", "driver"), ("포트 감지", "port detection"), ("감지 중", "detecting"), ("감지 완료", "detected"), ("감지 실패", "detection failed"), ("후면", "rear"), ("노트북", "laptop"), ("상관 없음", "not applicable"), ("데이터 케이블", "data cable"), ("연결한 뒤", "after connecting"), ("다시 시도해주세요", "try again"), ("설명드린 케이블", "recommended cable"), ("시도해주세요", "try"), ("다른 PC 또는 노트북", "another PC or laptop"), ("작업 로그", "task log"), ("저장합니다", "saving"), ("저장", "save"), ("경로", "path"), ("열기 실패", "open failed"), ("페이지", "page"), ("릴리즈", "release"), ("이전 로그", "previous logs"), ("화면 렌더링 성능", "screen rendering performance"), ("생략되었습니다", "omitted"), ("전체 내용", "full contents"), ("로그 저장", "saving the log"), ("백업", "backup"), ("수정한", "modified"), ("선택한", "selected"), ("감지된", "detected"), ("연결한 기기", "connected device"), ("올바른", "valid"), ("재선택", "select again"), ("재시도", "retry"), ("충전", "charge"), ("이상", "or more"), ("후", "after"), ("실제 값과 다를 수 있음", "may differ from the actual value"), ("내수롬", "China ROM"), ("글로벌롬", "Global ROM")],
        Dutch => &[("1단계", "Step 1"), ("2단계", "Step 2"), ("3단계", "Step 3"), ("4단계", "Step 4"), ("5단계", "Step 5"), ("6단계", "Step 6"), ("7단계", "Step 7"), ("8단계", "Step 8"), ("제한 시간 30초", "timeout 30 seconds"), ("30초", "30 seconds"), ("감지 문자열", "detected strings"), ("성공", "success"), ("실패", "failed"), ("완료", "completed"), ("개 모두 통과", " items passed"), ("개", " items"), ("경고", "warnings"), ("오류", "errors"), ("현재", "current"), ("최신", "latest"), ("기준", "source"), ("설정", "Settings"), ("국가 코드 재설정", "Country Code Reset"), ("ROM 옵션", "ROM Options"), ("OTA(업데이트)", "OTA update"), ("비활성화 작업을 시작합니다", "disable task is starting"), ("활성화 작업을 시작합니다", "enable task is starting"), ("펌웨어", "firmware"), ("업데이트", "update"), ("루틴", "routine"), ("버전", "version"), ("모델명", "model name"), ("플랫폼", "platform"), ("필수", "required"), ("유효성", "validity"), ("유/무", "status"), ("검사합니다", "check"), ("검사를 시작합니다", "check is starting"), ("확인", "check"), ("선택이", "selection"), ("취소되었습니다", "was canceled"), ("선택됨", "selected"), ("선택", "select"), ("폴더", "folder"), ("파일", "file"), ("작업", "task"), ("드라이버 설치", "driver installation"), ("드라이버", "driver"), ("포트 감지", "port detection"), ("감지 중", "detecting"), ("감지 완료", "detected"), ("감지 실패", "detection failed"), ("후면", "rear"), ("노트북", "laptop"), ("상관 없음", "not applicable"), ("데이터 케이블", "data cable"), ("연결한 뒤", "after connecting"), ("다시 시도해주세요", "try again"), ("설명드린 케이블", "recommended cable"), ("시도해주세요", "try"), ("다른 PC 또는 노트북", "another PC or laptop"), ("작업 로그", "task log"), ("저장합니다", "saving"), ("저장", "save"), ("경로", "path"), ("열기 실패", "open failed"), ("페이지", "page"), ("릴리즈", "release"), ("이전 로그", "previous logs"), ("화면 렌더링 성능", "screen rendering performance"), ("생략되었습니다", "omitted"), ("전체 내용", "full contents"), ("로그 저장", "saving the log"), ("백업", "backup"), ("수정한", "modified"), ("선택한", "selected"), ("감지된", "detected"), ("연결한 기기", "connected device"), ("올바른", "valid"), ("재선택", "select again"), ("재시도", "retry"), ("충전", "charge"), ("이상", "or more"), ("후", "after"), ("실제 값과 다를 수 있음", "may differ from the actual value"), ("내수롬", "China ROM"), ("글로벌롬", "Global ROM")],
        Arabic => &[("1단계", "Step 1"), ("2단계", "Step 2"), ("3단계", "Step 3"), ("4단계", "Step 4"), ("5단계", "Step 5"), ("6단계", "Step 6"), ("7단계", "Step 7"), ("8단계", "Step 8"), ("제한 시간 30초", "timeout 30 seconds"), ("30초", "30 seconds"), ("감지 문자열", "detected strings"), ("성공", "success"), ("실패", "failed"), ("완료", "completed"), ("개 모두 통과", " items passed"), ("개", " items"), ("경고", "warnings"), ("오류", "errors"), ("현재", "current"), ("최신", "latest"), ("기준", "source"), ("설정", "Settings"), ("국가 코드 재설정", "Country Code Reset"), ("ROM 옵션", "ROM Options"), ("OTA(업데이트)", "OTA update"), ("비활성화 작업을 시작합니다", "disable task is starting"), ("활성화 작업을 시작합니다", "enable task is starting"), ("펌웨어", "firmware"), ("업데이트", "update"), ("루틴", "routine"), ("버전", "version"), ("모델명", "model name"), ("플랫폼", "platform"), ("필수", "required"), ("유효성", "validity"), ("유/무", "status"), ("검사합니다", "check"), ("검사를 시작합니다", "check is starting"), ("확인", "check"), ("선택이", "selection"), ("취소되었습니다", "was canceled"), ("선택됨", "selected"), ("선택", "select"), ("폴더", "folder"), ("파일", "file"), ("작업", "task"), ("드라이버 설치", "driver installation"), ("드라이버", "driver"), ("포트 감지", "port detection"), ("감지 중", "detecting"), ("감지 완료", "detected"), ("감지 실패", "detection failed"), ("후면", "rear"), ("노트북", "laptop"), ("상관 없음", "not applicable"), ("데이터 케이블", "data cable"), ("연결한 뒤", "after connecting"), ("다시 시도해주세요", "try again"), ("설명드린 케이블", "recommended cable"), ("시도해주세요", "try"), ("다른 PC 또는 노트북", "another PC or laptop"), ("작업 로그", "task log"), ("저장합니다", "saving"), ("저장", "save"), ("경로", "path"), ("열기 실패", "open failed"), ("페이지", "page"), ("릴리즈", "release"), ("이전 로그", "previous logs"), ("화면 렌더링 성능", "screen rendering performance"), ("생략되었습니다", "omitted"), ("전체 내용", "full contents"), ("로그 저장", "saving the log"), ("백업", "backup"), ("수정한", "modified"), ("선택한", "selected"), ("감지된", "detected"), ("연결한 기기", "connected device"), ("올바른", "valid"), ("재선택", "select again"), ("재시도", "retry"), ("충전", "charge"), ("이상", "or more"), ("후", "after"), ("실제 값과 다를 수 있음", "may differ from the actual value"), ("내수롬", "China ROM"), ("글로벌롬", "Global ROM")],
        Spanish => &[("1단계", "Paso 1"), ("2단계", "Paso 2"), ("3단계", "Paso 3"), ("4단계", "Paso 4"), ("5단계", "Paso 5"), ("6단계", "Paso 6"), ("7단계", "Paso 7"), ("8단계", "Paso 8"), ("제한 시간 30초", "límite de 30 segundos"), ("30초", "30 segundos"), ("감지 문자열", "cadenas detectadas"), ("성공", "correcto"), ("실패", "fallido"), ("완료", "completado"), ("개", " elementos"), ("경고", "advertencias"), ("오류", "errores"), ("현재", "actual"), ("최신", "último"), ("기준", "origen"), ("설정", "Configuración"), ("국가 코드 재설정", "Restablecer código de país"), ("ROM 옵션", "Opciones ROM"), ("OTA(업데이트)", "Actualización OTA"), ("펌웨어", "firmware"), ("업데이트", "actualización"), ("루틴", "rutina"), ("버전", "versión"), ("모델명", "modelo"), ("플랫폼", "plataforma"), ("필수", "obligatorio"), ("검사합니다", "comprueba"), ("검사를 시작합니다", "inicia la comprobación"), ("확인", "comprobar"), ("선택이", "selección"), ("취소되었습니다", "se canceló"), ("선택됨", "seleccionado"), ("선택", "seleccionar"), ("폴더", "carpeta"), ("파일", "archivo"), ("작업", "tarea"), ("드라이버 설치", "instalación del controlador"), ("드라이버", "controlador"), ("포트 감지", "detección de puerto"), ("감지 중", "detectando"), ("감지 완료", "detectado"), ("감지 실패", "detección fallida"), ("다시 시도해주세요", "inténtelo de nuevo"), ("시도해주세요", "intente"), ("작업 로그", "registro de tarea"), ("저장합니다", "guardar"), ("저장", "guardar"), ("경로", "ruta"), ("열기 실패", "error al abrir"), ("백업", "copia"), ("수정한", "modificado"), ("선택한", "seleccionado"), ("감지된", "detectado"), ("올바른", "válido"), ("재선택", "volver a seleccionar"), ("재시도", "reintentar"), ("충전", "cargar"), ("이상", "o más"), ("후", "después"), ("실제 값과 다를 수 있음", "puede diferir del valor real"), ("내수롬", "ROM china"), ("글로벌롬", "ROM global")],
        Korean => &[],
    }
}

fn lpm_window_icon() -> Option<window::Icon> {
    window::icon::from_file_data(APP_ICON_PNG_BYTES, Some(image_crate::ImageFormat::Png)).ok()
}

fn lpm_font() -> Font {
    Font::with_name(LPM_FONT_FAMILY)
}

fn lpm_bold_font() -> Font {
    Font {
        weight: iced::font::Weight::Bold,
        ..Font::with_name(LPM_FONT_FAMILY)
    }
}

struct App {
    image_dir: Option<PathBuf>,
    busy: bool,
    log_lines: Vec<LogLine>,
    log_text_cache: String,
    log_display_rows_cache: Vec<String>,
    log_cache_dirty: bool,
    log_refresh_suspended: bool,

    rom_firmware_info: Option<FirmwareInfo>,
    rom_firmware_error: Option<String>,
    rom_mtk_driver_installed: Option<bool>,
    rom_mtk_driver_error: Option<String>,
    rom_vc_runtime_status: Option<lpmbox_device::VcRuntimeStatus>,
    rom_vc_runtime_error: Option<String>,

    live_rx: Option<Receiver<ProinfoLiveEvent>>,
    progress_line_indices: HashMap<String, usize>,
    active_spinners: HashMap<String, SpinnerState>,
    spinner_tick: usize,
    rom_check_loading_frame: usize,
    last_spft_stage: Option<String>,

    active_nav: NavPage,
    sidebar_expanded: bool,
    sidebar_anim: f32,
    sidebar_velocity: f32,

    dashboard_info: lpmbox_device::DashboardDeviceInfo,
    dashboard_refreshing: bool,
    battery_progress_level: Option<u8>,
    battery_progress_handle: iced::widget::image::Handle,

    rom_show_routine_select: bool,
    rom_option_target: Option<RomSlideTarget>,
    running_rom_target: Option<RomSlideTarget>,
    rom_option_data_wipe: bool,
    rom_option_data_wipe_locked: bool,
    rom_option_country_code: Option<String>,
    rom_country_popup_open: bool,

    dashboard_model_image: DashboardModelImage,
    dashboard_model_missing_since: Option<Instant>,
    model_tb375_handle: iced::widget::image::Handle,
    model_tb365_handle: iced::widget::image::Handle,
    model_tb335_handle: iced::widget::image::Handle,
    rom_install_icon_handle: iced::widget::image::Handle,
    rom_update_icon_handle: iced::widget::image::Handle,
    folder_select_icon_handle: iced::widget::image::Handle,
    folder_check_icon_handle: iced::widget::image::Handle,
    tablet_check_icon_handle: iced::widget::image::Handle,
    tablet_x_icon_handle: iced::widget::image::Handle,
    tablet_fix_icon_handle: iced::widget::image::Handle,
    slide_button_handle: iced::widget::image::Handle,
    warning_icon_handle: iced::widget::image::Handle,
    nav_home_handle: iced::widget::image::Handle,
    nav_refresh_handle: iced::widget::image::Handle,
    nav_tab_settings_handle: iced::widget::image::Handle,
    nav_firmware_download_handle: iced::widget::image::Handle,
    nav_qna_handle: iced::widget::image::Handle,
    nav_log_handle: iced::widget::image::Handle,
    nav_settings_handle: iced::widget::image::Handle,
    loading_progress_handles: Vec<iced::widget::image::Handle>,
    rom_install_hover_since: Option<Instant>,
    rom_update_hover_since: Option<Instant>,
    rom_reinstall_hover_since: Option<Instant>,
    rom_install_slide_width: f32,
    rom_update_slide_width: f32,
    rom_reinstall_slide_width: f32,
    rom_install_slide_velocity: f32,
    rom_update_slide_velocity: f32,
    rom_reinstall_slide_velocity: f32,

    rom_country_code_modal_open: bool,
    rom_selected_country_code: Option<&'static str>,
    rom_country_code_search: String,
    additional_country_reset_pending: bool,
    settings_language: LanguageOption,
    active_log_flow: Option<RuntimeFlowKind>,
    program_update_checking: bool,
    dashboard_update_notice: Option<ProgramUpdateCheckResult>,
}

#[derive(Debug, Clone)]
struct SpinnerState {
    line_index: usize,
    base_message: String,
}

#[derive(Debug, Clone)]
struct LogLine {
    timestamp: String,
    message: String,
}

#[derive(Debug, Clone)]
struct DashboardSnapshot {
    info: lpmbox_device::DashboardDeviceInfo,
}

#[derive(Debug, Clone)]
struct RomFirmwareCheckResult {
    firmware: FirmwareInfo,
    mtk_driver_installed: Option<bool>,
    mtk_driver_error: Option<String>,
    vc_runtime_status: Option<lpmbox_device::VcRuntimeStatus>,
    vc_runtime_error: Option<String>,
}

#[derive(Debug, Clone)]
struct ProgramUpdateCheckResult {
    current_version: String,
    latest_version: String,
    update_available: bool,
    release_url: String,
    asset_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DashboardModelImage {
    Unknown,
    Tb375,
    Tb365,
    Tb335,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavPage {
    Dashboard,
    Rom,
    Backup,
    Driver,
    FirmwareDownload,
    Qna,
    Log,
    Settings,
}


fn lpm_translate_exact_stage5(lang: LanguageOption, key: &str) -> Option<&'static str> {
    use LanguageOption::*;
    let tr = |s: &'static str| -> Option<&'static str> { Some(s) };
    match key {
        "저장된 언어 설정" => match lang { English => tr("saved language setting"), Russian => tr("сохранённая настройка языка"), Japanese => tr("保存された言語設定"), TraditionalChinese => tr("已儲存的語言設定"), Vietnamese => tr("cài đặt ngôn ngữ đã lưu"), Greek => tr("αποθηκευμένη ρύθμιση γλώσσας"), Hindi => tr("सहेजी गई भाषा सेटिंग"), Georgian => tr("შენახული ენის პარამეტრი"), Dutch => tr("opgeslagen taalinstelling"), Arabic => tr("إعداد اللغة المحفوظ"), Spanish => tr("configuración de idioma guardada"), Korean => None },
        "Windows OS 언어" => match lang { English => tr("Windows OS language"), Russian => tr("язык Windows"), Japanese => tr("Windows OSの言語"), TraditionalChinese => tr("Windows 作業系統語言"), Vietnamese => tr("ngôn ngữ Windows"), Greek => tr("γλώσσα των Windows"), Hindi => tr("Windows OS भाषा"), Georgian => tr("Windows OS-ის ენა"), Dutch => tr("Windows OS-taal"), Arabic => tr("لغة نظام Windows"), Spanish => tr("idioma de Windows"), Korean => None },
        "기본값 English" => match lang { English => tr("default English"), Russian => tr("English по умолчанию"), Japanese => tr("既定のEnglish"), TraditionalChinese => tr("預設 English"), Vietnamese => tr("English mặc định"), Greek => tr("προεπιλογή English"), Hindi => tr("डिफ़ॉल्ट English"), Georgian => tr("ნაგულისხმევი English"), Dutch => tr("standaard English"), Arabic => tr("English الافتراضي"), Spanish => tr("English predeterminado"), Korean => None },
        "유지" => match lang { English => tr("Keep"), Russian => tr("Сохранить"), Japanese => tr("維持"), TraditionalChinese => tr("保留"), Vietnamese => tr("Giữ"), Greek => tr("Διατήρηση"), Hindi => tr("रखें"), Georgian => tr("შენარჩუნება"), Dutch => tr("Behouden"), Arabic => tr("احتفاظ"), Spanish => tr("Mantener"), Korean => None },
        "초기화" => match lang { English => tr("Wipe"), Russian => tr("Сброс"), Japanese => tr("初期化"), TraditionalChinese => tr("清除"), Vietnamese => tr("Xóa"), Greek => tr("Διαγραφή"), Hindi => tr("वाइप"), Georgian => tr("გასუფთავება"), Dutch => tr("Wissen"), Arabic => tr("مسح"), Spanish => tr("Borrar"), Korean => None },
        "있음" => match lang { English => tr("Yes"), Russian => tr("Есть"), Japanese => tr("あり"), TraditionalChinese => tr("有"), Vietnamese => tr("Có"), Greek => tr("Ναι"), Hindi => tr("हाँ"), Georgian => tr("დიახ"), Dutch => tr("Ja"), Arabic => tr("نعم"), Spanish => tr("Sí"), Korean => None },
        "없음" => match lang { English => tr("No"), Russian => tr("Нет"), Japanese => tr("なし"), TraditionalChinese => tr("無"), Vietnamese => tr("Không"), Greek => tr("Όχι"), Hindi => tr("नहीं"), Georgian => tr("არა"), Dutch => tr("Nee"), Arabic => tr("لا"), Spanish => tr("No"), Korean => None },
        "성공" => match lang { English => tr("Success"), Russian => tr("Успешно"), Japanese => tr("成功"), TraditionalChinese => tr("成功"), Vietnamese => tr("Thành công"), Greek => tr("Επιτυχία"), Hindi => tr("सफल"), Georgian => tr("წარმატება"), Dutch => tr("Geslaagd"), Arabic => tr("نجاح"), Spanish => tr("Correcto"), Korean => None },
        "실패" => match lang { English => tr("Failed"), Russian => tr("Ошибка"), Japanese => tr("失敗"), TraditionalChinese => tr("失敗"), Vietnamese => tr("Thất bại"), Greek => tr("Αποτυχία"), Hindi => tr("विफल"), Georgian => tr("ვერ მოხერხდა"), Dutch => tr("Mislukt"), Arabic => tr("فشل"), Spanish => tr("Error"), Korean => None },
        "사용 가능" => match lang { English => tr("Available"), Russian => tr("Доступно"), Japanese => tr("使用可能"), TraditionalChinese => tr("可用"), Vietnamese => tr("Có thể dùng"), Greek => tr("Διαθέσιμο"), Hindi => tr("उपलब्ध"), Georgian => tr("ხელმისაწვდომია"), Dutch => tr("Beschikbaar"), Arabic => tr("متاح"), Spanish => tr("Disponible"), Korean => None },
        "사용 불가" => match lang { English => tr("Unavailable"), Russian => tr("Недоступно"), Japanese => tr("使用不可"), TraditionalChinese => tr("不可用"), Vietnamese => tr("Không thể dùng"), Greek => tr("Μη διαθέσιμο"), Hindi => tr("उपलब्ध नहीं"), Georgian => tr("მიუწვდომელია"), Dutch => tr("Niet beschikbaar"), Arabic => tr("غير متاح"), Spanish => tr("No disponible"), Korean => None },
        "적용됨" => match lang { English => tr("Applied"), Russian => tr("Применено"), Japanese => tr("適用済み"), TraditionalChinese => tr("已套用"), Vietnamese => tr("Đã áp dụng"), Greek => tr("Εφαρμόστηκε"), Hindi => tr("लागू"), Georgian => tr("გამოყენებულია"), Dutch => tr("Toegepast"), Arabic => tr("تم التطبيق"), Spanish => tr("Aplicado"), Korean => None },
        "미적용" => match lang { English => tr("Not applied"), Russian => tr("Не применено"), Japanese => tr("未適用"), TraditionalChinese => tr("未套用"), Vietnamese => tr("Chưa áp dụng"), Greek => tr("Δεν εφαρμόστηκε"), Hindi => tr("लागू नहीं"), Georgian => tr("არ არის გამოყენებული"), Dutch => tr("Niet toegepast"), Arabic => tr("غير مطبق"), Spanish => tr("No aplicado"), Korean => None },
        "누락 있음" => match lang { English => tr("Missing items"), Russian => tr("Есть отсутствующие элементы"), Japanese => tr("不足あり"), TraditionalChinese => tr("有缺漏"), Vietnamese => tr("Có mục bị thiếu"), Greek => tr("Υπάρχουν ελλείψεις"), Hindi => tr("कुछ गायब है"), Georgian => tr("არის გამოტოვებული ელემენტები"), Dutch => tr("Ontbrekende items"), Arabic => tr("توجد عناصر مفقودة"), Spanish => tr("Faltan elementos"), Korean => None },
        "원본 flash.xml:" => match lang { English => tr("Original flash.xml:"), Russian => tr("Исходный flash.xml:"), Japanese => tr("元のflash.xml:"), TraditionalChinese => tr("原始 flash.xml:"), Vietnamese => tr("flash.xml gốc:"), Greek => tr("Αρχικό flash.xml:"), Hindi => tr("मूल flash.xml:"), Georgian => tr("ორიგინალი flash.xml:"), Dutch => tr("Originele flash.xml:"), Arabic => tr("flash.xml الأصلي:"), Spanish => tr("flash.xml original:"), Korean => None },
        "원본 scatter:" => match lang { English => tr("Original scatter:"), Russian => tr("Исходный scatter:"), Japanese => tr("元のscatter:"), TraditionalChinese => tr("原始 scatter:"), Vietnamese => tr("scatter gốc:"), Greek => tr("Αρχικό scatter:"), Hindi => tr("मूल scatter:"), Georgian => tr("ორიგინალი scatter:"), Dutch => tr("Originele scatter:"), Arabic => tr("scatter الأصلي:"), Spanish => tr("scatter original:"), Korean => None },
        "작업 flash.xml:" => match lang { English => tr("Work flash.xml:"), Russian => tr("Рабочий flash.xml:"), Japanese => tr("作業用flash.xml:"), TraditionalChinese => tr("工作 flash.xml:"), Vietnamese => tr("flash.xml làm việc:"), Greek => tr("flash.xml εργασίας:"), Hindi => tr("कार्य flash.xml:"), Georgian => tr("სამუშაო flash.xml:"), Dutch => tr("Werk flash.xml:"), Arabic => tr("flash.xml للعمل:"), Spanish => tr("flash.xml de trabajo:"), Korean => None },
        "작업 scatter.xml:" => match lang { English => tr("Work scatter.xml:"), Russian => tr("Рабочий scatter.xml:"), Japanese => tr("作業用scatter.xml:"), TraditionalChinese => tr("工作 scatter.xml:"), Vietnamese => tr("scatter.xml làm việc:"), Greek => tr("scatter.xml εργασίας:"), Hindi => tr("कार्य scatter.xml:"), Georgian => tr("სამუშაო scatter.xml:"), Dutch => tr("Werk scatter.xml:"), Arabic => tr("scatter.xml للعمل:"), Spanish => tr("scatter.xml de trabajo:"), Korean => None },
        "DA 파일:" => match lang { English => tr("DA file:"), Russian => tr("Файл DA:"), Japanese => tr("DAファイル:"), TraditionalChinese => tr("DA 檔案:"), Vietnamese => tr("Tệp DA:"), Greek => tr("Αρχείο DA:"), Hindi => tr("DA फ़ाइल:"), Georgian => tr("DA ფაილი:"), Dutch => tr("DA-bestand:"), Arabic => tr("ملف DA:"), Spanish => tr("Archivo DA:"), Korean => None },
        "중국" => match lang { English => tr("China"), Russian => tr("Китай"), Japanese => tr("中国"), TraditionalChinese => tr("中國"), Vietnamese => tr("Trung Quốc"), Greek => tr("Κίνα"), Hindi => tr("चीन"), Georgian => tr("ჩინეთი"), Dutch => tr("China"), Arabic => tr("الصين"), Spanish => tr("China"), Korean => None },
        "내수" => match lang { English => tr("domestic"), Russian => tr("внутренний"), Japanese => tr("中国国内版"), TraditionalChinese => tr("內銷"), Vietnamese => tr("nội địa"), Greek => tr("εγχώριο"), Hindi => tr("घरेलू"), Georgian => tr("შიდა ბაზრის"), Dutch => tr("binnenlands"), Arabic => tr("محلي"), Spanish => tr("doméstico"), Korean => None },
        "글로벌" => match lang { English => tr("global"), Russian => tr("глобальный"), Japanese => tr("グローバル"), TraditionalChinese => tr("全球版"), Vietnamese => tr("quốc tế"), Greek => tr("παγκόσμιο"), Hindi => tr("ग्लोबल"), Georgian => tr("გლობალური"), Dutch => tr("globaal"), Arabic => tr("عالمي"), Spanish => tr("global"), Korean => None },
        _ => None,
    }
}

fn lpm_translate_stage5_phrasewise(lang: LanguageOption, content: String) -> String {
    if lang.is_korean() || !content.chars().any(|c| ('가'..='힣').contains(&c)) {
        return content;
    }

    let mut out = content;
    for (from, to) in lpm_stage5_phrase_pairs(lang) {
        out = out.replace(from, to);
    }
    out
}

fn lpm_stage5_phrase_pairs(lang: LanguageOption) -> &'static [(&'static str, &'static str)] {
    use LanguageOption::*;
    match lang {
        English => &[
            ("[설정] 초기 언어 설정", "[Settings] Initial language"),
            ("[설정] 언어 설정 파일 경로", "[Settings] Language setting file path"),
            ("[설정] 언어 설정 파일에 저장했습니다", "[Settings] Saved to the language setting file"),
            ("[프로그램] 펌웨어 다운로드 링크 열기 실패", "[Program] Failed to open the firmware download link"),
            ("[설정] 개발자 유튜브 링크 열기 실패", "[Settings] Failed to open the developer YouTube link"),
            ("[설정] 후원하기 링크 열기 실패", "[Settings] Failed to open the sponsorship link"),
            ("[설정] 피드백 링크 열기 실패", "[Settings] Failed to open the feedback link"),
            ("[추가 옵션] 국가 코드를 선택했습니다", "[Additional Options] Selected country code"),
            ("[ROM 옵션] 국가 코드를 선택했습니다", "[ROM Options] Selected country code"),
            ("[추가 옵션] 국가 코드 재설정 국가 코드를 선택합니다", "[Additional Options] Select a country code for country code reset"),
            ("[국가 코드 재설정] 국가 코드 재설정 작업을 시작합니다", "[Country Code Reset] Starting country code reset"),
            ("[국가 코드 재설정] 먼저 ROM 작업에서 image 폴더를 선택해주세요", "[Country Code Reset] Select an image folder from ROM Tasks first"),
            ("[국가 코드 재설정] proinfo 파티션만 플래싱하려면", "[Country Code Reset] To flash only the proinfo partition,"),
            ("[Update] 최신 LPMBox 릴리즈를 확인합니다", "[Update] Checking the latest LPMBox release"),
            ("[Update] 새 LPMBox 버전을 찾았습니다", "[Update] Found a new LPMBox version"),
            ("[Update] 대시보드에 업데이트 안내 창을 표시합니다", "[Update] Showing the update notice on the dashboard"),
            ("[Update] 이미 최신 버전을 사용 중입니다", "[Update] You are already using the latest version"),
            ("[Update] 수동 확인을 위해 GitHub Releases 페이지를 엽니다", "[Update] Opening the GitHub Releases page for manual check"),
            ("[Update] 이번 업데이트 안내를 다음에 다시 확인합니다", "[Update] This update notice will be checked again later"),
            ("펌웨어 정보 및 설치 환경 검사를 시작합니다", "Starting firmware information and installation environment check"),
            ("image 폴더 선택이 취소되었습니다", "Image folder selection was canceled"),
            ("image 폴더 선택됨", "Image folder selected"),
            ("펌웨어 검사 작업 스레드 오류", "Firmware check worker thread error"),
            ("대시보드 갱신 작업 스레드 오류", "Dashboard refresh worker thread error"),
            ("작업이 비정상적으로 종료되었습니다", "The task ended abnormally"),
            ("작업 로그 자동 저장 실패", "Failed to automatically save the task log"),
            ("텍스트 파일 저장 실패", "Failed to save the text file"),
            ("텍스트 파일을", "Saving the text file to"),
            ("작업 로그를", "Saving the task log to"),
            ("저장합니다", ""),
            ("기기 연결이 끊겼습니다", "The device was disconnected"),
            ("기기가 다시 감지되었습니다", "The device was detected again"),
            ("기기가 연결되어 있지 않아 실행할 수 없습니다", "Cannot run because the device is not connected"),
            ("기기가 연결되어 있지 않아 설치를 실행할 수 없습니다", "Cannot install because the device is not connected"),
            ("기기가 PRC(중국 내수롬)이므로 불가능 합니다", "Unavailable because the device is PRC (China ROM)"),
            ("PRC(중국 내수롬) 업데이트는 지원하지 않습니다", "PRC (China ROM) update is not supported"),
            ("ROW(글로벌롬) 업데이트로 진행해 주세요", "Proceed with ROW (Global ROM) update"),
            ("기기에 배터리가 부족합니다", "The device battery is too low"),
            ("연결한 기기", "Connected device"),
            ("배터리 잔량", "Battery level"),
            ("가동 시간", "Uptime"),
            ("펌웨어 유형", "Firmware type"),
            ("유효성 검사", "Validation"),
            ("확인 필요", "Needs check"),
            ("확인 불가", "Cannot verify"),
            ("펌웨어 검사 결과", "Firmware check result"),
            ("올바른 image 폴더를", "Select the correct image folder"),
            ("올바른 image 폴더 재선택", "Reselect the correct image folder"),
            ("다른 버전 파일로 재시도", "Retry with another version file"),
            ("올바른 기기 연결", "Connect the correct device"),
            ("기기에 맞는 image 폴더를 선택", "Select an image folder matching the device"),
            ("25% 이상 충전 후 다시 시도", "Charge to at least 25% and try again"),
            ("해주세요", ""),
            ("다음 단계로 진행해주세요", "Proceed to the next step"),
            ("PRC ↔ ROW 설치 루틴", "PRC ↔ ROW install routine"),
            ("중국 내수롬과 글로벌롬을 자유롭게 변경 가능", "Switch freely between China ROM and Global ROM"),
            ("데이터 초기화가 필수이기 때문에", "Because data wipe is required,"),
            ("시작하기 전 데이터 백업 후 진행해주세요", "Back up your data before starting"),
            ("ROW(글로벌롬) 업데이트 루틴", "ROW (Global ROM) update routine"),
            ("글로벌롬 펌웨어 버전을 업데이트합니다", "Update the Global ROM firmware version"),
            ("기기에 설치된 버전보다 낮을 경우/초기화 O", "If lower than the installed version / wipe required"),
            ("기기에 설치된 버전보다 높은 경우/초기화 X", "If higher than the installed version / no wipe"),
            ("기기 복구 루틴", "Device recovery routine"),
            ("펌웨어 설치 실패", "firmware installation failure"),
            ("기기를 복구합니다", "recover the device"),
            ("활성화 할 경우 기기를 초기화 합니다", "When enabled, the device will be wiped"),
            ("기기에 국가 코드를 변경합니다", "Change the country code on the device"),
            ("감지된 국가 코드", "Detected country code"),
            ("선택한 국가 코드", "Selected country code"),
            ("image 폴더 롬", "Image folder ROM"),
            ("기기에 설치된 롬", "Installed device ROM"),
            ("현재", "Current"),
            ("최신", "Latest"),
            ("새로운 버전을 감지했습니다", "Detected a new version"),
            ("취소", "Cancel"),
            ("드라이버 설치를 해주세요", "Please install the driver"),
            ("기기 재시작", "device restart"),
            ("기기를 재시작합니다", "Restarting the device"),
            ("안정성을 위해 5초 대기", "Waiting 5 seconds for stability"),
            ("국가 코드 변경을 위해 proinfo 파티션을 백업합니다", "Backing up proinfo to change the country code"),
            ("사용자가 선택한 국가 코드", "User-selected country code"),
            ("proinfo에 국가 코드를 변경합니다", "Changing the country code in proinfo"),
            ("proinfo 국가 코드 변경 완료", "proinfo country code change completed"),
            ("proinfo 국가 코드 확인 완료", "proinfo country code check completed"),
            ("올바르지 않은 국가 코드입니다", "Invalid country code"),
            ("국가 코드 토큰을 찾지 못했습니다", "Could not find a country code token"),
            ("기기에", "On the device,"),
            ("설치합니다", "install"),
            ("버전을 업데이트 합니다", "update the version"),
            ("데이터 유지 여부", "Keep data"),
            ("필요 여부", "Required"),
            ("경고", "warning"),
            ("오류", "error"),
            ("누락", "missing"),
            ("통과", "passed"),
            ("모두 통과", "all passed"),
        ],
        Japanese => &[
            ("[설정] 초기 언어 설정", "[設定] 初期言語"), ("[설정] 언어 설정 파일 경로", "[設定] 言語設定ファイルのパス"), ("[설정] 언어 설정 파일에 저장했습니다", "[設定] 言語設定ファイルに保存しました"),
            ("이미 작업이 진행 중입니다", "すでに作業が進行中です"), ("펌웨어 정보 및 설치 환경 검사를 시작합니다", "ファームウェア情報とインストール環境の検査を開始します"), ("image 폴더 선택이 취소되었습니다", "imageフォルダーの選択がキャンセルされました"), ("image 폴더 선택됨", "imageフォルダーが選択されました"),
            ("작업이 비정상적으로 종료되었습니다", "作業が異常終了しました"), ("기기 연결이 끊겼습니다", "デバイスの接続が切断されました"), ("기기가 다시 감지되었습니다", "デバイスが再検出されました"), ("기기가 연결되어 있지 않아 실행할 수 없습니다", "デバイスが接続されていないため実行できません"),
            ("펌웨어 검사 실패", "ファームウェア検査失敗"), ("확인 필요", "確認が必要"), ("확인 불가", "確認不可"), ("연결한 기기", "接続したデバイス"), ("배터리 잔량", "バッテリー残量"), ("가동 시간", "稼働時間"), ("펌웨어 유형", "ファームウェア種類"), ("유효성 검사", "妥当性検査"),
            ("PRC ↔ ROW 설치 루틴", "PRC ↔ ROWインストールルーチン"), ("ROW(글로벌롬) 업데이트 루틴", "ROW（グローバルROM）更新ルーチン"), ("기기 복구 루틴", "デバイス復旧ルーチン"), ("데이터 초기화가 필수", "データ初期化が必須"), ("데이터 백업", "データのバックアップ"),
            ("감지된 국가 코드", "検出された国コード"), ("선택한 국가 코드", "選択した国コード"), ("image 폴더 롬", "imageフォルダーROM"), ("기기에 설치된 롬", "デバイスにインストール済みのROM"), ("취소", "キャンセル"),
            ("proinfo 국가 코드 변경", "proinfo国コード変更"), ("proinfo 국가 코드 확인", "proinfo国コード確認"), ("올바르지 않은 국가 코드입니다", "正しくない国コードです"), ("통과", "通過"), ("누락", "不足"), ("오류", "エラー"), ("경고", "警告")
        ],
        TraditionalChinese => &[
            ("[설정] 초기 언어 설정", "[設定] 初始語言"), ("[설정] 언어 설정 파일 경로", "[設定] 語言設定檔路徑"), ("[설정] 언어 설정 파일에 저장했습니다", "[設定] 已儲存到語言設定檔"),
            ("이미 작업이 진행 중입니다", "已有工作正在進行"), ("펌웨어 정보 및 설치 환경 검사를 시작합니다", "開始檢查韌體資訊與安裝環境"), ("image 폴더 선택이 취소되었습니다", "已取消選擇 image 資料夾"), ("image 폴더 선택됨", "已選擇 image 資料夾"),
            ("작업이 비정상적으로 종료되었습니다", "工作異常結束"), ("기기 연결이 끊겼습니다", "裝置連線已中斷"), ("기기가 다시 감지되었습니다", "已重新偵測到裝置"), ("기기가 연결되어 있지 않아 실행할 수 없습니다", "裝置未連接，無法執行"),
            ("펌웨어 검사 실패", "韌體檢查失敗"), ("확인 필요", "需要確認"), ("확인 불가", "無法確認"), ("연결한 기기", "已連接裝置"), ("배터리 잔량", "電池電量"), ("가동 시간", "運作時間"), ("펌웨어 유형", "韌體類型"), ("유효성 검사", "有效性檢查"),
            ("PRC ↔ ROW 설치 루틴", "PRC ↔ ROW 安裝流程"), ("ROW(글로벌롬) 업데이트 루틴", "ROW（全球版 ROM）更新流程"), ("기기 복구 루틴", "裝置修復流程"), ("데이터 초기화가 필수", "必須清除資料"), ("데이터 백업", "備份資料"),
            ("감지된 국가 코드", "偵測到的國家代碼"), ("선택한 국가 코드", "選擇的國家代碼"), ("image 폴더 롬", "image 資料夾 ROM"), ("기기에 설치된 롬", "裝置已安裝 ROM"), ("취소", "取消"),
            ("proinfo 국가 코드 변경", "變更 proinfo 國家代碼"), ("proinfo 국가 코드 확인", "確認 proinfo 國家代碼"), ("올바르지 않은 국가 코드입니다", "國家代碼不正確"), ("통과", "通過"), ("누락", "缺少"), ("오류", "錯誤"), ("경고", "警告")
        ],
        Spanish => &[
            ("[설정] 초기 언어 설정", "[Configuración] Idioma inicial"), ("[설정] 언어 설정 파일 경로", "[Configuración] Ruta del archivo de idioma"), ("[설정] 언어 설정 파일에 저장했습니다", "[Configuración] Guardado en el archivo de idioma"),
            ("이미 작업이 진행 중입니다", "Ya hay una tarea en curso"), ("펌웨어 정보 및 설치 환경 검사를 시작합니다", "Iniciando la comprobación de firmware y entorno"), ("image 폴더 선택이 취소되었습니다", "Selección de carpeta image cancelada"), ("image 폴더 선택됨", "Carpeta image seleccionada"),
            ("작업이 비정상적으로 종료되었습니다", "La tarea terminó de forma anormal"), ("기기 연결이 끊겼습니다", "El dispositivo se desconectó"), ("기기가 다시 감지되었습니다", "El dispositivo se detectó de nuevo"), ("기기가 연결되어 있지 않아 실행할 수 없습니다", "No se puede ejecutar porque el dispositivo no está conectado"),
            ("펌웨어 검사 실패", "Error de comprobación de firmware"), ("확인 필요", "Requiere revisión"), ("확인 불가", "No se puede verificar"), ("연결한 기기", "Dispositivo conectado"), ("배터리 잔량", "Batería restante"), ("가동 시간", "Tiempo activo"), ("펌웨어 유형", "Tipo de firmware"), ("유효성 검사", "Validación"),
            ("PRC ↔ ROW 설치 루틴", "Rutina de instalación PRC ↔ ROW"), ("ROW(글로벌롬) 업데이트 루틴", "Rutina de actualización ROW (ROM global)"), ("기기 복구 루틴", "Rutina de recuperación del dispositivo"), ("데이터 초기화가 필수", "El borrado de datos es obligatorio"), ("데이터 백업", "copia de seguridad de datos"),
            ("감지된 국가 코드", "Código de país detectado"), ("선택한 국가 코드", "Código de país seleccionado"), ("image 폴더 롬", "ROM de la carpeta image"), ("기기에 설치된 롬", "ROM instalada en el dispositivo"), ("취소", "Cancelar"),
            ("proinfo 국가 코드 변경", "Cambiar código de país en proinfo"), ("proinfo 국가 코드 확인", "Verificar código de país en proinfo"), ("올바르지 않은 국가 코드입니다", "Código de país no válido"), ("통과", "aprobado"), ("누락", "faltante"), ("오류", "error"), ("경고", "advertencia")
        ],
        Russian => &[("이미 작업이 진행 중입니다", "Задача уже выполняется"), ("펌웨어 검사 실패", "Ошибка проверки прошивки"), ("확인 필요", "Требуется проверка"), ("확인 불가", "Невозможно проверить"), ("취소", "Отмена"), ("통과", "пройдено"), ("누락", "отсутствует"), ("오류", "ошибка"), ("경고", "предупреждение"), ("기기 연결이 끊겼습니다", "Устройство отключено"), ("작업이 비정상적으로 종료되었습니다", "Задача завершилась некорректно")],
        Vietnamese => &[("이미 작업이 진행 중입니다", "Một tác vụ đang chạy"), ("펌웨어 검사 실패", "Kiểm tra firmware thất bại"), ("확인 필요", "Cần kiểm tra"), ("확인 불가", "Không thể xác minh"), ("취소", "Hủy"), ("통과", "đạt"), ("누락", "thiếu"), ("오류", "lỗi"), ("경고", "cảnh báo"), ("기기 연결이 끊겼습니다", "Thiết bị đã ngắt kết nối"), ("작업이 비정상적으로 종료되었습니다", "Tác vụ kết thúc bất thường")],
        Greek => &[("이미 작업이 진행 중입니다", "Μια εργασία εκτελείται ήδη"), ("펌웨어 검사 실패", "Αποτυχία ελέγχου firmware"), ("확인 필요", "Απαιτείται έλεγχος"), ("확인 불가", "Δεν είναι δυνατή η επαλήθευση"), ("취소", "Άκυρο"), ("통과", "πέρασε"), ("누락", "λείπει"), ("오류", "σφάλμα"), ("경고", "προειδοποίηση"), ("기기 연결이 끊겼습니다", "Η συσκευή αποσυνδέθηκε"), ("작업이 비정상적으로 종료되었습니다", "Η εργασία τερματίστηκε μη κανονικά")],
        Hindi => &[("이미 작업이 진행 중입니다", "एक कार्य पहले से चल रहा है"), ("펌웨어 검사 실패", "Firmware जांच विफल"), ("확인 필요", "जांच आवश्यक"), ("확인 불가", "सत्यापित नहीं किया जा सकता"), ("취소", "रद्द करें"), ("통과", "पास"), ("누락", "गुम"), ("오류", "त्रुटि"), ("경고", "चेतावनी"), ("기기 연결이 끊겼습니다", "डिवाइस डिस्कनेक्ट हो गया"), ("작업이 비정상적으로 종료되었습니다", "कार्य असामान्य रूप से समाप्त हुआ")],
        Georgian => &[("이미 작업이 진행 중입니다", "ამოცანა უკვე მიმდინარეობს"), ("펌웨어 검사 실패", "Firmware-ის შემოწმება ვერ მოხერხდა"), ("확인 필요", "საჭიროა შემოწმება"), ("확인 불가", "ვერ მოწმდება"), ("취소", "გაუქმება"), ("통과", "გაიარა"), ("누락", "აკლია"), ("오류", "შეცდომა"), ("경고", "გაფრთხილება"), ("기기 연결이 끊겼습니다", "მოწყობილობა გაითიშა"), ("작업이 비정상적으로 종료되었습니다", "ამოცანა არასწორად დასრულდა")],
        Dutch => &[("이미 작업이 진행 중입니다", "Er is al een taak bezig"), ("펌웨어 검사 실패", "Firmwarecontrole mislukt"), ("확인 필요", "Controle vereist"), ("확인 불가", "Kan niet worden gecontroleerd"), ("취소", "Annuleren"), ("통과", "geslaagd"), ("누락", "ontbreekt"), ("오류", "fout"), ("경고", "waarschuwing"), ("기기 연결이 끊겼습니다", "Het apparaat is losgekoppeld"), ("작업이 비정상적으로 종료되었습니다", "De taak is abnormaal beëindigd")],
        Arabic => &[("이미 작업이 진행 중입니다", "هناك مهمة قيد التنفيذ بالفعل"), ("펌웨어 검사 실패", "فشل فحص firmware"), ("확인 필요", "يتطلب التحقق"), ("확인 불가", "لا يمكن التحقق"), ("취소", "إلغاء"), ("통과", "نجح"), ("누락", "مفقود"), ("오류", "خطأ"), ("경고", "تحذير"), ("기기 연결이 끊겼습니다", "تم فصل الجهاز"), ("작업이 비정상적으로 종료되었습니다", "انتهت المهمة بشكل غير طبيعي")],
        _ => &[],
    }
}

fn lpm_translate_exact_stage4(lang: LanguageOption, key: &str) -> Option<&'static str> {
    use LanguageOption::*;
    let en = |s: &'static str| -> Option<&'static str> { Some(s) };
    match key {
        "대시 보드" => match lang { English => en("Dashboard"), Russian => en("Панель управления"), Japanese => en("ダッシュボード"), TraditionalChinese => en("控制面板"), Vietnamese => en("Bảng điều khiển"), Greek => en("Πίνακας ελέγχου"), Hindi => en("डैशबोर्ड"), Georgian => en("დაფა"), Dutch => en("Dashboard"), Arabic => en("لوحة التحكم"), Spanish => en("Panel"), Korean => None },
        "현재 image 폴더" => match lang { English => en("Current image folder"), Russian => en("Текущая папка image"), Japanese => en("現在のimageフォルダー"), TraditionalChinese => en("目前的 image 資料夾"), Vietnamese => en("Thư mục image hiện tại"), Greek => en("Τρέχων φάκελος image"), Hindi => en("वर्तमान image फ़ोल्डर"), Georgian => en("მიმდინარე image საქაღალდე"), Dutch => en("Huidige image-map"), Arabic => en("مجلد image الحالي"), Spanish => en("Carpeta image actual"), Korean => None },
        "선택된 image 폴더 없음" => match lang { English => en("No image folder selected"), Russian => en("Папка image не выбрана"), Japanese => en("imageフォルダーが選択されていません"), TraditionalChinese => en("尚未選擇 image 資料夾"), Vietnamese => en("Chưa chọn thư mục image"), Greek => en("Δεν έχει επιλεγεί φάκελος image"), Hindi => en("कोई image फ़ोल्डर चयनित नहीं"), Georgian => en("image საქაღალდე არჩეული არ არის"), Dutch => en("Geen image-map geselecteerd"), Arabic => en("لم يتم اختيار مجلد image"), Spanish => en("No se seleccionó carpeta image"), Korean => None },
        "LPMBox image 폴더 선택" => match lang { English => en("Select LPMBox image folder"), Russian => en("Выберите папку image LPMBox"), Japanese => en("LPMBox imageフォルダーを選択"), TraditionalChinese => en("選擇 LPMBox image 資料夾"), Vietnamese => en("Chọn thư mục image LPMBox"), Greek => en("Επιλέξτε φάκελο image LPMBox"), Hindi => en("LPMBox image फ़ोल्डर चुनें"), Georgian => en("აირჩიეთ LPMBox image საქაღალდე"), Dutch => en("Selecteer LPMBox image-map"), Arabic => en("اختر مجلد image الخاص بـ LPMBox"), Spanish => en("Seleccionar carpeta image de LPMBox"), Korean => None },
        "먼저 image 폴더를 선택해주세요." => match lang { English => en("Please select the image folder first."), Russian => en("Сначала выберите папку image."), Japanese => en("先にimageフォルダーを選択してください。"), TraditionalChinese => en("請先選擇 image 資料夾。"), Vietnamese => en("Vui lòng chọn thư mục image trước."), Greek => en("Επιλέξτε πρώτα τον φάκελο image."), Hindi => en("पहले image फ़ोल्डर चुनें।"), Georgian => en("ჯერ აირჩიეთ image საქაღალდე."), Dutch => en("Selecteer eerst de image-map."), Arabic => en("يرجى اختيار مجلد image أولاً."), Spanish => en("Seleccione primero la carpeta image."), Korean => None },
        "이미 작업이 진행 중입니다." => match lang { English => en("A task is already in progress."), Russian => en("Задача уже выполняется."), Japanese => en("すでに作業が進行中です。"), TraditionalChinese => en("已有工作正在進行。"), Vietnamese => en("Một tác vụ đang chạy."), Greek => en("Μια εργασία βρίσκεται ήδη σε εξέλιξη."), Hindi => en("एक कार्य पहले से चल रहा है।"), Georgian => en("ამოცანა უკვე მიმდინარეობს."), Dutch => en("Er is al een taak bezig."), Arabic => en("هناك مهمة قيد التنفيذ بالفعل."), Spanish => en("Ya hay una tarea en curso."), Korean => None },
        "작업 중" => match lang { English => en("Working"), Russian => en("Выполняется"), Japanese => en("作業中"), TraditionalChinese => en("作業中"), Vietnamese => en("Đang xử lý"), Greek => en("Σε εξέλιξη"), Hindi => en("कार्य जारी"), Georgian => en("მიმდინარეობს"), Dutch => en("Bezig"), Arabic => en("جارٍ العمل"), Spanish => en("Trabajando"), Korean => None },
        "대기 중" => match lang { English => en("Idle"), Russian => en("Ожидание"), Japanese => en("待機中"), TraditionalChinese => en("待機中"), Vietnamese => en("Đang chờ"), Greek => en("Αναμονή"), Hindi => en("प्रतीक्षा में"), Georgian => en("მოლოდინი"), Dutch => en("Wachten"), Arabic => en("في الانتظار"), Spanish => en("En espera"), Korean => None },
        "로그가 없습니다." => match lang { English => en("No logs."), Russian => en("Журнал пуст."), Japanese => en("ログはありません。"), TraditionalChinese => en("沒有日誌。"), Vietnamese => en("Không có nhật ký."), Greek => en("Δεν υπάρχουν καταγραφές."), Hindi => en("कोई लॉग नहीं है।"), Georgian => en("ჟურნალი ცარიელია."), Dutch => en("Geen logs."), Arabic => en("لا توجد سجلات."), Spanish => en("No hay registros."), Korean => None },
        "검색 결과가 없습니다." => match lang { English => en("No search results."), Russian => en("Нет результатов поиска."), Japanese => en("検索結果がありません。"), TraditionalChinese => en("沒有搜尋結果。"), Vietnamese => en("Không có kết quả tìm kiếm."), Greek => en("Δεν υπάρχουν αποτελέσματα."), Hindi => en("कोई खोज परिणाम नहीं है।"), Georgian => en("ძიების შედეგები არ არის."), Dutch => en("Geen zoekresultaten."), Arabic => en("لا توجد نتائج بحث."), Spanish => en("No hay resultados."), Korean => None },
        "국가 코드 또는 국가명 검색" => match lang { English => en("Search country code or country name"), Russian => en("Поиск кода или названия страны"), Japanese => en("国コードまたは国名を検索"), TraditionalChinese => en("搜尋國家代碼或國家名稱"), Vietnamese => en("Tìm mã quốc gia hoặc tên quốc gia"), Greek => en("Αναζήτηση κωδικού ή ονόματος χώρας"), Hindi => en("देश कोड या देश का नाम खोजें"), Georgian => en("მოძებნეთ ქვეყნის კოდი ან სახელი"), Dutch => en("Zoek landcode of landnaam"), Arabic => en("ابحث عن رمز الدولة أو اسمها"), Spanish => en("Buscar código o nombre de país"), Korean => None },
        "올바른 image 폴더로 다시 시도해 주세요." => match lang { English => en("Try again with a valid image folder."), Russian => en("Повторите с правильной папкой image."), Japanese => en("正しいimageフォルダーで再試行してください。"), TraditionalChinese => en("請使用正確的 image 資料夾重試。"), Vietnamese => en("Thử lại với thư mục image hợp lệ."), Greek => en("Δοκιμάστε ξανά με σωστό φάκελο image."), Hindi => en("सही image फ़ोल्डर के साथ फिर कोशिश करें।"), Georgian => en("სწორი image საქაღალდით სცადეთ თავიდან."), Dutch => en("Probeer opnieuw met een geldige image-map."), Arabic => en("حاول مرة أخرى باستخدام مجلد image صحيح."), Spanish => en("Inténtelo de nuevo con una carpeta image válida."), Korean => None },
        "올바른 기기를 연결해주세요." => match lang { English => en("Connect the correct device."), Russian => en("Подключите правильное устройство."), Japanese => en("正しいデバイスを接続してください。"), TraditionalChinese => en("請連接正確的裝置。"), Vietnamese => en("Kết nối đúng thiết bị."), Greek => en("Συνδέστε τη σωστή συσκευή."), Hindi => en("सही डिवाइस कनेक्ट करें।"), Georgian => en("შეაერთეთ სწორი მოწყობილობა."), Dutch => en("Sluit het juiste apparaat aan."), Arabic => en("وصّل الجهاز الصحيح."), Spanish => en("Conecte el dispositivo correcto."), Korean => None },
        "25% 이상 충전 후 다시 시도해주세요." => match lang { English => en("Charge to at least 25% and try again."), Russian => en("Зарядите минимум до 25% и повторите."), Japanese => en("25%以上充電してから再試行してください。"), TraditionalChinese => en("請充電至 25% 以上後重試。"), Vietnamese => en("Sạc ít nhất 25% rồi thử lại."), Greek => en("Φορτίστε τουλάχιστον στο 25% και δοκιμάστε ξανά."), Hindi => en("कम से कम 25% चार्ज करके फिर कोशिश करें।"), Georgian => en("დატენეთ მინიმუმ 25%-მდე და სცადეთ თავიდან."), Dutch => en("Laad tot minimaal 25% en probeer opnieuw."), Arabic => en("اشحن إلى 25% على الأقل ثم حاول مرة أخرى."), Spanish => en("Cargue al menos al 25% e inténtelo de nuevo."), Korean => None },
        "다른 버전 파일로 다시 시도해 주세요." => match lang { English => en("Try again with a different version file."), Russian => en("Повторите с файлом другой версии."), Japanese => en("別のバージョンのファイルで再試行してください。"), TraditionalChinese => en("請使用其他版本檔案重試。"), Vietnamese => en("Thử lại với tệp phiên bản khác."), Greek => en("Δοκιμάστε ξανά με αρχείο άλλης έκδοσης."), Hindi => en("दूसरे version file के साथ फिर कोशिश करें।"), Georgian => en("სხვა ვერსიის ფაილით სცადეთ თავიდან."), Dutch => en("Probeer opnieuw met een ander versiebestand."), Arabic => en("حاول مرة أخرى بملف إصدار آخر."), Spanish => en("Inténtelo con un archivo de otra versión."), Korean => None },
        _ => None,
    }
}

fn lpm_translate_stage4_phrasewise(lang: LanguageOption, content: String) -> String {
    if lang.is_korean() || !content.chars().any(|c| ('가'..='힣').contains(&c)) {
        return content;
    }

    let mut out = content;
    for (from, to) in lpm_stage4_phrase_pairs(lang) {
        out = out.replace(from, to);
    }
    out
}

fn lpm_stage4_phrase_pairs(lang: LanguageOption) -> &'static [(&'static str, &'static str)] {
    use LanguageOption::*;
    match lang {
        English => &[
            ("ADB 기기 감지 및 기기 정보 확인", "ADB device detection and device information check"),
            ("image 폴더 검사 및 플래싱 준비", "image folder check and flashing preparation"),
            ("image 폴더 기본 검사", "basic image folder check"),
            ("Flash Plan 준비 및 작업용 scatter/xml 생성", "Flash Plan preparation and work scatter/xml creation"),
            ("current slot A 설정", "set current slot A"),
            ("MediaTek PreLoader 포트 감지", "MediaTek PreLoader port detection"),
            ("SPFlashToolV6 ROM 설치", "SPFlashToolV6 ROM installation"),
            ("SPFlashToolV6 proinfo 파티션만 플래싱", "SPFlashToolV6 proinfo-only flashing"),
            ("국가 코드 재설정용 재부팅", "reboot for country code reset"),
            ("proinfo 백업 및 선택한 국가 코드로 수정", "proinfo backup and patch with the selected country code"),
            ("proinfo 전용 Flash Plan 준비", "proinfo-only Flash Plan preparation"),
            ("수정한 proinfo 플래싱용 재부팅", "reboot for flashing the patched proinfo"),
            ("설치 금지 펌웨어입니다", "This firmware is blocked"),
            ("기기 복구 준비 실패", "device recovery preparation failed"),
            ("국가 코드 재설정 실패", "country code reset failed"),
            ("OTA(업데이트) 비활성화 실패", "OTA update disable failed"),
            ("OTA(업데이트) 활성화 실패", "OTA update enable failed"),
            ("USB 디버깅 활성화", "enable USB debugging"),
            ("개발자 옵션", "Developer options"),
            ("잠금 해제", "unlock"),
            ("메세지 창 왼쪽 중간 체크 박스 체크", "check the checkbox in the middle-left of the message window"),
            ("오른쪽 하단", "bottom right"),
            ("허용", "Allow"),
            ("기기를 감지하고 있습니다", "detecting the device"),
            ("기기 감지 완료", "device detected"),
            ("기기 감지 실패", "device detection failed"),
            ("기기 감지 중", "detecting device"),
            ("재시작 요청 완료", "restart request completed"),
            ("재시작 요청 실패", "restart request failed"),
            ("안정화를 위해", "for stabilization"),
            ("대기합니다", "waiting"),
            ("대기 완료", "wait complete"),
            ("설정 적용 실패", "settings apply failed"),
            ("패키지 비활성화 확인 필요", "package disable needs confirmation"),
            ("패키지 활성화 확인 필요", "package enable needs confirmation"),
            ("패키지 복원 확인 필요", "package restore needs confirmation"),
            ("기기에 설치된 ROM 타입", "ROM type installed on the device"),
            ("상태를 확인해주세요", "please check the status"),
            ("USB ADB 권한 요청/감지 중", "requesting/detecting USB ADB authorization"),
            ("ADB unauthorized 상태입니다", "ADB is unauthorized"),
            ("외부 adb server가 USB ADB 인터페이스를 점유 중입니다", "an external adb server is occupying the USB ADB interface"),
            ("USB ADB 기기를 찾지 못했습니다", "USB ADB device not found"),
            ("ADB 기기 감지 시간 초과", "ADB device detection timed out"),
            ("ADB USB 직접 연결 실패", "direct ADB USB connection failed"),
            ("Fastboot USB 인터페이스 열기 실패", "failed to open Fastboot USB interface"),
            ("Fastboot 명령 실패", "Fastboot command failed"),
            ("Fastboot 응답이 너무 짧습니다", "Fastboot response is too short"),
            ("MTK 드라이버 파일 준비", "MTK driver file preparation"),
            ("MTK 드라이버 설치 파일 준비", "MTK driver installer preparation"),
            ("MTK 드라이버 설치 감지 완료", "MTK driver installation detected"),
            ("설치 창을 완료한 뒤 다시 시도해주세요", "finish the installer window and try again"),
            ("다운로드 HTTP 오류", "download HTTP error"),
            ("파일 크기가 0입니다", "file size is 0"),
            ("ZIP 열기 실패", "failed to open ZIP"),
            ("ZIP 항목 읽기 실패", "failed to read ZIP entry"),
            ("관리자 권한 확인 창에서 취소되었습니다", "was cancelled at the administrator permission prompt"),
            ("상위 root 경로를 찾지 못했습니다", "could not find the parent root path"),
            ("모델과 플랫폼 정보가 일치하지 않습니다", "model and platform information do not match"),
            ("flash.xml scatter 경로가 손상되었습니다", "flash.xml scatter path is corrupted"),
            ("항목을 찾지 못했습니다", "entry not found"),
            ("값이 비어 있습니다", "value is empty"),
            ("필수 partition 누락", "required partition missing"),
            ("데이터 유지", "keep data"),
            ("데이터 초기화", "wipe data"),
            ("proinfo는 비활성화합니다", "proinfo is disabled"),
            ("userdata를 다운로드 대상으로 유지합니다", "keep userdata as a download target"),
            ("userdata를 반드시 비활성화하여 데이터를 유지합니다", "disable userdata to preserve data"),
            ("proinfo만 다운로드 대상으로 활성화합니다", "enable only proinfo as a download target"),
            ("파티션을 모두 다운로드 대상으로 활성화합니다", "enable all partitions as download targets"),
            ("공식 SPFlashToolV6 다운로드 실패", "official SPFlashToolV6 download failed"),
            ("내장 SPFlashToolV6 추출", "built-in SPFlashToolV6 extraction"),
            ("필수 파일이 없습니다", "required files are missing"),
            ("파일을 찾을 수 없습니다", "file not found"),
            ("지원하지 않는 모델입니다", "unsupported model"),
            ("펌웨어 폴더가 올바르지 않습니다", "firmware folder is invalid"),
            ("차단된 펌웨어 버전입니다", "blocked firmware version"),
            ("scatter 복호화에 실패했습니다", "scatter decryption failed"),
            ("XML 파싱에 실패했습니다", "XML parsing failed"),
            ("I/O 오류", "I/O error"),
            ("알 수 없는 응답", "unknown response"),
            ("사용 하지 않음", "not used"),
            ("차단 필요", "blocking required"),
            ("차단 완료", "blocked"),
            ("사용 중", "in use"),
            ("감지됨", "detected"),
            ("Windows 전용 감지", "Windows-only detection"),
            ("제한 시간 초과", "timeout"),
            ("읽기 실패", "read failed"),
            ("쓰기 실패", "write failed"),
            ("너무 작습니다", "too small"),
            ("잘못되었습니다", "invalid"),
            ("파싱 실패", "parse failed"),
            ("음수입니다", "is negative"),
            ("범위가 잘못되었습니다", "range is invalid"),
        ],
        Russian => &[("이미 작업이 진행 중입니다", "Задача уже выполняется"), ("알 수 없음", "Неизвестно"), ("감지 전", "До обнаружения"), ("기기 감지", "Обнаружение устройства"), ("설정 적용", "Применение настройки"), ("패키지 비활성화", "Отключение пакета"), ("패키지 활성화", "Включение пакета"), ("패키지 복원", "Восстановление пакета"), ("기기 복구", "Восстановление устройства"), ("국가 코드 재설정", "Сброс кода страны"), ("PreLoader 포트 감지", "Обнаружение порта PreLoader"), ("프로그램 업데이트", "Обновление программы")],
        Japanese => &[("이미 작업이 진행 중입니다", "すでに作業が進行中です"), ("알 수 없음", "不明"), ("감지 전", "検出前"), ("기기 감지", "デバイス検出"), ("설정 적용", "設定適用"), ("패키지 비활성화", "パッケージ無効化"), ("패키지 활성화", "パッケージ有効化"), ("패키지 복원", "パッケージ復元"), ("기기 복구", "デバイス復旧"), ("국가 코드 재설정", "国コードリセット"), ("PreLoader 포트 감지", "PreLoaderポート検出"), ("프로그램 업데이트", "プログラム更新")],
        TraditionalChinese => &[("이미 작업이 진행 중입니다", "已有工作正在進行"), ("알 수 없음", "未知"), ("감지 전", "偵測前"), ("기기 감지", "裝置偵測"), ("설정 적용", "套用設定"), ("패키지 비활성화", "停用套件"), ("패키지 활성화", "啟用套件"), ("패키지 복원", "還原套件"), ("기기 복구", "裝置修復"), ("국가 코드 재설정", "重設國家代碼"), ("PreLoader 포트 감지", "偵測 PreLoader 連接埠"), ("프로그램 업데이트", "程式更新")],
        Vietnamese => &[("이미 작업이 진행 중입니다", "Một tác vụ đang chạy"), ("알 수 없음", "Không rõ"), ("감지 전", "Chưa phát hiện"), ("기기 감지", "Phát hiện thiết bị"), ("설정 적용", "Áp dụng cài đặt"), ("패키지 비활성화", "Tắt gói"), ("패키지 활성화", "Bật gói"), ("패키지 복원", "Khôi phục gói"), ("기기 복구", "Khôi phục thiết bị"), ("국가 코드 재설정", "Đặt lại mã quốc gia"), ("PreLoader 포트 감지", "Phát hiện cổng PreLoader"), ("프로그램 업데이트", "Cập nhật chương trình")],
        Greek => &[("이미 작업이 진행 중입니다", "Μια εργασία βρίσκεται ήδη σε εξέλιξη"), ("알 수 없음", "Άγνωστο"), ("감지 전", "Πριν τον εντοπισμό"), ("기기 감지", "Εντοπισμός συσκευής"), ("설정 적용", "Εφαρμογή ρύθμισης"), ("패키지 비활성화", "Απενεργοποίηση πακέτου"), ("패키지 활성화", "Ενεργοποίηση πακέτου"), ("패키지 복원", "Επαναφορά πακέτου"), ("기기 복구", "Ανάκτηση συσκευής"), ("국가 코드 재설정", "Επαναφορά κωδικού χώρας"), ("PreLoader 포트 감지", "Εντοπισμός θύρας PreLoader"), ("프로그램 업데이트", "Ενημέρωση προγράμματος")],
        Hindi => &[("이미 작업이 진행 중입니다", "एक कार्य पहले से चल रहा है"), ("알 수 없음", "अज्ञात"), ("감지 전", "पता लगाने से पहले"), ("기기 감지", "डिवाइस पहचान"), ("설정 적용", "सेटिंग लागू"), ("패키지 비활성화", "पैकेज निष्क्रिय"), ("패키지 활성화", "पैकेज सक्रिय"), ("패키지 복원", "पैकेज बहाल"), ("기기 복구", "डिवाइस रिकवरी"), ("국가 코드 재설정", "देश कोड रीसेट"), ("PreLoader 포트 감지", "PreLoader पोर्ट पहचान"), ("프로그램 업데이트", "प्रोग्राम अपडेट")],
        Georgian => &[("이미 작업이 진행 중입니다", "ამოცანა უკვე მიმდინარეობს"), ("알 수 없음", "უცნობია"), ("감지 전", "აღმოჩენამდე"), ("기기 감지", "მოწყობილობის აღმოჩენა"), ("설정 적용", "პარამეტრის გამოყენება"), ("패키지 비활성화", "პაკეტის გამორთვა"), ("패키지 활성화", "პაკეტის ჩართვა"), ("패키지 복원", "პაკეტის აღდგენა"), ("기기 복구", "მოწყობილობის აღდგენა"), ("국가 코드 재설정", "ქვეყნის კოდის გადაყენება"), ("PreLoader 포트 감지", "PreLoader პორტის აღმოჩენა"), ("프로그램 업데이트", "პროგრამის განახლება")],
        Dutch => &[("이미 작업이 진행 중입니다", "Er is al een taak bezig"), ("알 수 없음", "Onbekend"), ("감지 전", "Vóór detectie"), ("기기 감지", "Apparaatdetectie"), ("설정 적용", "Instelling toepassen"), ("패키지 비활성화", "Pakket uitschakelen"), ("패키지 활성화", "Pakket inschakelen"), ("패키지 복원", "Pakket herstellen"), ("기기 복구", "Apparaatherstel"), ("국가 코드 재설정", "Landcode resetten"), ("PreLoader 포트 감지", "PreLoader-poortdetectie"), ("프로그램 업데이트", "Programma-update")],
        Arabic => &[("이미 작업이 진행 중입니다", "هناك مهمة قيد التنفيذ"), ("알 수 없음", "غير معروف"), ("감지 전", "قبل الاكتشاف"), ("기기 감지", "اكتشاف الجهاز"), ("설정 적용", "تطبيق الإعداد"), ("패키지 비활성화", "تعطيل الحزمة"), ("패키지 활성화", "تفعيل الحزمة"), ("패키지 복원", "استعادة الحزمة"), ("기기 복구", "استرداد الجهاز"), ("국가 코드 재설정", "إعادة تعيين رمز الدولة"), ("PreLoader 포트 감지", "اكتشاف منفذ PreLoader"), ("프로그램 업데이트", "تحديث البرنامج")],
        Spanish => &[("이미 작업이 진행 중입니다", "Ya hay una tarea en curso"), ("알 수 없음", "Desconocido"), ("감지 전", "Antes de detectar"), ("기기 감지", "Detección del dispositivo"), ("설정 적용", "Aplicar ajuste"), ("패키지 비활성화", "Desactivar paquete"), ("패키지 활성화", "Activar paquete"), ("패키지 복원", "Restaurar paquete"), ("기기 복구", "Recuperación del dispositivo"), ("국가 코드 재설정", "Restablecer código de país"), ("PreLoader 포트 감지", "Detección del puerto PreLoader"), ("프로그램 업데이트", "Actualización del programa")],
        Korean => &[],
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RomSlideTarget {
    Install,
    Update,
    Reinstall,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LanguageOption {
    English,
    Korean,
    Russian,
    Japanese,
    TraditionalChinese,
    Vietnamese,
    Greek,
    Hindi,
    Georgian,
    Dutch,
    Arabic,
    Spanish,
}

const LANGUAGE_OPTIONS: [LanguageOption; 12] = [
    LanguageOption::English,
    LanguageOption::Korean,
    LanguageOption::Russian,
    LanguageOption::Japanese,
    LanguageOption::TraditionalChinese,
    LanguageOption::Vietnamese,
    LanguageOption::Greek,
    LanguageOption::Hindi,
    LanguageOption::Georgian,
    LanguageOption::Dutch,
    LanguageOption::Arabic,
    LanguageOption::Spanish,
];

impl LanguageOption {
    fn is_korean(self) -> bool {
        matches!(self, LanguageOption::Korean)
    }

    fn index(self) -> u8 {
        self as u8
    }

    fn from_index(index: u8) -> Self {
        match index {
            0 => LanguageOption::English,
            1 => LanguageOption::Korean,
            2 => LanguageOption::Russian,
            3 => LanguageOption::Japanese,
            4 => LanguageOption::TraditionalChinese,
            5 => LanguageOption::Vietnamese,
            6 => LanguageOption::Greek,
            7 => LanguageOption::Hindi,
            8 => LanguageOption::Georgian,
            9 => LanguageOption::Dutch,
            10 => LanguageOption::Arabic,
            11 => LanguageOption::Spanish,
            _ => LanguageOption::Korean,
        }
    }

    fn code(self) -> &'static str {
        match self {
            LanguageOption::English => "en",
            LanguageOption::Korean => "ko",
            LanguageOption::Russian => "ru",
            LanguageOption::Japanese => "jp",
            LanguageOption::TraditionalChinese => "zh-TW",
            LanguageOption::Vietnamese => "vi",
            LanguageOption::Greek => "el",
            LanguageOption::Hindi => "hi",
            LanguageOption::Georgian => "ka",
            LanguageOption::Dutch => "nl",
            LanguageOption::Arabic => "ar",
            LanguageOption::Spanish => "es",
        }
    }

    fn from_code(code: &str) -> Option<Self> {
        match code.trim().to_ascii_lowercase().as_str() {
            "en" | "english" => Some(LanguageOption::English),
            "ko" | "kr" | "ko-kr" | "korean" => Some(LanguageOption::Korean),
            "ru" | "russian" => Some(LanguageOption::Russian),
            "jp" | "ja" | "ja-jp" | "jp-jp" | "japanese" => Some(LanguageOption::Japanese),
            "zh" | "zh-tw" | "zh-hk" | "zh-mo" | "zh-hant" | "tw" => Some(LanguageOption::TraditionalChinese),
            "zh-cn" | "zh-sg" | "zh-hans" | "cn" => Some(LanguageOption::English),
            "vi" | "vi-vn" | "vietnamese" => Some(LanguageOption::Vietnamese),
            "el" | "el-gr" | "greek" => Some(LanguageOption::Greek),
            "hi" | "hi-in" | "hindi" => Some(LanguageOption::Hindi),
            "ka" | "ka-ge" | "georgian" => Some(LanguageOption::Georgian),
            "nl" | "nl-nl" | "dutch" => Some(LanguageOption::Dutch),
            "ar" | "arabic" => Some(LanguageOption::Arabic),
            "es" | "spanish" => Some(LanguageOption::Spanish),
            _ => None,
        }
    }

    fn from_locale(locale: &str) -> Option<Self> {
        let normalized = locale.trim().to_ascii_lowercase();
        if normalized.starts_with("zh-tw")
            || normalized.starts_with("zh_hk")
            || normalized.starts_with("zh-hk")
            || normalized.starts_with("zh-mo")
            || normalized.contains("hant")
        {
            return Some(LanguageOption::TraditionalChinese);
        }

        if normalized.starts_with("zh-cn")
            || normalized.starts_with("zh-sg")
            || normalized.contains("hans")
        {
            return Some(LanguageOption::English);
        }

        let primary = normalized
            .split(['-', '_', '.', '@'])
            .next()
            .unwrap_or("");

        match primary {
            "en" => Some(LanguageOption::English),
            "ko" => Some(LanguageOption::Korean),
            "ru" => Some(LanguageOption::Russian),
            "ja" | "jp" => Some(LanguageOption::Japanese),
            "vi" => Some(LanguageOption::Vietnamese),
            "el" => Some(LanguageOption::Greek),
            "hi" => Some(LanguageOption::Hindi),
            "ka" => Some(LanguageOption::Georgian),
            "nl" => Some(LanguageOption::Dutch),
            "ar" => Some(LanguageOption::Arabic),
            "es" => Some(LanguageOption::Spanish),
            _ => None,
        }
    }
}

impl std::fmt::Display for LanguageOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            LanguageOption::English => "English (en)",
            LanguageOption::Korean => "한국어 (ko)",
            LanguageOption::Russian => "Русский (ru)",
            LanguageOption::Japanese => "日本語 (jp)",
            LanguageOption::TraditionalChinese => "繁體中文 (zh-TW)",
            LanguageOption::Vietnamese => "Tiếng Việt (vi)",
            LanguageOption::Greek => "Ελληνικά (el)",
            LanguageOption::Hindi => "हिन्दी (hi)",
            LanguageOption::Georgian => "ქართული (ka)",
            LanguageOption::Dutch => "Nederlands (nl)",
            LanguageOption::Arabic => "العربية (ar)",
            LanguageOption::Spanish => "Español (es)",
        };

        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeFlowKind {
    PrcRowInstall,
    RowUpdate,
    DeviceRecovery,
    CountryReset,
    OtaDisable,
    OtaEnable,
}

impl RuntimeFlowKind {
    fn label(self) -> &'static str {
        match self {
            RuntimeFlowKind::PrcRowInstall => "PRC ↔ ROW 설치",
            RuntimeFlowKind::RowUpdate => "ROW 업데이트",
            RuntimeFlowKind::DeviceRecovery => "기기 복구",
            RuntimeFlowKind::CountryReset => "국가 코드 재설정",
            RuntimeFlowKind::OtaDisable => "OTA 비활성화",
            RuntimeFlowKind::OtaEnable => "OTA 활성화",
        }
    }

    fn file_stem(self) -> &'static str {
        match self {
            RuntimeFlowKind::PrcRowInstall => "prc_row_install",
            RuntimeFlowKind::RowUpdate => "row_update",
            RuntimeFlowKind::DeviceRecovery => "device_recovery",
            RuntimeFlowKind::CountryReset => "country_reset",
            RuntimeFlowKind::OtaDisable => "ota_disable",
            RuntimeFlowKind::OtaEnable => "ota_enable",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CountryEntry {
    code: &'static str,
}

const ROM_COUNTRY_COUNT: usize = 88;

const ROM_COUNTRY_CODES: &[CountryEntry; ROM_COUNTRY_COUNT] = &[
    CountryEntry { code: "AE" },
    CountryEntry { code: "AM" },
    CountryEntry { code: "AR" },
    CountryEntry { code: "AT" },
    CountryEntry { code: "AU" },
    CountryEntry { code: "AZ" },
    CountryEntry { code: "BE" },
    CountryEntry { code: "BG" },
    CountryEntry { code: "BH" },
    CountryEntry { code: "BR" },
    CountryEntry { code: "CA" },
    CountryEntry { code: "CH" },
    CountryEntry { code: "CL" },
    CountryEntry { code: "CN" },
    CountryEntry { code: "CO" },
    CountryEntry { code: "CR" },
    CountryEntry { code: "CY" },
    CountryEntry { code: "CZ" },
    CountryEntry { code: "DE" },
    CountryEntry { code: "DK" },
    CountryEntry { code: "EC" },
    CountryEntry { code: "EE" },
    CountryEntry { code: "EG" },
    CountryEntry { code: "ES" },
    CountryEntry { code: "FI" },
    CountryEntry { code: "FR" },
    CountryEntry { code: "GB" },
    CountryEntry { code: "GE" },
    CountryEntry { code: "GH" },
    CountryEntry { code: "GR" },
    CountryEntry { code: "GT" },
    CountryEntry { code: "HK" },
    CountryEntry { code: "HR" },
    CountryEntry { code: "HU" },
    CountryEntry { code: "ID" },
    CountryEntry { code: "IL" },
    CountryEntry { code: "IN" },
    CountryEntry { code: "IS" },
    CountryEntry { code: "IT" },
    CountryEntry { code: "JO" },
    CountryEntry { code: "JP" },
    CountryEntry { code: "KE" },
    CountryEntry { code: "KG" },
    CountryEntry { code: "KR" },
    CountryEntry { code: "KW" },
    CountryEntry { code: "KZ" },
    CountryEntry { code: "LB" },
    CountryEntry { code: "LT" },
    CountryEntry { code: "LV" },
    CountryEntry { code: "MA" },
    CountryEntry { code: "MD" },
    CountryEntry { code: "MX" },
    CountryEntry { code: "MY" },
    CountryEntry { code: "MZ" },
    CountryEntry { code: "NG" },
    CountryEntry { code: "NL" },
    CountryEntry { code: "NO" },
    CountryEntry { code: "NZ" },
    CountryEntry { code: "OM" },
    CountryEntry { code: "PA" },
    CountryEntry { code: "PE" },
    CountryEntry { code: "PH" },
    CountryEntry { code: "PK" },
    CountryEntry { code: "PL" },
    CountryEntry { code: "PT" },
    CountryEntry { code: "QA" },
    CountryEntry { code: "RO" },
    CountryEntry { code: "RS" },
    CountryEntry { code: "RU" },
    CountryEntry { code: "SA" },
    CountryEntry { code: "SE" },
    CountryEntry { code: "SG" },
    CountryEntry { code: "SI" },
    CountryEntry { code: "SK" },
    CountryEntry { code: "SV" },
    CountryEntry { code: "TH" },
    CountryEntry { code: "TJ" },
    CountryEntry { code: "TN" },
    CountryEntry { code: "TR" },
    CountryEntry { code: "TW" },
    CountryEntry { code: "TZ" },
    CountryEntry { code: "UA" },
    CountryEntry { code: "UG" },
    CountryEntry { code: "US" },
    CountryEntry { code: "UY" },
    CountryEntry { code: "UZ" },
    CountryEntry { code: "VE" },
    CountryEntry { code: "VN" },
];

const ROM_COUNTRY_NAMES_EN: [&str; ROM_COUNTRY_COUNT] = [
    "United Arab Emirates",
    "Armenia",
    "Argentina",
    "Austria",
    "Australia",
    "Azerbaijan",
    "Belgium",
    "Bulgaria",
    "Bahrain",
    "Brazil",
    "Canada",
    "Switzerland",
    "Chile",
    "China",
    "Colombia",
    "Costa Rica",
    "Cyprus",
    "Czechia",
    "Germany",
    "Denmark",
    "Ecuador",
    "Estonia",
    "Egypt",
    "Spain",
    "Finland",
    "France",
    "United Kingdom",
    "Georgia",
    "Ghana",
    "Greece",
    "Guatemala",
    "Hong Kong SAR China",
    "Croatia",
    "Hungary",
    "Indonesia",
    "Israel",
    "India",
    "Iceland",
    "Italy",
    "Jordan",
    "Japan",
    "Kenya",
    "Kyrgyzstan",
    "South Korea",
    "Kuwait",
    "Kazakhstan",
    "Lebanon",
    "Lithuania",
    "Latvia",
    "Morocco",
    "Moldova",
    "Mexico",
    "Malaysia",
    "Mozambique",
    "Nigeria",
    "Netherlands",
    "Norway",
    "New Zealand",
    "Oman",
    "Panama",
    "Peru",
    "Philippines",
    "Pakistan",
    "Poland",
    "Portugal",
    "Qatar",
    "Romania",
    "Serbia",
    "Russia",
    "Saudi Arabia",
    "Sweden",
    "Singapore",
    "Slovenia",
    "Slovakia",
    "El Salvador",
    "Thailand",
    "Tajikistan",
    "Tunisia",
    "Türkiye",
    "Taiwan",
    "Tanzania",
    "Ukraine",
    "Uganda",
    "United States",
    "Uruguay",
    "Uzbekistan",
    "Venezuela",
    "Vietnam",
];

const ROM_COUNTRY_NAMES_KO: [&str; ROM_COUNTRY_COUNT] = [
    "아랍에미리트",
    "아르메니아",
    "아르헨티나",
    "오스트리아",
    "오스트레일리아",
    "아제르바이잔",
    "벨기에",
    "불가리아",
    "바레인",
    "브라질",
    "캐나다",
    "스위스",
    "칠레",
    "중국",
    "콜롬비아",
    "코스타리카",
    "키프로스",
    "체코",
    "독일",
    "덴마크",
    "에콰도르",
    "에스토니아",
    "이집트",
    "스페인",
    "핀란드",
    "프랑스",
    "영국",
    "조지아",
    "가나",
    "그리스",
    "과테말라",
    "홍콩(중국 특별행정구)",
    "크로아티아",
    "헝가리",
    "인도네시아",
    "이스라엘",
    "인도",
    "아이슬란드",
    "이탈리아",
    "요르단",
    "일본",
    "케냐",
    "키르기스스탄",
    "대한민국",
    "쿠웨이트",
    "카자흐스탄",
    "레바논",
    "리투아니아",
    "라트비아",
    "모로코",
    "몰도바",
    "멕시코",
    "말레이시아",
    "모잠비크",
    "나이지리아",
    "네덜란드",
    "노르웨이",
    "뉴질랜드",
    "오만",
    "파나마",
    "페루",
    "필리핀",
    "파키스탄",
    "폴란드",
    "포르투갈",
    "카타르",
    "루마니아",
    "세르비아",
    "러시아",
    "사우디아라비아",
    "스웨덴",
    "싱가포르",
    "슬로베니아",
    "슬로바키아",
    "엘살바도르",
    "태국",
    "타지키스탄",
    "튀니지",
    "튀르키예",
    "대만",
    "탄자니아",
    "우크라이나",
    "우간다",
    "미국",
    "우루과이",
    "우즈베키스탄",
    "베네수엘라",
    "베트남",
];

const ROM_COUNTRY_NAMES_RU: [&str; ROM_COUNTRY_COUNT] = [
    "ОАЭ",
    "Армения",
    "Аргентина",
    "Австрия",
    "Австралия",
    "Азербайджан",
    "Бельгия",
    "Болгария",
    "Бахрейн",
    "Бразилия",
    "Канада",
    "Швейцария",
    "Чили",
    "Китай",
    "Колумбия",
    "Коста-Рика",
    "Кипр",
    "Чехия",
    "Германия",
    "Дания",
    "Эквадор",
    "Эстония",
    "Египет",
    "Испания",
    "Финляндия",
    "Франция",
    "Великобритания",
    "Грузия",
    "Гана",
    "Греция",
    "Гватемала",
    "Гонконг (САР)",
    "Хорватия",
    "Венгрия",
    "Индонезия",
    "Израиль",
    "Индия",
    "Исландия",
    "Италия",
    "Иордания",
    "Япония",
    "Кения",
    "Киргизия",
    "Республика Корея",
    "Кувейт",
    "Казахстан",
    "Ливан",
    "Литва",
    "Латвия",
    "Марокко",
    "Молдова",
    "Мексика",
    "Малайзия",
    "Мозамбик",
    "Нигерия",
    "Нидерланды",
    "Норвегия",
    "Новая Зеландия",
    "Оман",
    "Панама",
    "Перу",
    "Филиппины",
    "Пакистан",
    "Польша",
    "Португалия",
    "Катар",
    "Румыния",
    "Сербия",
    "Россия",
    "Саудовская Аравия",
    "Швеция",
    "Сингапур",
    "Словения",
    "Словакия",
    "Сальвадор",
    "Таиланд",
    "Таджикистан",
    "Тунис",
    "Турция",
    "Тайвань",
    "Танзания",
    "Украина",
    "Уганда",
    "Соединенные Штаты",
    "Уругвай",
    "Узбекистан",
    "Венесуэла",
    "Вьетнам",
];

const ROM_COUNTRY_NAMES_JA: [&str; ROM_COUNTRY_COUNT] = [
    "アラブ首長国連邦",
    "アルメニア",
    "アルゼンチン",
    "オーストリア",
    "オーストラリア",
    "アゼルバイジャン",
    "ベルギー",
    "ブルガリア",
    "バーレーン",
    "ブラジル",
    "カナダ",
    "スイス",
    "チリ",
    "中国",
    "コロンビア",
    "コスタリカ",
    "キプロス",
    "チェコ",
    "ドイツ",
    "デンマーク",
    "エクアドル",
    "エストニア",
    "エジプト",
    "スペイン",
    "フィンランド",
    "フランス",
    "イギリス",
    "ジョージア",
    "ガーナ",
    "ギリシャ",
    "グアテマラ",
    "中華人民共和国香港特別行政区",
    "クロアチア",
    "ハンガリー",
    "インドネシア",
    "イスラエル",
    "インド",
    "アイスランド",
    "イタリア",
    "ヨルダン",
    "日本",
    "ケニア",
    "キルギス",
    "韓国",
    "クウェート",
    "カザフスタン",
    "レバノン",
    "リトアニア",
    "ラトビア",
    "モロッコ",
    "モルドバ",
    "メキシコ",
    "マレーシア",
    "モザンビーク",
    "ナイジェリア",
    "オランダ",
    "ノルウェー",
    "ニュージーランド",
    "オマーン",
    "パナマ",
    "ペルー",
    "フィリピン",
    "パキスタン",
    "ポーランド",
    "ポルトガル",
    "カタール",
    "ルーマニア",
    "セルビア",
    "ロシア",
    "サウジアラビア",
    "スウェーデン",
    "シンガポール",
    "スロベニア",
    "スロバキア",
    "エルサルバドル",
    "タイ",
    "タジキスタン",
    "チュニジア",
    "トルコ",
    "台湾",
    "タンザニア",
    "ウクライナ",
    "ウガンダ",
    "アメリカ合衆国",
    "ウルグアイ",
    "ウズベキスタン",
    "ベネズエラ",
    "ベトナム",
];

const ROM_COUNTRY_NAMES_ZH_TW: [&str; ROM_COUNTRY_COUNT] = [
    "阿拉伯聯合大公國",
    "亞美尼亞",
    "阿根廷",
    "奧地利",
    "澳洲",
    "亞塞拜然",
    "比利時",
    "保加利亞",
    "巴林",
    "巴西",
    "加拿大",
    "瑞士",
    "智利",
    "中國",
    "哥倫比亞",
    "哥斯大黎加",
    "賽普勒斯",
    "捷克",
    "德國",
    "丹麥",
    "厄瓜多",
    "愛沙尼亞",
    "埃及",
    "西班牙",
    "芬蘭",
    "法國",
    "英國",
    "喬治亞",
    "迦納",
    "希臘",
    "瓜地馬拉",
    "中國香港特別行政區",
    "克羅埃西亞",
    "匈牙利",
    "印尼",
    "以色列",
    "印度",
    "冰島",
    "義大利",
    "約旦",
    "日本",
    "肯亞",
    "吉爾吉斯",
    "南韓",
    "科威特",
    "哈薩克",
    "黎巴嫩",
    "立陶宛",
    "拉脫維亞",
    "摩洛哥",
    "摩爾多瓦",
    "墨西哥",
    "馬來西亞",
    "莫三比克",
    "奈及利亞",
    "荷蘭",
    "挪威",
    "紐西蘭",
    "阿曼",
    "巴拿馬",
    "秘魯",
    "菲律賓",
    "巴基斯坦",
    "波蘭",
    "葡萄牙",
    "卡達",
    "羅馬尼亞",
    "塞爾維亞",
    "俄羅斯",
    "沙烏地阿拉伯",
    "瑞典",
    "新加坡",
    "斯洛維尼亞",
    "斯洛伐克",
    "薩爾瓦多",
    "泰國",
    "塔吉克",
    "突尼西亞",
    "土耳其",
    "台灣",
    "坦尚尼亞",
    "烏克蘭",
    "烏干達",
    "美國",
    "烏拉圭",
    "烏茲別克",
    "委內瑞拉",
    "越南",
];

const ROM_COUNTRY_NAMES_VI: [&str; ROM_COUNTRY_COUNT] = [
    "Các Tiểu Vương quốc Ả Rập Thống nhất",
    "Armenia",
    "Argentina",
    "Áo",
    "Australia",
    "Azerbaijan",
    "Bỉ",
    "Bulgaria",
    "Bahrain",
    "Brazil",
    "Canada",
    "Thụy Sĩ",
    "Chile",
    "Trung Quốc",
    "Colombia",
    "Costa Rica",
    "Síp",
    "Séc",
    "Đức",
    "Đan Mạch",
    "Ecuador",
    "Estonia",
    "Ai Cập",
    "Tây Ban Nha",
    "Phần Lan",
    "Pháp",
    "Vương quốc Anh",
    "Georgia",
    "Ghana",
    "Hy Lạp",
    "Guatemala",
    "Đặc khu Hành chính Hồng Kông, Trung Quốc",
    "Croatia",
    "Hungary",
    "Indonesia",
    "Israel",
    "Ấn Độ",
    "Iceland",
    "Italy",
    "Jordan",
    "Nhật Bản",
    "Kenya",
    "Kyrgyzstan",
    "Hàn Quốc",
    "Kuwait",
    "Kazakhstan",
    "Li-băng",
    "Litva",
    "Latvia",
    "Ma-rốc",
    "Moldova",
    "Mexico",
    "Malaysia",
    "Mozambique",
    "Nigeria",
    "Hà Lan",
    "Na Uy",
    "New Zealand",
    "Oman",
    "Panama",
    "Peru",
    "Philippines",
    "Pakistan",
    "Ba Lan",
    "Bồ Đào Nha",
    "Qatar",
    "Romania",
    "Serbia",
    "Nga",
    "Ả Rập Xê-út",
    "Thụy Điển",
    "Singapore",
    "Slovenia",
    "Slovakia",
    "El Salvador",
    "Thái Lan",
    "Tajikistan",
    "Tunisia",
    "Thổ Nhĩ Kỳ",
    "Đài Loan",
    "Tanzania",
    "Ukraina",
    "Uganda",
    "Hoa Kỳ",
    "Uruguay",
    "Uzbekistan",
    "Venezuela",
    "Việt Nam",
];

const ROM_COUNTRY_NAMES_EL: [&str; ROM_COUNTRY_COUNT] = [
    "Ηνωμένα Αραβικά Εμιράτα",
    "Αρμενία",
    "Αργεντινή",
    "Αυστρία",
    "Αυστραλία",
    "Αζερμπαϊτζάν",
    "Βέλγιο",
    "Βουλγαρία",
    "Μπαχρέιν",
    "Βραζιλία",
    "Καναδάς",
    "Ελβετία",
    "Χιλή",
    "Κίνα",
    "Κολομβία",
    "Κόστα Ρίκα",
    "Κύπρος",
    "Τσεχία",
    "Γερμανία",
    "Δανία",
    "Ισημερινός",
    "Εσθονία",
    "Αίγυπτος",
    "Ισπανία",
    "Φινλανδία",
    "Γαλλία",
    "Ηνωμένο Βασίλειο",
    "Γεωργία",
    "Γκάνα",
    "Ελλάδα",
    "Γουατεμάλα",
    "Χονγκ Κονγκ ΕΔΠ Κίνας",
    "Κροατία",
    "Ουγγαρία",
    "Ινδονησία",
    "Ισραήλ",
    "Ινδία",
    "Ισλανδία",
    "Ιταλία",
    "Ιορδανία",
    "Ιαπωνία",
    "Κένυα",
    "Κιργιστάν",
    "Νότια Κορέα",
    "Κουβέιτ",
    "Καζακστάν",
    "Λίβανος",
    "Λιθουανία",
    "Λετονία",
    "Μαρόκο",
    "Μολδαβία",
    "Μεξικό",
    "Μαλαισία",
    "Μοζαμβίκη",
    "Νιγηρία",
    "Κάτω Χώρες",
    "Νορβηγία",
    "Νέα Ζηλανδία",
    "Ομάν",
    "Παναμάς",
    "Περού",
    "Φιλιππίνες",
    "Πακιστάν",
    "Πολωνία",
    "Πορτογαλία",
    "Κατάρ",
    "Ρουμανία",
    "Σερβία",
    "Ρωσία",
    "Σαουδική Αραβία",
    "Σουηδία",
    "Σιγκαπούρη",
    "Σλοβενία",
    "Σλοβακία",
    "Ελ Σαλβαδόρ",
    "Ταϊλάνδη",
    "Τατζικιστάν",
    "Τυνησία",
    "Τουρκία",
    "Ταϊβάν",
    "Τανζανία",
    "Ουκρανία",
    "Ουγκάντα",
    "Ηνωμένες Πολιτείες",
    "Ουρουγουάη",
    "Ουζμπεκιστάν",
    "Βενεζουέλα",
    "Βιετνάμ",
];

const ROM_COUNTRY_NAMES_HI: [&str; ROM_COUNTRY_COUNT] = [
    "संयुक्त अरब अमीरात",
    "आर्मेनिया",
    "अर्जेंटीना",
    "ऑस्ट्रिया",
    "ऑस्ट्रेलिया",
    "अज़रबैजान",
    "बेल्जियम",
    "बुल्गारिया",
    "बहरीन",
    "ब्राज़ील",
    "कनाडा",
    "स्विट्ज़रलैंड",
    "चिली",
    "चीन",
    "कोलंबिया",
    "कोस्टारिका",
    "साइप्रस",
    "चेकिया",
    "जर्मनी",
    "डेनमार्क",
    "इक्वाडोर",
    "एस्टोनिया",
    "मिस्र",
    "स्पेन",
    "फ़िनलैंड",
    "फ़्रांस",
    "यूनाइटेड किंगडम",
    "जॉर्जिया",
    "घाना",
    "यूनान",
    "ग्वाटेमाला",
    "हाँग काँग (चीन विशेष प्रशासनिक क्षेत्र)",
    "क्रोएशिया",
    "हंगरी",
    "इंडोनेशिया",
    "इज़राइल",
    "भारत",
    "आइसलैंड",
    "इटली",
    "जॉर्डन",
    "जापान",
    "केन्या",
    "किर्गिज़स्तान",
    "दक्षिण कोरिया",
    "कुवैत",
    "कज़ाखस्तान",
    "लेबनान",
    "लिथुआनिया",
    "लातविया",
    "मोरक्को",
    "मॉल्डोवा",
    "मैक्सिको",
    "मलेशिया",
    "मोज़ांबिक",
    "नाइजीरिया",
    "नीदरलैंड",
    "नॉर्वे",
    "न्यूज़ीलैंड",
    "ओमान",
    "पनामा",
    "पेरू",
    "फ़िलिपींस",
    "पाकिस्तान",
    "पोलैंड",
    "पुर्तगाल",
    "क़तर",
    "रोमानिया",
    "सर्बिया",
    "रूस",
    "सऊदी अरब",
    "स्वीडन",
    "सिंगापुर",
    "स्लोवेनिया",
    "स्लोवाकिया",
    "अल सल्वाडोर",
    "थाईलैंड",
    "ताजिकिस्तान",
    "ट्यूनीशिया",
    "तुर्किये",
    "ताइवान",
    "तंज़ानिया",
    "यूक्रेन",
    "युगांडा",
    "संयुक्त राज्य",
    "उरूग्वे",
    "उज़्बेकिस्तान",
    "वेनेज़ुएला",
    "वियतनाम",
];

const ROM_COUNTRY_NAMES_KA: [&str; ROM_COUNTRY_COUNT] = [
    "არაბთა გაერთიანებული საამიროები",
    "სომხეთი",
    "არგენტინა",
    "ავსტრია",
    "ავსტრალია",
    "აზერბაიჯანი",
    "ბელგია",
    "ბულგარეთი",
    "ბაჰრეინი",
    "ბრაზილია",
    "კანადა",
    "შვეიცარია",
    "ჩილე",
    "ჩინეთი",
    "კოლუმბია",
    "კოსტა-რიკა",
    "კვიპროსი",
    "ჩეხეთი",
    "გერმანია",
    "დანია",
    "ეკვადორი",
    "ესტონეთი",
    "ეგვიპტე",
    "ესპანეთი",
    "ფინეთი",
    "საფრანგეთი",
    "გაერთიანებული სამეფო",
    "საქართველო",
    "განა",
    "საბერძნეთი",
    "გვატემალა",
    "ჰონკონგის სპეციალური ადმინისტრაციული რეგიონი, ჩინეთი",
    "ხორვატია",
    "უნგრეთი",
    "ინდონეზია",
    "ისრაელი",
    "ინდოეთი",
    "ისლანდია",
    "იტალია",
    "იორდანია",
    "იაპონია",
    "კენია",
    "ყირგიზეთი",
    "სამხრეთ კორეა",
    "ქუვეითი",
    "ყაზახეთი",
    "ლიბანი",
    "ლიეტუვა",
    "ლატვია",
    "მაროკო",
    "მოლდოვა",
    "მექსიკა",
    "მალაიზია",
    "მოზამბიკი",
    "ნიგერია",
    "ნიდერლანდები",
    "ნორვეგია",
    "ახალი ზელანდია",
    "ომანი",
    "პანამა",
    "პერუ",
    "ფილიპინები",
    "პაკისტანი",
    "პოლონეთი",
    "პორტუგალია",
    "კატარი",
    "რუმინეთი",
    "სერბეთი",
    "რუსეთი",
    "საუდის არაბეთი",
    "შვედეთი",
    "სინგაპური",
    "სლოვენია",
    "სლოვაკეთი",
    "სალვადორი",
    "ტაილანდი",
    "ტაჯიკეთი",
    "ტუნისი",
    "თურქეთი",
    "ტაივანი",
    "ტანზანია",
    "უკრაინა",
    "უგანდა",
    "ამერიკის შეერთებული შტატები",
    "ურუგვაი",
    "უზბეკეთი",
    "ვენესუელა",
    "ვიეტნამი",
];

const ROM_COUNTRY_NAMES_NL: [&str; ROM_COUNTRY_COUNT] = [
    "Verenigde Arabische Emiraten",
    "Armenië",
    "Argentinië",
    "Oostenrijk",
    "Australië",
    "Azerbeidzjan",
    "België",
    "Bulgarije",
    "Bahrein",
    "Brazilië",
    "Canada",
    "Zwitserland",
    "Chili",
    "China",
    "Colombia",
    "Costa Rica",
    "Cyprus",
    "Tsjechië",
    "Duitsland",
    "Denemarken",
    "Ecuador",
    "Estland",
    "Egypte",
    "Spanje",
    "Finland",
    "Frankrijk",
    "Verenigd Koninkrijk",
    "Georgië",
    "Ghana",
    "Griekenland",
    "Guatemala",
    "Hongkong SAR van China",
    "Kroatië",
    "Hongarije",
    "Indonesië",
    "Israël",
    "India",
    "IJsland",
    "Italië",
    "Jordanië",
    "Japan",
    "Kenia",
    "Kirgizië",
    "Zuid-Korea",
    "Koeweit",
    "Kazachstan",
    "Libanon",
    "Litouwen",
    "Letland",
    "Marokko",
    "Moldavië",
    "Mexico",
    "Maleisië",
    "Mozambique",
    "Nigeria",
    "Nederland",
    "Noorwegen",
    "Nieuw-Zeeland",
    "Oman",
    "Panama",
    "Peru",
    "Filipijnen",
    "Pakistan",
    "Polen",
    "Portugal",
    "Qatar",
    "Roemenië",
    "Servië",
    "Rusland",
    "Saoedi-Arabië",
    "Zweden",
    "Singapore",
    "Slovenië",
    "Slowakije",
    "El Salvador",
    "Thailand",
    "Tadzjikistan",
    "Tunesië",
    "Turkije",
    "Taiwan",
    "Tanzania",
    "Oekraïne",
    "Oeganda",
    "Verenigde Staten",
    "Uruguay",
    "Oezbekistan",
    "Venezuela",
    "Vietnam",
];

const ROM_COUNTRY_NAMES_AR: [&str; ROM_COUNTRY_COUNT] = [
    "الإمارات العربية المتحدة",
    "أرمينيا",
    "الأرجنتين",
    "النمسا",
    "أستراليا",
    "أذربيجان",
    "بلجيكا",
    "بلغاريا",
    "البحرين",
    "البرازيل",
    "كندا",
    "سويسرا",
    "تشيلي",
    "الصين",
    "كولومبيا",
    "كوستاريكا",
    "قبرص",
    "التشيك",
    "ألمانيا",
    "الدانمرك",
    "الإكوادور",
    "إستونيا",
    "مصر",
    "إسبانيا",
    "فنلندا",
    "فرنسا",
    "المملكة المتحدة",
    "جورجيا",
    "غانا",
    "اليونان",
    "غواتيمالا",
    "هونغ كونغ الصينية (منطقة إدارية خاصة)",
    "كرواتيا",
    "هنغاريا",
    "إندونيسيا",
    "إسرائيل",
    "الهند",
    "آيسلندا",
    "إيطاليا",
    "الأردن",
    "اليابان",
    "كينيا",
    "قيرغيزستان",
    "كوريا الجنوبية",
    "الكويت",
    "كازاخستان",
    "لبنان",
    "ليتوانيا",
    "لاتفيا",
    "المغرب",
    "مولدوفا",
    "المكسيك",
    "ماليزيا",
    "موزمبيق",
    "نيجيريا",
    "هولندا",
    "النرويج",
    "نيوزيلندا",
    "عُمان",
    "بنما",
    "بيرو",
    "الفلبين",
    "باكستان",
    "بولندا",
    "البرتغال",
    "قطر",
    "رومانيا",
    "صربيا",
    "روسيا",
    "المملكة العربية السعودية",
    "السويد",
    "سنغافورة",
    "سلوفينيا",
    "سلوفاكيا",
    "السلفادور",
    "تايلاند",
    "طاجيكستان",
    "تونس",
    "تركيا",
    "تايوان",
    "تنزانيا",
    "أوكرانيا",
    "أوغندا",
    "الولايات المتحدة",
    "أورغواي",
    "أوزبكستان",
    "فنزويلا",
    "فيتنام",
];

const ROM_COUNTRY_NAMES_ES: [&str; ROM_COUNTRY_COUNT] = [
    "Emiratos Árabes Unidos",
    "Armenia",
    "Argentina",
    "Austria",
    "Australia",
    "Azerbaiyán",
    "Bélgica",
    "Bulgaria",
    "Baréin",
    "Brasil",
    "Canadá",
    "Suiza",
    "Chile",
    "China",
    "Colombia",
    "Costa Rica",
    "Chipre",
    "Chequia",
    "Alemania",
    "Dinamarca",
    "Ecuador",
    "Estonia",
    "Egipto",
    "España",
    "Finlandia",
    "Francia",
    "Reino Unido",
    "Georgia",
    "Ghana",
    "Grecia",
    "Guatemala",
    "RAE de Hong Kong (China)",
    "Croacia",
    "Hungría",
    "Indonesia",
    "Israel",
    "India",
    "Islandia",
    "Italia",
    "Jordania",
    "Japón",
    "Kenia",
    "Kirguistán",
    "Corea del Sur",
    "Kuwait",
    "Kazajistán",
    "Líbano",
    "Lituania",
    "Letonia",
    "Marruecos",
    "Moldavia",
    "México",
    "Malasia",
    "Mozambique",
    "Nigeria",
    "Países Bajos",
    "Noruega",
    "Nueva Zelanda",
    "Omán",
    "Panamá",
    "Perú",
    "Filipinas",
    "Pakistán",
    "Polonia",
    "Portugal",
    "Catar",
    "Rumanía",
    "Serbia",
    "Rusia",
    "Arabia Saudí",
    "Suecia",
    "Singapur",
    "Eslovenia",
    "Eslovaquia",
    "El Salvador",
    "Tailandia",
    "Tayikistán",
    "Túnez",
    "Turquía",
    "Taiwán",
    "Tanzania",
    "Ucrania",
    "Uganda",
    "Estados Unidos",
    "Uruguay",
    "Uzbekistán",
    "Venezuela",
    "Vietnam",
];

fn localized_country_name(index: usize, language: LanguageOption) -> &'static str {
    match language {
        LanguageOption::English => ROM_COUNTRY_NAMES_EN[index],
        LanguageOption::Korean => ROM_COUNTRY_NAMES_KO[index],
        LanguageOption::Russian => ROM_COUNTRY_NAMES_RU[index],
        LanguageOption::Japanese => ROM_COUNTRY_NAMES_JA[index],
        LanguageOption::TraditionalChinese => ROM_COUNTRY_NAMES_ZH_TW[index],
        LanguageOption::Vietnamese => ROM_COUNTRY_NAMES_VI[index],
        LanguageOption::Greek => ROM_COUNTRY_NAMES_EL[index],
        LanguageOption::Hindi => ROM_COUNTRY_NAMES_HI[index],
        LanguageOption::Georgian => ROM_COUNTRY_NAMES_KA[index],
        LanguageOption::Dutch => ROM_COUNTRY_NAMES_NL[index],
        LanguageOption::Arabic => ROM_COUNTRY_NAMES_AR[index],
        LanguageOption::Spanish => ROM_COUNTRY_NAMES_ES[index],
    }
}

#[derive(Debug, Clone)]
enum Message {
    Noop,
    SelectImageFolder,
    CheckFirmware,
    FirmwareChecked(Result<RomFirmwareCheckResult, String>),

    DashboardRefreshTick,
    DashboardLoaded(Result<DashboardSnapshot, String>),
    DashboardModelTimeoutTick,

    StartConvertWipe,
    StartRowUpdateKeepData,
    StartReinstallWipe,
    BackupProinfo,
    InstallMtkDriver,
    InstallVcRuntime,
    DrainLiveEvents,
    ExportLog,
    ClearLog,

    SelectNav(NavPage),
    SidebarHoverEnter,
    SidebarHoverExit,
    SidebarAnimTick,

    RomCardHoverEnter(RomSlideTarget),
    RomCardHoverExit(RomSlideTarget),
    RomSlideTick,
    RomCheckLoadingTick,
    RomProceedToRoutine,
    RomBackToImageInfo,
    RomOpenOptions(RomSlideTarget),
    RomBackToRoutineSelect,
    RomToggleDataWipe,
    RomCountrySelect,
    RomSelectCountry(String),
    RomCloseCountryPopup,
    RomContinueFromOptions,
    RomOpenCountryCodeModal,
    RomCloseCountryCodeModal,
    RomSelectCountryCode(&'static str),
    RomCountryCodeSearchChanged(String),
    AdditionalOpenCountryReset,
    StartOtaDisable,
    StartOtaEnable,
    DashboardOpenRomFolderSelect,
    OpenFirmwareDownload,
    OpenQna,
    OpenDeveloperYoutube,
    OpenProgramVideos,
    OpenDonate,
    OpenFeedback,
    CheckProgramUpdate,
    ProgramUpdateChecked(Result<ProgramUpdateCheckResult, String>),
    DashboardProgramUpdateChecked(Result<ProgramUpdateCheckResult, String>),
    OpenProgramUpdateRelease,
    DismissProgramUpdateNotice,
    SettingsLanguageSelected(LanguageOption),
    WindowCloseRequested,
}

const LPM_NAV_MAIN: &[(NavPage, &'static [u8], &str)] = &[
    (NavPage::Dashboard, NAV_HOME_ICON_BYTES, "대시보드"),
    (NavPage::Rom, NAV_REFRESH_ICON_BYTES, "ROM 작업"),
    (NavPage::Driver, NAV_TAB_SETTINGS_ICON_BYTES, "추가 옵션"),
];

const LPM_NAV_TOOLS: &[(NavPage, &'static [u8], &str)] = &[
    (
        NavPage::FirmwareDownload,
        NAV_FIRMWARE_DOWNLOAD_ICON_BYTES,
        "펌웨어 다운로드",
    ),
    (NavPage::Qna, NAV_QNA_ICON_BYTES, "QnA"),
    (NavPage::Log, NAV_LOG_ICON_BYTES, "로그 관리"),
    (NavPage::Settings, NAV_SETTINGS_ICON_BYTES, "설정"),
];

fn lpm_nav_button<'a>(
    page: NavPage,
    icon_handle: iced::widget::image::Handle,
    label: &'static str,
    active: bool,
    label_alpha: f32,
) -> Element<'a, Message> {
    let icon_pill: Element<'a, Message> = container(
        iced::widget::image(icon_handle)
            .width(Length::Fixed(21.0))
            .height(Length::Fixed(21.0)),
    )
    .width(Length::Fixed(32.0))
    .height(Length::Fixed(28.0))
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .style(move |_theme: &Theme| {
        if active {
            container::Style {
                background: Some(Background::Color(Color::from_rgb8(224, 228, 255))),
                text_color: Some(Color::from_rgb8(43, 58, 118)),
                border: iced::Border {
                    radius: 14.0.into(),
                    ..iced::Border::default()
                },
                ..container::Style::default()
            }
        } else {
            container::Style::default()
        }
    })
    .into();

    let mut inner = row![icon_pill]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    if label_alpha > 0.0 {
        let alpha = label_alpha;

        inner = inner.push(
            text(label)
                .size(lpm_sidebar_label_size())
                .height(Length::Fill)
                .align_y(iced::alignment::Vertical::Center)
                .wrapping(iced::widget::text::Wrapping::Word)
                .style(move |_theme: &Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(32.0 / 255.0, 35.0 / 255.0, 47.0 / 255.0, alpha)),
                }),
        );
    }

    let content = container(inner)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(iced::Alignment::Center);

    button(content)
        .padding([0.0, 15.0])
        .width(Length::Fill)
        .height(Length::Fixed(NAV_BTN_HEIGHT))
        .on_press(Message::SelectNav(page))
        .style(move |_theme: &Theme, status| {
            let hovered = matches!(status, iced::widget::button::Status::Hovered);

            iced::widget::button::Style {
                background: if hovered {
                    Some(Background::Color(Color::from_rgb8(233, 235, 246)))
                } else {
                    None
                },
                text_color: Color::from_rgb8(32, 35, 47),
                border: iced::Border {
                    radius: 18.0.into(),
                    ..iced::Border::default()
                },
                ..iced::widget::button::Style::default()
            }
        })
        .into()
}

fn lpm_nav_section_header<'a>(label: &'static str, label_alpha: f32) -> Element<'a, Message> {
    let alpha = if label_alpha > 0.08 { label_alpha } else { 0.0 };

    let header_content: Element<'a, Message> = if alpha > 0.0 {
        text(label)
            .size(11)
            .font(lpm_bold_font())
            .width(Length::Fixed(SIDEBAR_EXPANDED_WIDTH - 30.0))
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .wrapping(iced::widget::text::Wrapping::None)
            .style(move |_theme: &Theme| iced::widget::text::Style {
                color: Some(Color::from_rgba(
                    113.0 / 255.0,
                    116.0 / 255.0,
                    130.0 / 255.0,
                    alpha,
                )),
            })
            .into()
    } else {
        iced::widget::Space::new()
            .width(Length::Fixed(SIDEBAR_EXPANDED_WIDTH - 30.0))
            .height(Length::Fill)
            .into()
    };

    container(header_content)
        .width(Length::Fill)
        .height(Length::Fixed(31.0))
        .padding([0.0, 0.0])
        .clip(true)
        .style(lpm_nav_section_label_style)
        .into()
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn lpm_approach_anim_value(current: f32, target: f32, factor: f32, threshold: f32) -> f32 {
    let delta = target - current;

    if delta.abs() <= threshold {
        target
    } else {
        current + delta * factor
    }
}

fn nav_page_title(page: NavPage) -> &'static str {
    match page {
        NavPage::Dashboard => "대시보드",
        NavPage::Rom => "image 폴더 선택",
        NavPage::Backup => "proinfo 백업",
        NavPage::Driver => "기기 관리",
        NavPage::FirmwareDownload => "펌웨어 다운로드",
        NavPage::Qna => "QnA",
        NavPage::Log => "로그 관리",
        NavPage::Settings => "설정",
    }
}

fn nav_page_subtitle(page: NavPage) -> &'static str {
    match page {
        NavPage::Dashboard => "LPMBox 작업 상태와 주요 기능을 한 화면에서 관리합니다.",
        NavPage::Rom => "image 폴더를 선택해주세요.",
        NavPage::Backup => "proinfo 파티션을 백업합니다.",
        NavPage::Driver => "기기를 추가적으로 설정합니다.",
        NavPage::FirmwareDownload => "샤오신패드 펌웨어 다운로드 페이지로 이동합니다.",
        NavPage::Qna => "LPMBOX QnA 페이지로 이동합니다.",
        NavPage::Log => "작업 로그를 확인하고 텍스트 파일로 저장합니다.",
        NavPage::Settings => "LPMBOX 프로그램을 설정합니다.",
    }
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let (initial_language, initial_language_source) = initial_language_option_with_source();
        set_active_language_option(initial_language);

    let mut app = Self {
        image_dir: None,
        busy: false,
        log_lines: Vec::new(),
        log_text_cache: build_log_text(&[]),
        log_display_rows_cache: build_log_display_rows(&[]),
        log_cache_dirty: false,
        log_refresh_suspended: false,

        rom_firmware_info: None,
        rom_firmware_error: None,
        rom_mtk_driver_installed: None,
        rom_mtk_driver_error: None,
        rom_vc_runtime_status: None,
        rom_vc_runtime_error: None,

        live_rx: None,
        progress_line_indices: HashMap::new(),
        active_spinners: HashMap::new(),
        spinner_tick: 0,
        rom_check_loading_frame: 0,
        last_spft_stage: None,

        active_nav: NavPage::Dashboard,
        sidebar_expanded: false,
        sidebar_anim: 0.0,
        sidebar_velocity: 0.0,

        dashboard_info: lpmbox_device::DashboardDeviceInfo::default(),
        dashboard_refreshing: false,
        battery_progress_level: None,
        battery_progress_handle: build_battery_progress_handle(None),

        rom_show_routine_select: false,
        rom_option_target: None,
        running_rom_target: None,
        rom_option_data_wipe: true,
        rom_option_data_wipe_locked: true,
        rom_option_country_code: None,
        rom_country_popup_open: false,

        dashboard_model_image: DashboardModelImage::Unknown,
        dashboard_model_missing_since: None,
        model_tb375_handle: iced::widget::image::Handle::from_bytes(MODEL_TB375FC_IMAGE_BYTES.to_vec()),
        model_tb365_handle: iced::widget::image::Handle::from_bytes(MODEL_TB365FC_IMAGE_BYTES.to_vec()),
        model_tb335_handle: iced::widget::image::Handle::from_bytes(MODEL_TB335FC_IMAGE_BYTES.to_vec()),
        rom_install_icon_handle: smooth_png_handle(ROM_INSTALL_ICON_BYTES, 150, 150),
        rom_update_icon_handle: smooth_png_handle(ROM_UPDATE_ICON_BYTES, 150, 150),
        folder_select_icon_handle: smooth_png_handle(FOLDER_SELECT_ICON_BYTES, 96, 96),
        folder_check_icon_handle: smooth_png_handle(FOLDER_CHECK_ICON_BYTES, 96, 96),
        tablet_check_icon_handle: smooth_png_handle(TABLET_CHECK_ICON_BYTES, 92, 92),
        tablet_x_icon_handle: smooth_png_handle(TABLET_X_ICON_BYTES, 92, 92),
        tablet_fix_icon_handle: smooth_png_handle(TABLET_FIX_ICON_BYTES, 92, 92),
        slide_button_handle: iced::widget::image::Handle::from_bytes(SLIDE_BUTTON_BYTES.to_vec()),
        warning_icon_handle: smooth_png_handle(WARNING_ICON_BYTES, 96, 96),
        nav_home_handle: smooth_png_handle(NAV_HOME_ICON_BYTES, 21, 21),
        nav_refresh_handle: smooth_png_handle(NAV_REFRESH_ICON_BYTES, 21, 21),
        nav_tab_settings_handle: smooth_png_handle(NAV_TAB_SETTINGS_ICON_BYTES, 21, 21),
        nav_firmware_download_handle: smooth_png_handle(NAV_FIRMWARE_DOWNLOAD_ICON_BYTES, 21, 21),
        nav_qna_handle: smooth_png_handle(NAV_QNA_ICON_BYTES, 21, 21),
        nav_log_handle: smooth_png_handle(NAV_LOG_ICON_BYTES, 21, 21),
        nav_settings_handle: smooth_png_handle(NAV_SETTINGS_ICON_BYTES, 21, 21),
        loading_progress_handles: build_loading_progress_handles(),

        rom_install_hover_since: None,
        rom_update_hover_since: None,
        rom_reinstall_hover_since: None,
        rom_install_slide_width: 0.0,
        rom_update_slide_width: 0.0,
        rom_reinstall_slide_width: 0.0,
        rom_install_slide_velocity: 0.0,
        rom_update_slide_velocity: 0.0,
        rom_reinstall_slide_velocity: 0.0,
        rom_country_code_modal_open: false,
        rom_selected_country_code: None,
        rom_country_code_search: String::new(),
        additional_country_reset_pending: false,
        settings_language: initial_language,
        active_log_flow: None,
        program_update_checking: false,
        dashboard_update_notice: None,
    };

        app.push_log(format!(
            "[설정] 초기 언어 설정: {} / 기준: {}",
            initial_language,
            initial_language_source.label()
        ));
        app.push_log(format!(
            "[설정] 언어 설정 파일 경로: {}",
            lpm_language_config_path().display()
        ));

        (
            app,
            Task::perform(
                check_program_update_worker(APP_DISPLAY_VERSION.to_string()),
                Message::DashboardProgramUpdateChecked,
            ),
        )
    }

fn animate_rom_slide_width(current: f32, velocity: &mut f32, target: f32) -> f32 {
    *velocity = 0.0;
    lpm_approach_anim_value(
        current,
        target,
        ROM_SLIDE_ANIM_FACTOR,
        ROM_SLIDE_ANIM_THRESHOLD,
    )
    .clamp(0.0, ROM_ROUTINE_EXPAND_WIDTH)
}

fn reset_rom_routine_slide_state(&mut self) {
    self.rom_install_hover_since = None;
    self.rom_update_hover_since = None;
    self.rom_reinstall_hover_since = None;

    self.rom_install_slide_width = 0.0;
    self.rom_update_slide_width = 0.0;
    self.rom_reinstall_slide_width = 0.0;
    self.rom_install_slide_velocity = 0.0;
    self.rom_update_slide_velocity = 0.0;
    self.rom_reinstall_slide_velocity = 0.0;
}

fn rom_routine_slide_target_widths(&self) -> (f32, f32, f32) {
    let install_should_open = self
        .rom_install_hover_since
        .map(|since| since.elapsed() >= Duration::from_millis(ROM_HOVER_OPEN_DELAY_MS))
        .unwrap_or(false);

    let update_should_open = self
        .rom_update_hover_since
        .map(|since| since.elapsed() >= Duration::from_millis(ROM_HOVER_OPEN_DELAY_MS))
        .unwrap_or(false);

    let reinstall_should_open = self
        .rom_reinstall_hover_since
        .map(|since| since.elapsed() >= Duration::from_millis(ROM_HOVER_OPEN_DELAY_MS))
        .unwrap_or(false);

    (
        if install_should_open { ROM_ROUTINE_EXPAND_WIDTH } else { 0.0 },
        if update_should_open { ROM_ROUTINE_EXPAND_WIDTH } else { 0.0 },
        if reinstall_should_open { ROM_ROUTINE_EXPAND_WIDTH } else { 0.0 },
    )
}

fn rom_routine_slide_settled(&self) -> bool {
    let hover_delay = Duration::from_millis(ROM_HOVER_OPEN_DELAY_MS);
    let hover_delay_pending = self
        .rom_install_hover_since
        .or(self.rom_update_hover_since)
        .or(self.rom_reinstall_hover_since)
        .map(|since| since.elapsed() < hover_delay)
        .unwrap_or(false);

    if hover_delay_pending {
        return false;
    }

    let (install_target, update_target, reinstall_target) = self.rom_routine_slide_target_widths();

    (self.rom_install_slide_width - install_target).abs() < 0.25
        && (self.rom_update_slide_width - update_target).abs() < 0.25
        && (self.rom_reinstall_slide_width - reinstall_target).abs() < 0.25
        && self.rom_install_slide_velocity.abs() < 2.0
        && self.rom_update_slide_velocity.abs() < 2.0
        && self.rom_reinstall_slide_velocity.abs() < 2.0
}

fn sidebar_anim_settled(&self) -> bool {
    (self.sidebar_anim - self.sidebar_anim_target()).abs() < 0.001
        && self.sidebar_velocity.abs() < 0.05
}

fn ui_animation_active(&self) -> bool {
    !self.sidebar_anim_settled()
        || (self.active_nav == NavPage::Rom && !self.rom_routine_slide_settled())
}

fn reset_rom_check_loading_stack(&mut self) {
    self.rom_check_loading_frame = 0;
}

fn subscription(&self) -> Subscription<Message> {
    let mut subscriptions = Vec::new();

    if matches!(self.active_nav, NavPage::Dashboard | NavPage::Rom)
        && !self.busy
        && !self.ui_animation_active()
    {
        subscriptions.push(
            iced::time::every(DASHBOARD_REFRESH_INTERVAL)
                .map(|_| Message::DashboardRefreshTick),
        );
    }

    if matches!(self.active_nav, NavPage::Dashboard | NavPage::Rom)
        && !self.busy
        && self.dashboard_model_missing_since.is_some()
    {
        subscriptions.push(
            iced::time::every(DASHBOARD_MODEL_TIMEOUT_INTERVAL)
                .map(|_| Message::DashboardModelTimeoutTick),
        );
    }

    if self.active_nav == NavPage::Rom
        && self.busy
        && self.image_dir.is_some()
        && self.rom_firmware_info.is_none()
    {
        subscriptions.push(
            iced::time::every(ROM_CHECK_LOADING_INTERVAL)
                .map(|_| Message::RomCheckLoadingTick),
        );
    }

    if self.live_rx.is_some() {
        subscriptions.push(
            iced::time::every(LIVE_EVENT_DRAIN_INTERVAL).map(|_| Message::DrainLiveEvents),
        );
    }

    if !self.sidebar_anim_settled() {
        subscriptions.push(
            iced::time::every(SIDEBAR_ANIM_INTERVAL).map(|_| Message::SidebarAnimTick),
        );
    }

    let rom_slide_settled = self.rom_routine_slide_settled();

    if self.active_nav == NavPage::Rom && !rom_slide_settled {
        subscriptions.push(
            iced::time::every(ROM_SLIDE_ANIM_INTERVAL).map(|_| Message::RomSlideTick),
        );
    }

    subscriptions.push(iced::event::listen_with(|event, _, _| {
        if let iced::Event::Window(iced::window::Event::CloseRequested) = event {
            Some(Message::WindowCloseRequested)
        } else {
            None
        }
    }));

    Subscription::batch(subscriptions)
}

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
Message::Noop => Task::none(),

Message::DashboardRefreshTick => {
    if !matches!(self.active_nav, NavPage::Dashboard | NavPage::Rom)
        || self.busy
        || self.dashboard_refreshing
    {
        return Task::none();
    }

    self.dashboard_refreshing = true;

    Task::perform(load_dashboard_snapshot_worker(), Message::DashboardLoaded)
}

Message::DashboardLoaded(result) => {
    self.dashboard_refreshing = false;

    let was_device_unknown = lpm_is_unknown_text(&self.dashboard_info.product_device);

    match result {
        Ok(snapshot) => {
            let model_image = Self::dashboard_model_image_from_info(&snapshot.info);
            let next_info = snapshot.info;
            let battery_level = next_info.battery_level;

            if self.battery_progress_level != battery_level {
                self.battery_progress_level = battery_level;
                self.battery_progress_handle = build_battery_progress_handle(battery_level);
            }

            self.dashboard_info = next_info;

            if model_image == DashboardModelImage::Unknown {
                if self.dashboard_model_missing_since.is_none() {
                    self.dashboard_model_missing_since = Some(Instant::now());
                }
            } else {
                self.dashboard_model_image = model_image;
                self.dashboard_model_missing_since = None;
            }
        }

        Err(_err) => {
            if matches!(self.active_nav, NavPage::Dashboard | NavPage::Rom) {
                self.dashboard_info = lpmbox_device::DashboardDeviceInfo::default();

                if self.battery_progress_level != None {
                    self.battery_progress_level = None;
                    self.battery_progress_handle = build_battery_progress_handle(None);
                }
            }

            if self.dashboard_model_missing_since.is_none() {
                self.dashboard_model_missing_since = Some(Instant::now());
            }
        }
    }

let now_device_unknown = lpm_is_unknown_text(&self.dashboard_info.product_device);

self.return_to_rom_image_info_after_device_state_changed(
    was_device_unknown,
    now_device_unknown,
);

Task::none()
}

Message::DashboardModelTimeoutTick => {
    if let Some(missing_since) = self.dashboard_model_missing_since {
        if missing_since.elapsed() >= Duration::from_secs(1) {
            self.dashboard_model_image = DashboardModelImage::Unknown;
        }
    }

    Task::none()
}

Message::RomCardHoverEnter(target) => {
    let now = Instant::now();

    match target {
        RomSlideTarget::Install => {
            if self.rom_install_hover_since.is_none() {
                self.rom_install_hover_since = Some(now);
            }
            self.rom_update_hover_since = None;
            self.rom_reinstall_hover_since = None;
        }
        RomSlideTarget::Update => {
            if self.rom_update_hover_since.is_none() {
                self.rom_update_hover_since = Some(now);
            }
            self.rom_install_hover_since = None;
            self.rom_reinstall_hover_since = None;
        }
        RomSlideTarget::Reinstall => {
            if self.rom_reinstall_hover_since.is_none() {
                self.rom_reinstall_hover_since = Some(now);
            }
            self.rom_install_hover_since = None;
            self.rom_update_hover_since = None;
        }
    }

    Task::none()
}

Message::RomCardHoverExit(target) => {
    match target {
        RomSlideTarget::Install => {
            self.rom_install_hover_since = None;
        }
        RomSlideTarget::Update => {
            self.rom_update_hover_since = None;
        }
        RomSlideTarget::Reinstall => {
            self.rom_reinstall_hover_since = None;
        }
    }

    Task::none()
}

Message::RomCheckLoadingTick => {
    let frame_count = self.loading_progress_handles.len().max(1);

    if self.rom_check_loading_frame + 1 >= frame_count {
        self.rom_check_loading_frame = 0;
    } else {
        self.rom_check_loading_frame += 1;
    }

    Task::none()
}

Message::RomProceedToRoutine => {
    self.rom_show_routine_select = true;
    self.rom_option_target = None;
    self.running_rom_target = None;
    self.rom_country_popup_open = false;
    self.rom_country_code_modal_open = false;
    self.rom_country_code_search.clear();
    self.reset_rom_routine_slide_state();
    self.reset_rom_check_loading_stack();
    Task::none()
}

Message::RomBackToImageInfo => {
    self.rom_show_routine_select = false;
    self.rom_option_target = None;
    self.running_rom_target = None;
    self.rom_country_popup_open = false;
    self.rom_country_code_modal_open = false;
    self.rom_country_code_search.clear();
    self.reset_rom_routine_slide_state();
    self.reset_rom_check_loading_stack();
    Task::none()
}

Message::RomOpenOptions(target) => {
    if self.busy {
        self.push_log("이미 작업이 진행 중입니다.");
        return Task::none();
    }

    self.reset_rom_routine_slide_state();
    self.reset_rom_check_loading_stack();
    self.rom_option_target = Some(target);
    self.rom_country_popup_open = false;
    self.rom_country_code_search.clear();
    self.apply_rom_option_wipe_rule(target);
    Task::none()
}

Message::RomBackToRoutineSelect => {
    self.rom_option_target = None;
    self.rom_country_popup_open = false;
    self.rom_country_code_modal_open = false;
    self.rom_country_code_search.clear();
    self.reset_rom_routine_slide_state();
    self.reset_rom_check_loading_stack();
    Task::none()
}

Message::RomToggleDataWipe => {
    if !self.rom_option_data_wipe_locked {
        self.rom_option_data_wipe = !self.rom_option_data_wipe;
    }

    Task::none()
}

Message::RomOpenCountryCodeModal => {
    if self.busy {
        self.push_log("이미 작업이 진행 중입니다.");
        return Task::none();
    }

    if self.rom_country_code_locked_for_current_option() {
        return Task::none();
    }

    self.rom_country_popup_open = true;
    self.rom_country_code_modal_open = true;
    self.rom_country_code_search.clear();
    Task::none()
}

Message::RomCountrySelect => {
    if self.busy {
        self.push_log("이미 작업이 진행 중입니다.");
        return Task::none();
    }

    if self.rom_country_code_locked_for_current_option() {
        return Task::none();
    }

    self.rom_country_popup_open = true;
    self.rom_country_code_search.clear();
    Task::none()
}

Message::RomSelectCountry(code) => {
    if self.busy {
        self.push_log("이미 작업이 진행 중입니다.");
        return Task::none();
    }

    if self.rom_country_code_locked_for_current_option() {
        return Task::none();
    }

    let start_additional_country_reset = self.additional_country_reset_pending;

    self.rom_option_country_code = Some(code.clone());
    self.rom_country_popup_open = false;
    self.rom_country_code_modal_open = false;
    self.rom_country_code_search.clear();
    self.additional_country_reset_pending = false;

    if start_additional_country_reset {
        self.push_log(format!("[추가 옵션] 국가 코드를 선택했습니다: {code}"));
        return self.start_country_reset_flow(code);
    }

    self.push_log(format!("[ROM 옵션] 국가 코드를 선택했습니다: {code}"));
    Task::none()
}

Message::RomCloseCountryPopup => {
    self.rom_country_popup_open = false;
    self.rom_country_code_search.clear();
    self.additional_country_reset_pending = false;
    Task::none()
}

Message::RomCloseCountryCodeModal => {
    self.rom_country_popup_open = false;
    self.rom_country_code_modal_open = false;
    self.rom_country_code_search.clear();
    self.additional_country_reset_pending = false;
    Task::none()
}

Message::RomSelectCountryCode(code) => {
    if self.busy {
        self.push_log("이미 작업이 진행 중입니다.");
        return Task::none();
    }

    if self.rom_country_code_locked_for_current_option() {
        return Task::none();
    }

    let start_additional_country_reset = self.additional_country_reset_pending;

    self.rom_option_country_code = Some(code.to_string());
    self.rom_selected_country_code = Some(code);
    self.rom_country_popup_open = false;
    self.rom_country_code_modal_open = false;
    self.rom_country_code_search.clear();
    self.additional_country_reset_pending = false;

    if start_additional_country_reset {
        self.push_log(format!("[추가 옵션] 국가 코드를 선택했습니다: {code}"));
        return self.start_country_reset_flow(code.to_string());
    }

    self.push_log(format!("[ROM 옵션] 국가 코드를 선택했습니다: {code}"));
    Task::none()
}

Message::RomCountryCodeSearchChanged(value) => {
    self.rom_country_code_search = value;
    Task::none()
}

Message::RomContinueFromOptions => {
    if self.busy {
        self.push_log("이미 작업이 진행 중입니다.");
        return Task::none();
    }

    self.active_nav = NavPage::Log;

    match self.rom_option_target {
        Some(RomSlideTarget::Install) => self.update(Message::StartConvertWipe),
        Some(RomSlideTarget::Update) => self.update(Message::StartRowUpdateKeepData),
        Some(RomSlideTarget::Reinstall) => self.update(Message::StartReinstallWipe),
        None => Task::none(),
    }
}

Message::RomSlideTick => {
    let (install_target_width, update_target_width, reinstall_target_width) =
        self.rom_routine_slide_target_widths();

    if self.rom_routine_slide_settled() {
        self.rom_install_slide_width = install_target_width;
        self.rom_update_slide_width = update_target_width;
        self.rom_reinstall_slide_width = reinstall_target_width;
        self.rom_install_slide_velocity = 0.0;
        self.rom_update_slide_velocity = 0.0;
        self.rom_reinstall_slide_velocity = 0.0;
        return Task::none();
    }

    self.rom_install_slide_width = Self::animate_rom_slide_width(
        self.rom_install_slide_width,
        &mut self.rom_install_slide_velocity,
        install_target_width,
    );

    self.rom_update_slide_width = Self::animate_rom_slide_width(
        self.rom_update_slide_width,
        &mut self.rom_update_slide_velocity,
        update_target_width,
    );

    self.rom_reinstall_slide_width = Self::animate_rom_slide_width(
        self.rom_reinstall_slide_width,
        &mut self.rom_reinstall_slide_velocity,
        reinstall_target_width,
    );

    Task::none()
}

Message::SelectNav(page) => {
    if page == NavPage::FirmwareDownload {
        return self.update(Message::OpenFirmwareDownload);
    }

    if page == NavPage::Qna {
        return self.update(Message::OpenQna);
    }

    if self.active_nav != page {
        self.reset_rom_check_loading_stack();
        self.additional_country_reset_pending = false;
    }

    self.active_nav = page;
    Task::none()
}

Message::AdditionalOpenCountryReset => {
    if self.busy {
        self.push_log("이미 작업이 진행 중입니다.");
        return Task::none();
    }

    self.active_nav = NavPage::Driver;
    self.rom_option_target = None;
    self.rom_option_country_code = None;
    self.rom_selected_country_code = None;
    self.rom_country_popup_open = true;
    self.rom_country_code_modal_open = true;
    self.rom_country_code_search.clear();
    self.additional_country_reset_pending = true;
    self.push_log("[추가 옵션] 국가 코드 재설정 국가 코드를 선택합니다.");
    Task::none()
}

Message::StartOtaDisable => {
    if self.busy {
        self.push_log("이미 작업이 진행 중입니다.");
        return Task::none();
    }

    self.active_nav = NavPage::Log;
    self.busy = true;
    self.progress_line_indices.clear();
    self.active_spinners.clear();
    self.spinner_tick = 0;
    self.last_spft_stage = None;
    self.active_log_flow = Some(RuntimeFlowKind::OtaDisable);
    self.push_log("[OTA] OTA(업데이트) 비활성화 작업을 시작합니다.");

    let (tx, rx) = mpsc::channel::<ProinfoLiveEvent>();
    self.live_rx = Some(rx);

    thread::spawn(move || {
        run_ota_disable_flow(tx);
    });

    Task::none()
}

Message::StartOtaEnable => {
    if self.busy {
        self.push_log("이미 작업이 진행 중입니다.");
        return Task::none();
    }

    self.active_nav = NavPage::Log;
    self.busy = true;
    self.progress_line_indices.clear();
    self.active_spinners.clear();
    self.spinner_tick = 0;
    self.last_spft_stage = None;
    self.active_log_flow = Some(RuntimeFlowKind::OtaEnable);
    self.push_log("[OTA] OTA(업데이트) 활성화 작업을 시작합니다.");

    let (tx, rx) = mpsc::channel::<ProinfoLiveEvent>();
    self.live_rx = Some(rx);

    thread::spawn(move || {
        run_ota_enable_flow(tx);
    });

    Task::none()
}

Message::DashboardOpenRomFolderSelect => {
    if self.busy {
        self.push_log("이미 작업이 진행 중입니다.");
        return Task::none();
    }

    self.active_nav = NavPage::Rom;
    self.image_dir = None;
    self.rom_firmware_info = None;
    self.rom_firmware_error = None;
    self.rom_mtk_driver_installed = None;
    self.rom_mtk_driver_error = None;
    self.rom_vc_runtime_status = None;
    self.rom_vc_runtime_error = None;
    self.rom_show_routine_select = false;
    self.rom_option_target = None;
    self.running_rom_target = None;
    self.active_log_flow = None;
    self.rom_option_country_code = None;
    self.rom_country_popup_open = false;
    self.rom_country_code_modal_open = false;
    self.rom_country_code_search.clear();
    self.reset_rom_routine_slide_state();
    self.reset_rom_check_loading_stack();
    Task::none()
}

Message::OpenFirmwareDownload => {
    if let Err(err) = open::that(
        "https://drive.google.com/drive/folders/1wTTNoS0H97fbvWI8wdRiERpo25q_712p?usp=sharing",
    ) {
        self.push_log(format!("[프로그램] 펌웨어 다운로드 링크 열기 실패: {err}"));
    }

    Task::none()
}

Message::OpenQna => {
    if let Err(err) = open::that(
        "https://dwas.tistory.com/category/%ED%94%84%EB%A1%9C%EA%B7%B8%EB%9E%A8/LPMBOX%20%2812.7%2C%2012.1%2C%2011%29",
    ) {
        self.push_log(format!("[프로그램] QnA 링크 열기 실패: {err}"));
    }

    Task::none()
}

Message::OpenDeveloperYoutube => {
    if let Err(err) = open::that("https://www.youtube.com/@dwas_KR?sub_confirmation=1") {
        self.push_log(format!("[설정] 개발자 유튜브 링크 열기 실패: {err}"));
    }

    Task::none()
}

Message::OpenProgramVideos => {
    if let Err(err) = open::that("https://www.youtube.com/@dwas_KR/videos") {
        self.push_log(format!("[프로그램] 더 많은 프로그램 링크 열기 실패: {err}"));
    }

    Task::none()
}

Message::OpenDonate => {
    if let Err(err) = open::that("https://www.youtube.com/channel/UCe3U-W3fIYyIi4SZGK4zI8Q/join") {
        self.push_log(format!("[설정] 후원하기 링크 열기 실패: {err}"));
    }

    Task::none()
}

Message::OpenFeedback => {
    let feedback_url = if self.settings_language.is_korean() {
        "https://github.com/dwas-KR/LPMBox/issues/9"
    } else {
        "https://github.com/dwas-KR/LPMBox/issues/3"
    };

    if let Err(err) = open::that(feedback_url) {
        self.push_log(format!("[설정] 피드백 링크 열기 실패: {err}"));
    }

    Task::none()
}

Message::CheckProgramUpdate => {
    if self.program_update_checking {
        self.push_log("[Update] 이미 최신 릴리즈 확인이 진행 중입니다.");
        return Task::none();
    }

    self.program_update_checking = true;
    self.push_log("[Update] 최신 LPMBox 릴리즈를 확인합니다.");

    Task::perform(
        check_program_update_worker(APP_DISPLAY_VERSION.to_string()),
        Message::ProgramUpdateChecked,
    )
}

Message::ProgramUpdateChecked(result) => {
    self.program_update_checking = false;

    match result {
        Ok(info) => {
            if info.update_available {
                self.push_log(format!(
                    "[Update] 새 LPMBox 버전을 찾았습니다: 현재 {} → 최신 {}",
                    info.current_version, info.latest_version
                ));

                if let Some(asset_name) = &info.asset_name {
                    self.push_log(format!("[Update] 릴리즈 ZIP 파일: {asset_name}"));
                }

                self.push_log("[Update] 대시보드에 업데이트 안내 창을 표시합니다.");
                self.dashboard_update_notice = Some(info);
                self.active_nav = NavPage::Dashboard;
            } else {
                self.push_log(format!(
                    "[Update] 이미 최신 버전을 사용 중입니다: LPMBox {}",
                    info.current_version
                ));
            }
        }
        Err(err) => {
            self.push_log(format!("[Update] 업데이트 확인 실패: {err}"));
            self.push_log("[Update] 수동 확인을 위해 GitHub Releases 페이지를 엽니다.");

            if let Err(open_err) = open::that(LPMBOX_RELEASES_URL) {
                self.push_log(format!("[Update] GitHub Releases 페이지 열기 실패: {open_err}"));
            }
        }
    }

    Task::none()
}

Message::DashboardProgramUpdateChecked(result) => {
    if let Ok(info) = result {
        if info.update_available {
            self.dashboard_update_notice = Some(info);

            if self.active_nav == NavPage::Dashboard {
                self.push_log("[Update] 새로운 업데이트 파일을 감지했습니다.");
            }
        }
    }

    Task::none()
}

Message::OpenProgramUpdateRelease => {
    let release_url = self
        .dashboard_update_notice
        .as_ref()
        .map(|info| info.release_url.clone())
        .unwrap_or_else(|| LPMBOX_RELEASES_URL.to_string());

    self.dashboard_update_notice = None;

    if let Err(err) = open::that(&release_url) {
        self.push_log(format!("[Update] GitHub Releases 페이지 열기 실패: {err}"));
    }

    Task::none()
}

Message::DismissProgramUpdateNotice => {
    self.dashboard_update_notice = None;
    self.push_log("[Update] 이번 업데이트 안내를 다음에 다시 확인합니다.");
    Task::none()
}

Message::SettingsLanguageSelected(language) => {
    self.settings_language = language;
    set_active_language_option(language);
    save_language_option(language);
    self.refresh_log_content();
    self.push_log(format!("[설정] 프로그램 언어를 선택했습니다: {language}"));
    self.push_log(format!(
        "[설정] 언어 설정 파일에 저장했습니다: {}",
        lpm_language_config_path().display()
    ));
    Task::none()
}

Message::WindowCloseRequested => {
    self.cleanup_before_process_exit();
    Task::none()
}

Message::SidebarHoverEnter => {
    self.sidebar_expanded = true;
    Task::none()
}

Message::SidebarHoverExit => {
    self.sidebar_expanded = false;
    Task::none()
}

Message::SidebarAnimTick => {
    let target = self.sidebar_anim_target();
    self.sidebar_anim = lpm_approach_anim_value(
        self.sidebar_anim,
        target,
        SIDEBAR_ANIM_FACTOR,
        SIDEBAR_ANIM_THRESHOLD,
    )
    .clamp(0.0, 1.0);
    self.sidebar_velocity = 0.0;

    Task::none()
}

            Message::SelectImageFolder => {
                if self.busy {
                    self.push_log("이미 작업이 진행 중입니다.");
                    return Task::none();
                }

                let dialog_title = lpm_translate_owned("LPMBox image 폴더 선택".to_string());
                let Some(folder) = rfd::FileDialog::new()
                    .set_title(&dialog_title)
                    .pick_folder()
                else {
                    self.push_log("image 폴더 선택이 취소되었습니다.");
                    return Task::none();
                };

                self.image_dir = Some(folder.clone());
                self.rom_firmware_info = None;
                self.rom_firmware_error = None;
                self.rom_mtk_driver_installed = None;
                self.rom_mtk_driver_error = None;
                self.rom_vc_runtime_status = None;
                self.rom_vc_runtime_error = None;
                self.rom_show_routine_select = false;
                self.rom_option_target = None;
    self.running_rom_target = None;
                self.rom_option_data_wipe = true;
                self.rom_option_data_wipe_locked = true;
                self.rom_option_country_code = None;
                self.rom_country_popup_open = false;
                self.rom_country_code_modal_open = false;
                self.rom_country_code_search.clear();
                self.reset_rom_routine_slide_state();
                self.reset_rom_check_loading_stack();

                self.busy = true;

                self.push_log(format!("image 폴더 선택됨: {}", folder.display()));
                self.push_log("펌웨어 정보 및 설치 환경 검사를 시작합니다.");

                Task::perform(check_firmware_worker(folder), Message::FirmwareChecked)
            }

            Message::CheckFirmware => {
                let Some(image_dir) = self.image_dir.clone() else {
                    self.push_log("먼저 image 폴더를 선택해주세요.");
                    return Task::none();
                };

                if self.busy {
                    self.push_log("이미 작업이 진행 중입니다.");
                    return Task::none();
                }

                self.busy = true;
                self.rom_firmware_info = None;
                self.rom_firmware_error = None;
                self.rom_mtk_driver_installed = None;
                self.rom_mtk_driver_error = None;
                self.rom_vc_runtime_status = None;
                self.rom_vc_runtime_error = None;
                self.rom_show_routine_select = false;
                self.reset_rom_check_loading_stack();

                self.push_log("펌웨어 정보 및 설치 환경 검사를 시작합니다.");

                Task::perform(check_firmware_worker(image_dir), Message::FirmwareChecked)
            }

            Message::FirmwareChecked(result) => {
                self.busy = false;
                self.reset_rom_check_loading_stack();

                match result {
                    Ok(check) => {
                        self.rom_mtk_driver_installed = check.mtk_driver_installed;
                        self.rom_mtk_driver_error = check.mtk_driver_error;
                        self.rom_vc_runtime_status = check.vc_runtime_status;
                        self.rom_vc_runtime_error = check.vc_runtime_error;

                        let info = check.firmware;

                        self.push_firmware_result_logs(&info);
                        self.rom_firmware_info = Some(info);
                        self.rom_firmware_error = None;
                        self.rom_show_routine_select = false;

                        if self.rom_mtk_driver_installed == Some(false) {
                            self.start_mtk_driver_package_prepare_if_needed();
                        }
                    }

                    Err(err) => {
                        self.push_log(format!("오류: {err}"));
                        self.rom_firmware_info = None;
                        self.rom_firmware_error = Some(err);
                    }
                }

                Task::none()
            }

            Message::StartConvertWipe => {
                let Some(image_dir) = self.image_dir.clone() else {
                    self.push_log("먼저 image 폴더를 선택해주세요.");
                    return Task::none();
                };

                if self.busy {
                    self.push_log("이미 작업이 진행 중입니다.");
                    return Task::none();
                }

                self.active_nav = NavPage::Log;
                self.running_rom_target = Some(RomSlideTarget::Install);
                self.busy = true;
                self.progress_line_indices.clear();
                self.active_spinners.clear();
                self.spinner_tick = 0;
                self.last_spft_stage = None;

                self.active_log_flow = Some(RuntimeFlowKind::PrcRowInstall);
                let routine_version_log = lpm_routine_version_log_text(
                    active_language_option(),
                    self.rom_firmware_info.as_ref(),
                );
                self.push_log(routine_version_log);
                self.push_log("1번 옵션을 시작합니다: PRC/ROW 펌웨어 설치 [데이터 초기화]");

let selected_country_code = self.rom_option_country_code.clone();

let (tx, rx) = mpsc::channel::<ProinfoLiveEvent>();
self.live_rx = Some(rx);

thread::spawn(move || {
    run_convert_wipe_flow(image_dir, selected_country_code, tx);
});

                Task::none()
            }

    Message::StartRowUpdateKeepData => {
        let Some(image_dir) = self.image_dir.clone() else {
            self.push_log("먼저 image 폴더를 선택해주세요.");
            return Task::none();
        };

        if self.busy {
            self.push_log("이미 작업이 진행 중입니다.");
            return Task::none();
        }

        self.active_nav = NavPage::Log;
        self.running_rom_target = Some(RomSlideTarget::Update);
        self.busy = true;
        self.progress_line_indices.clear();
        self.active_spinners.clear();
        self.spinner_tick = 0;
        self.last_spft_stage = None;

        self.active_log_flow = Some(RuntimeFlowKind::RowUpdate);
        let routine_version_log = lpm_routine_version_log_text(
            active_language_option(),
            self.rom_firmware_info.as_ref(),
        );
        self.push_log(routine_version_log);
        self.push_log("2번 옵션을 시작합니다: ROW(글로벌) 펌웨어 업데이트 [데이터 유지]");

let selected_country_code = self.rom_option_country_code.clone();

let (tx, rx) = mpsc::channel::<ProinfoLiveEvent>();
self.live_rx = Some(rx);

thread::spawn(move || {
    run_row_update_keep_data_flow(image_dir, selected_country_code, tx);
});

        Task::none()
    }

Message::StartReinstallWipe => {
    let Some(image_dir) = self.image_dir.clone() else {
        self.push_log("먼저 image 폴더를 선택해주세요.");
        return Task::none();
    };

    if self.busy {
        self.push_log("이미 작업이 진행 중입니다.");
        return Task::none();
    }

    self.active_nav = NavPage::Log;
    self.running_rom_target = Some(RomSlideTarget::Reinstall);
    self.busy = true;
    self.progress_line_indices.clear();
    self.active_spinners.clear();
    self.spinner_tick = 0;
    self.last_spft_stage = None;

    self.active_log_flow = Some(RuntimeFlowKind::DeviceRecovery);
    let routine_version_log = lpm_routine_version_log_text(
        active_language_option(),
        self.rom_firmware_info.as_ref(),
    );
    self.push_log(routine_version_log);
    self.push_log("3번 옵션을 시작합니다: 기기 복구 [데이터 초기화]");

    let (tx, rx) = mpsc::channel::<ProinfoLiveEvent>();
    self.live_rx = Some(rx);

    thread::spawn(move || {
        run_reinstall_wipe_flow(image_dir, tx);
    });

    Task::none()
}

            Message::BackupProinfo => {
                let Some(image_dir) = self.image_dir.clone() else {
                    self.push_log("먼저 image 폴더를 선택해주세요.");
                    return Task::none();
                };

                if self.busy {
                    self.push_log("이미 작업이 진행 중입니다.");
                    return Task::none();
                }

                self.busy = true;
                self.progress_line_indices.clear();
                self.active_spinners.clear();
                self.spinner_tick = 0;
                self.last_spft_stage = None;

                self.push_log("proinfo 백업을 시작합니다.");

                let (tx, rx) = mpsc::channel::<ProinfoLiveEvent>();
                self.live_rx = Some(rx);

                thread::spawn(move || {
                    run_proinfo_backup_flow(image_dir, tx);
                });

                Task::none()
            }

            Message::InstallMtkDriver => {
                if self.busy {
                    self.push_log("이미 작업이 진행 중입니다.");
                    return Task::none();
                }

                self.busy = true;
                self.progress_line_indices.clear();
                self.active_spinners.clear();
                self.spinner_tick = 0;
                self.last_spft_stage = None;

                self.push_log("[Driver] MTK 드라이버 설치를 시작합니다.");

                let (tx, rx) = mpsc::channel::<ProinfoLiveEvent>();
                self.live_rx = Some(rx);

                thread::spawn(move || {
                    run_mtk_driver_install_flow(tx);
                });

                Task::none()
            }

            Message::InstallVcRuntime => {
                if self.busy {
                    self.push_log("이미 작업이 진행 중입니다.");
                    return Task::none();
                }

                self.busy = true;
                self.progress_line_indices.clear();
                self.active_spinners.clear();
                self.spinner_tick = 0;
                self.last_spft_stage = None;

                self.push_log("[Runtime] Microsoft Visual C++ x86/x64 런타임 설치를 시작합니다.");

                let (tx, rx) = mpsc::channel::<ProinfoLiveEvent>();
                self.live_rx = Some(rx);

                thread::spawn(move || {
                    run_vc_runtime_install_flow(tx);
                });

                Task::none()
            }

            Message::DrainLiveEvents => {
                self.log_refresh_suspended = true;
                self.tick_active_spinners();

                let mut events = Vec::new();
                let mut disconnected = false;

                if let Some(rx) = &self.live_rx {
                    loop {
                        match rx.try_recv() {
                            Ok(event) => events.push(event),
                            Err(TryRecvError::Empty) => break,
                            Err(TryRecvError::Disconnected) => {
                                disconnected = true;
                                break;
                            }
                        }
                    }
                }

                let mut finished = false;

                for event in events {
                    if self.handle_live_event(event) {
                        finished = true;
                    }
                }

                self.log_refresh_suspended = false;
                self.flush_log_content_if_needed();

                if finished || disconnected {
                    let completed_rom_target = self.running_rom_target.take();
                    let completed_log_flow = self.active_log_flow.take();

                    if disconnected && !finished && completed_log_flow.is_some() {
                        self.push_log("[Error] 작업이 비정상적으로 종료되었습니다.");
                        self.push_common_failure_guidance();
                    }

                    if let Some(flow) = completed_log_flow {
                        if let Err(err) = self.save_completed_flow_log_to_file(flow) {
                            self.push_log(format!("[Log] 작업 로그 자동 저장 실패: {err}"));
                        }
                    }

                    self.busy = false;
                    self.live_rx = None;
                    self.progress_line_indices.clear();
                    self.active_spinners.clear();
                    self.spinner_tick = 0;
                    self.last_spft_stage = None;
                    self.reset_rom_check_loading_stack();

                    if completed_rom_target.is_some() {
                        self.active_nav = NavPage::Rom;
                        self.rom_show_routine_select = false;
                        self.rom_option_target = None;
                        self.rom_country_popup_open = false;
                        self.rom_country_code_modal_open = false;
                        self.rom_country_code_search.clear();
                        self.reset_rom_routine_slide_state();
                    }
                }

                Task::none()
            }

            Message::ExportLog => {
                match self.export_log_to_file() {
                    Ok(_) => {}
                    Err(err) => self.push_log(format!("[Log] 텍스트 파일 저장 실패: {err}")),
                }

                Task::none()
            }

            Message::ClearLog => {
                self.log_lines.clear();
                self.progress_line_indices.clear();
                self.active_spinners.clear();
                self.spinner_tick = 0;
                self.last_spft_stage = None;
                self.refresh_log_content();
                Task::none()
            }
        }
    }

fn view(&self) -> Element<'_, Message> {
    let status = if self.busy { lpm_translate_owned("작업 중".to_string()) } else { lpm_translate_owned("대기 중".to_string()) };

    let image_dir_text = self
        .image_dir
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "선택된 image 폴더 없음".to_string());

    let status_badge = if self.busy {
        container(text(status.clone()).size(12))
            .padding([6.0, 12.0])
            .style(lpm_nav_status_busy_style)
    } else {
        container(text(status.clone()).size(12))
            .padding([6.0, 12.0])
            .style(lpm_nav_status_idle_style)
    };

    let header_right: Element<'_, Message> =
        if self.active_nav == NavPage::Rom
            || self.active_nav == NavPage::Driver
            || self.active_nav == NavPage::Settings
        {
            iced::widget::Space::new()
                .width(Length::Fixed(1.0))
                .height(Length::Fixed(1.0))
                .into()
        } else {
            status_badge.into()
        };

    let page_title = if self.active_nav == NavPage::Rom && self.rom_option_target.is_some() {
        "옵션 선택"
    } else if self.active_nav == NavPage::Rom && self.rom_show_routine_select {
        "작업 선택"
    } else if self.active_nav == NavPage::Rom && self.image_dir.is_some() {
        "선택한 image 폴더 정보"
    } else {
        nav_page_title(self.active_nav)
    };

    let page_subtitle = if self.active_nav == NavPage::Rom && self.rom_option_target.is_some() {
        "ROM 작업을 진행하기 전 세부 옵션을 설정합니다."
    } else if self.active_nav == NavPage::Rom && self.rom_show_routine_select {
        "아래 작업을 선택하여 기기에 적용합니다."
    } else if self.active_nav == NavPage::Rom && self.image_dir.is_some() {
        "Image 폴더 정보를 확인합니다."
    } else {
        nav_page_subtitle(self.active_nav)
    };

let header = container(
    row![
        column![
            text(page_title).size(24),
            text(page_subtitle).size(12),
        ]
        .spacing(3)
        .width(Length::Fill),
        header_right,
    ]
    .spacing(12),
)
.width(Length::Fill)
.padding([14.0, 16.0])
.style(lpm_nav_header_style);

    let image_panel = container(
        column![
            row![
                column![
                    text("현재 image 폴더").size(13),
                    text(compact_text(&image_dir_text, 64)).size(BODY_FONT),
                ]
                .spacing(3)
                .width(Length::Fill),
                button(text("폴더 선택").size(BODY_FONT)).on_press(Message::SelectImageFolder),
            ]
            .spacing(10),
            text("flash.xml / scatter / DA 파일이 포함된 image 폴더를 선택해주세요.").size(12),
        ]
        .spacing(8),
    )
    .width(Length::Fill)
    .padding(14)
    .style(lpm_nav_panel_style);

    let rom_panel: Element<'_, Message> = if self.active_nav == NavPage::Rom {
        self.rom_image_folder_select_panel()
    } else {
        iced::widget::Space::new()
            .width(Length::Fixed(1.0))
            .height(Length::Fixed(1.0))
            .into()
    };

    let backup_panel = container(
        column![
            text("proinfo 백업").size(18),
            text("SPFlashToolV6 readback으로 proinfo 파티션을 추출합니다.").size(12),
            button(text("proinfo 백업 시작").size(BODY_FONT)).on_press(Message::BackupProinfo),
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .padding(14)
    .style(lpm_nav_panel_style);

    let ota_buttons = container(
        row![
            button(
                container(
                    text("활성화")
                        .size(12)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
            )
            .width(Length::Fixed(120.0))
            .height(Length::Fixed(38.0))
            .padding(0.0)
            .style(lpm_nav_additional_option_button_style)
            .on_press(Message::StartOtaEnable),
            button(
                container(
                    text("비활성화")
                        .size(12)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
            )
            .width(Length::Fixed(120.0))
            .height(Length::Fixed(38.0))
            .padding(0.0)
            .style(lpm_nav_additional_option_button_style)
            .on_press(Message::StartOtaDisable),
        ]
        .spacing(14)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center);

    let ota_section = column![
        text("OTA(업데이트)")
            .size(20)
            .font(lpm_bold_font())
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .wrapping(iced::widget::text::Wrapping::Word),
        text("업데이트 기능을 활성화 또는 비활성화로 설정합니다.")
            .size(12)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .wrapping(iced::widget::text::Wrapping::Word),
        ota_buttons,
    ]
    .spacing(7)
    .width(Length::Fixed(300.0))
    .align_x(iced::Alignment::Center);

    let country_reset_button = container(
        button(
            container(
                text("재설정")
                    .size(12)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center)
                    .align_y(iced::alignment::Vertical::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
        )
        .width(Length::Fixed(120.0))
        .height(Length::Fixed(38.0))
        .padding(0.0)
        .style(lpm_nav_additional_option_button_style)
        .on_press(Message::AdditionalOpenCountryReset),
    )
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center);

    let country_reset_section = column![
        text("국가 코드 재설정")
            .size(20)
            .font(lpm_bold_font())
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .wrapping(iced::widget::text::Wrapping::Word),
        text("기기에 설정된 국가 코드를 변경합니다.")
            .size(12)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .wrapping(iced::widget::text::Wrapping::Word),
        country_reset_button,
    ]
    .spacing(7)
    .width(Length::Fixed(300.0))
    .align_x(iced::Alignment::Center);

    let driver_panel = container(
        container(
            row![ota_section, country_reset_section]
                .spacing(0)
                .align_y(iced::Alignment::Center),
        )
        .width(Length::Fixed(650.0))
        .height(Length::Fixed(190.0))
        .padding([22.0, 14.0])
        .style(lpm_nav_extra_option_card_style),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding([24.0, 0.0])
    .align_x(iced::alignment::Horizontal::Center);

let mut log_rows = column![]
    .spacing(0)
    .width(Length::Fill);

if matches!(self.active_nav, NavPage::Log | NavPage::Backup) {
    for log_row_text in &self.log_display_rows_cache {
        log_rows = log_rows.push(
            iced_text(log_row_text.as_str())
                .size(LOG_FONT)
                .width(Length::Fill)
                .wrapping(iced::widget::text::Wrapping::Word),
        );
    }
}

let log_content = container(log_rows)
    .width(Length::Fill)
    .padding(iced::Padding {
        top: 12.0,
        right: 20.0,
        bottom: 12.0,
        left: 12.0,
    });

    let log_panel = container(
        scrollable(log_content)
            .anchor_bottom()
            .anchor_left()
            .auto_scroll(true)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(0)
    .style(log_container_style);

    let log_section = container(
        column![
            row![
                text("작업 로그").size(18),
                text("ADB / Fastboot / PreLoader / SPFlashToolV6 진행 상태").size(12),
            ]
            .spacing(10),
            row![
                button(text("로그 내보내기").size(BODY_FONT)).on_press(Message::ExportLog),
                button(text("로그 지우기").size(BODY_FONT)).on_press(Message::ClearLog),
            ]
            .spacing(8),
            log_panel,
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(14)
    .style(lpm_nav_panel_style);

    let language_picker = pick_list(
        LANGUAGE_OPTIONS,
        Some(self.settings_language),
        Message::SettingsLanguageSelected,
    )
    .text_size(12)
    .style(lpm_nav_language_pick_list_style)
    .menu_style(lpm_nav_language_pick_list_menu_style)
    .width(Length::Fixed(170.0));

    let settings_panel = container(
        column![
            container(
                row![
                    column![
                        text("언어 변경")
                            .size(22)
                            .font(lpm_bold_font()),
                        text("프로그램 언어를 변경합니다.")
                            .size(12),
                    ]
                    .spacing(6)
                    .width(Length::Fill),
                    language_picker,
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .padding([4.0, 0.0]),
            container(
                row![
                    column![
                        text("개발자 유튜브")
                            .size(22)
                            .font(lpm_bold_font()),
                        text("샤오신패드에 유용한 프로그램을 확인하실 수 있습니다.")
                            .size(12),
                    ]
                    .spacing(6)
                    .width(Length::Fill),
                    button(
                        container(text("이동").size(12))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(iced::alignment::Horizontal::Center)
                            .align_y(iced::alignment::Vertical::Center),
                    )
                    .width(Length::Fixed(100.0))
                    .height(Length::Fixed(28.0))
                    .padding(0.0)
                    .style(lpm_nav_settings_move_button_style)
                    .on_press(Message::OpenDeveloperYoutube),
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .padding([4.0, 0.0]),
            container(
                row![
                    column![
                        text("후원하기")
                            .size(22)
                            .font(lpm_bold_font()),
                        text("개발자에게 큰 힘과 응원이 됩니다.")
                            .size(12),
                    ]
                    .spacing(6)
                    .width(Length::Fill),
                    button(
                        container(text("이동").size(12))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(iced::alignment::Horizontal::Center)
                            .align_y(iced::alignment::Vertical::Center),
                    )
                    .width(Length::Fixed(100.0))
                    .height(Length::Fixed(28.0))
                    .padding(0.0)
                    .style(lpm_nav_settings_move_button_style)
                    .on_press(Message::OpenDonate),
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .padding([4.0, 0.0]),
            container(
                row![
                    column![
                        text("프로그램 업데이트")
                            .size(22)
                            .font(lpm_bold_font()),
                        text("LPMBox 최신 릴리즈 버전을 확인합니다.")
                            .size(12),
                    ]
                    .spacing(6)
                    .width(Length::Fill),
                    if self.program_update_checking {
                        button(
                            container(text("확인 중").size(12))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .align_x(iced::alignment::Horizontal::Center)
                                .align_y(iced::alignment::Vertical::Center),
                        )
                        .width(Length::Fixed(100.0))
                        .height(Length::Fixed(28.0))
                        .padding(0.0)
                        .style(lpm_nav_settings_move_button_style)
                    } else {
                        button(
                            container(text("확인").size(12))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .align_x(iced::alignment::Horizontal::Center)
                                .align_y(iced::alignment::Vertical::Center),
                        )
                        .width(Length::Fixed(100.0))
                        .height(Length::Fixed(28.0))
                        .padding(0.0)
                        .style(lpm_nav_settings_move_button_style)
                        .on_press(Message::CheckProgramUpdate)
                    },
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .padding([4.0, 0.0]),
            container(
                row![
                    column![
                        text("피드백")
                            .size(22)
                            .font(lpm_bold_font()),
                        text("의견을 주시면 프로그램이 완벽해질 수 있습니다.")
                            .size(12),
                    ]
                    .spacing(6)
                    .width(Length::Fill),
                    button(
                        container(text("이동").size(12))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(iced::alignment::Horizontal::Center)
                            .align_y(iced::alignment::Vertical::Center),
                    )
                    .width(Length::Fixed(100.0))
                    .height(Length::Fixed(28.0))
                    .padding(0.0)
                    .style(lpm_nav_settings_move_button_style)
                    .on_press(Message::OpenFeedback),
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .padding([4.0, 0.0]),
        ]
        .spacing(24),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding([18.0, 16.0])
    .style(lpm_nav_settings_panel_style);

    let main_stack = match self.active_nav {
        NavPage::Dashboard => column![self.dashboard_panel()],
        NavPage::Rom => column![header, rom_panel],
        NavPage::Backup => column![header, image_panel, backup_panel, log_section],
        NavPage::Driver => column![header, driver_panel],
        NavPage::FirmwareDownload => column![header, settings_panel],
        NavPage::Qna => column![header, settings_panel],
        NavPage::Log => column![header, log_section],
        NavPage::Settings => column![header, settings_panel],
    }
    .spacing(10)
    .width(Length::Fill)
    .height(Length::Fill);

    let footer = container(
        row![
            iced_text(format!("● {status}")).size(12).width(Length::Fill),
            iced_text(format!("v{APP_DISPLAY_VERSION}")).size(12).width(Length::Fixed(60.0)).align_x(iced::alignment::Horizontal::Right),
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .padding([6.0, 12.0])
    .style(lpm_nav_footer_style);

    let main_content = container(column![
        container(main_stack)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .style(lpm_nav_app_background_style),
        footer,
    ])
    .width(Length::Fill)
    .height(Length::Fill);

    let rail_placeholder = container(iced::widget::Space::new())
        .width(Length::Fixed(SIDEBAR_RAIL_WIDTH))
        .height(Length::Fill);

    let row_base = row![rail_placeholder, main_content].height(Length::Fill);

let mut layers: Vec<Element<'_, Message>> = vec![row_base.into(), self.sidebar()];

if self.rom_country_popup_open {
    layers.push(self.rom_country_popup_view());
}

if self.rom_should_show_vc_runtime_popup() {
    layers.push(self.rom_vc_runtime_popup_view());
} else if self.rom_should_show_mtk_driver_popup() {
    layers.push(self.rom_mtk_driver_popup_view());
}

if self.should_show_dashboard_update_notice() {
    layers.push(self.dashboard_update_notice_view());
}

let stacked = iced::widget::Stack::with_children(layers)
    .width(Length::Fill)
    .height(Length::Fill);

stacked.into()
}

fn nav_icon_handle(&self, page: NavPage) -> iced::widget::image::Handle {
    match page {
        NavPage::Dashboard => self.nav_home_handle.clone(),
        NavPage::Rom => self.nav_refresh_handle.clone(),
        NavPage::Driver => self.nav_tab_settings_handle.clone(),
        NavPage::FirmwareDownload => self.nav_firmware_download_handle.clone(),
        NavPage::Qna => self.nav_qna_handle.clone(),
        NavPage::Log => self.nav_log_handle.clone(),
        NavPage::Settings => self.nav_settings_handle.clone(),
        NavPage::Backup => self.nav_log_handle.clone(),
    }
}

fn sidebar(&self) -> Element<'_, Message> {
    let label_t = ((self.sidebar_anim - 0.4) / 0.5).clamp(0.0, 1.0);
    let label_alpha = ease_out_cubic(label_t);

    let mut nav = column![].spacing(1).padding([16.0, 0.0]);

    nav = nav.push(lpm_nav_section_header("기기 관리", label_alpha));

    for &(page, _icon, label) in LPM_NAV_MAIN {
        nav = nav.push(lpm_nav_button(
            page,
            self.nav_icon_handle(page),
            label,
            self.active_nav == page,
            label_alpha,
        ));
    }

    nav = nav.push(lpm_nav_section_header("프로그램", label_alpha));

    for &(page, _icon, label) in LPM_NAV_TOOLS {
        nav = nav.push(lpm_nav_button(
            page,
            self.nav_icon_handle(page),
            label,
            self.active_nav == page,
            label_alpha,
        ));
    }

    let width =
        SIDEBAR_RAIL_WIDTH + (SIDEBAR_EXPANDED_WIDTH - SIDEBAR_RAIL_WIDTH) * self.sidebar_anim;

    let panel = container(nav)
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .style(lpm_nav_menu_panel_style);

    let divider = container(iced::widget::Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fill)
        .style(lpm_nav_divider_style);

    let shell = row![panel, divider].height(Length::Fill);

    iced::widget::mouse_area(shell)
        .on_enter(Message::SidebarHoverEnter)
        .on_exit(Message::SidebarHoverExit)
        .on_press(Message::Noop)
        .interaction(iced::mouse::Interaction::Idle)
        .into()
}

fn sidebar_anim_target(&self) -> f32 {
    if self.sidebar_expanded {
        1.0
    } else {
        0.0
    }
}

fn dashboard_panel(&self) -> Element<'_, Message> {
    let info = &self.dashboard_info;
    let lang = active_language_option();

    let model_title = if info.model_name.trim().is_empty()
        || info.model_name == "알 수 없음"
        || info.model_name == "감지 전"
    {
        "알 수 없음".to_string()
    } else {
        compact_text(&info.model_name, 52)
    };

    let model_preview_content: Element<'_, Message> = match self.dashboard_model_image {
        DashboardModelImage::Unknown => container(text(""))
            .width(Length::Fixed(150.0))
            .height(Length::Fixed(95.0))
            .style(lpm_nav_dashboard_black_screen_style)
            .into(),

        _ => iced::widget::image(self.dashboard_model_image_handle())
            .width(Length::Fixed(150.0))
            .height(Length::Fixed(95.0))
            .into(),
    };

    let model_preview = container(model_preview_content)
        .width(Length::Fixed(150.0))
        .height(Length::Fixed(168.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);

    let widevine_level_display = lpm_format_widevine_value(active_language_option(), &info.widevine_level);

    let left_column = column![model_preview]
        .spacing(0)
        .width(Length::Fixed(145.0))
        .height(Length::Fixed(168.0));

let dashboard_rtl = matches!(lang, LanguageOption::Arabic);
let dashboard_column_spacing = if dashboard_rtl { 3.0 } else { 5.0 };
let dashboard_content_spacing = if dashboard_rtl { 7.0 } else { 9.0 };
let dashboard_column1_width = if dashboard_rtl { 130.0 } else { 140.0 };
let dashboard_column2_width = if dashboard_rtl { 140.0 } else { 145.0 };
let dashboard_column3_width = if dashboard_rtl { 165.0 } else { 200.0 };

let column1 = column![
    self.dashboard_info_item("국가 코드", &info.country_code),
    self.dashboard_info_item("펌웨어 버전", &info.firmware_version),
    self.dashboard_info_item("설정 언어", &info.locale),
    self.dashboard_info_item("와이드바인 레벨", &widevine_level_display),
]
.spacing(5)
.width(Length::Fixed(dashboard_column1_width));

let column2 = column![
    self.dashboard_info_item("기기에 원본 롬", &info.original_rom),
    self.dashboard_info_item("기기에 설치한 롬", &info.installed_rom),
    self.dashboard_info_item("설정된 슬롯 값", &info.slot_suffix),
    self.dashboard_info_item("하드웨어 정보", &info.hardware_info),
]
.spacing(5)
.width(Length::Fixed(dashboard_column2_width));

let column3 = column![
    self.dashboard_info_item("AP 칩셋", &info.ap_chipset),
    self.dashboard_info_item("플랫폼", &info.platform),
    self.dashboard_info_item("시스템 업데이트(OTA)", &info.ota_status),
    self.dashboard_info_item("시리얼 넘버", &info.serial_number),
]
.spacing(5)
.width(Length::Fixed(dashboard_column3_width));

    let top_divider = container(iced::widget::Space::new())
        .width(Length::Fixed(1.3))
        .height(Length::Fixed(168.0))
        .style(lpm_nav_divider_style);

let info_grid = row![column1, column2, column3]
    .spacing(dashboard_column_spacing)
    .width(Length::Shrink);

    let content_row = row![left_column, top_divider, info_grid]
        .spacing(dashboard_content_spacing)
        .align_y(iced::Alignment::Start)
        .width(Length::Shrink);

    let top_panel = container(
        column![
            text(model_title)
                .size(27.5)
                .font(lpm_bold_font()),
            content_row,
        ]
        .spacing(8)
        .width(Length::Fill)
        .align_x(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(255.0))
    .padding([14.0, 12.0])
    .style(lpm_nav_dashboard_inner_style);

    let action_cards = row![
        self.dashboard_action_card(
            lpm_dashboard_promo_text(lang, "donate_title"),
            lpm_dashboard_promo_text(lang, "donate_description"),
            lpm_dashboard_promo_text(lang, "move_button"),
            Message::OpenDonate,
        ),
        self.dashboard_action_card(
            lpm_dashboard_promo_text(lang, "start_title"),
            lpm_dashboard_promo_text(lang, "start_description"),
            lpm_dashboard_promo_text(lang, "start_button"),
            Message::DashboardOpenRomFolderSelect,
        ),
        self.dashboard_action_card(
            lpm_dashboard_promo_text(lang, "more_title"),
            lpm_dashboard_promo_text(lang, "more_description"),
            lpm_dashboard_promo_text(lang, "move_button"),
            Message::OpenProgramVideos,
        ),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Start)
    .width(Length::Fill);

    container(
        column![
            text("대시 보드")
                .size(24)
                .font(lpm_bold_font()),
            top_panel,
            action_cards,
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding([8.0, 12.0])
    .into()
}

fn rom_image_folder_select_panel(&self) -> Element<'_, Message> {
    let content: Element<'_, Message> = if let Some(image_dir) = &self.image_dir {
        self.rom_after_folder_selected_panel(image_dir.display().to_string())
    } else {
        self.rom_select_folder_card()
    };

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .padding(iced::Padding {
            top: 24.0,
            right: 10.0,
            bottom: 24.0,
            left: 10.0,
        })
        .into()
}

fn rom_select_folder_card(&self) -> Element<'_, Message> {
    let select_card_content = container(
        column![
            iced::widget::image(self.folder_select_icon_handle.clone())
                .width(Length::Fixed(90.0))
                .height(Length::Fixed(90.0)),

            text("image 폴더를 선택해주세요.")
                .size(13)
                .font(lpm_font())
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),
        ]
        .spacing(20)
        .width(Length::Shrink)
        .align_x(iced::Alignment::Center),
    )
    .width(Length::Fixed(320.0))
    .height(Length::Fixed(190.0))
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .style(lpm_nav_rom_select_card_style);

    button(select_card_content)
        .padding(0)
        .on_press(Message::SelectImageFolder)
        .style(lpm_nav_flat_button_style)
        .into()
}

fn rom_after_folder_selected_panel(&self, image_dir_text: String) -> Element<'_, Message> {
let loading_handle = if self.loading_progress_handles.is_empty() {
    smooth_png_handle(
        LOADING_PROGRESS_FRAME_01_BYTES,
        LOADING_PROGRESS_SIZE,
        LOADING_PROGRESS_SIZE,
    )
} else {
    self.loading_progress_handles
        .get(self.rom_check_loading_frame % self.loading_progress_handles.len())
        .cloned()
        .unwrap_or_else(|| smooth_png_handle(
            LOADING_PROGRESS_FRAME_01_BYTES,
            LOADING_PROGRESS_SIZE,
            LOADING_PROGRESS_SIZE,
        ))
};

let loading_image = iced::widget::image(loading_handle)
    .width(Length::Fixed(LOADING_PROGRESS_SIZE as f32))
    .height(Length::Fixed(LOADING_PROGRESS_SIZE as f32));

    let driver_install_button: Element<'_, Message> =
        if self.rom_mtk_driver_installed == Some(false) {
            button(text("드라이버 설치").size(12))
                .on_press(Message::InstallMtkDriver)
                .into()
        } else {
            iced::widget::Space::new()
                .width(Length::Fixed(0.0))
                .height(Length::Fixed(1.0))
                .into()
        };

    let retry_controls: Element<'_, Message> = if self.busy {
        iced::widget::Space::new()
            .width(Length::Fixed(1.0))
            .height(Length::Fixed(1.0))
            .into()
    } else if self.rom_firmware_error.is_some() {
        row![
            button(text("재선택").size(12))
                .on_press(Message::SelectImageFolder),
            button(text("다시 검사").size(12))
                .on_press(Message::CheckFirmware),
            driver_install_button,
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        row![
            button(text("다시 검사").size(12))
                .on_press(Message::CheckFirmware),
            driver_install_button,
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .into()
    };

    let validation_panel = container(
        column![
            loading_image,

            text("펌웨어 버전, 플랫폼, 모델명, 필수 partition 유효성, MTK 드라이버 및 Microsoft Visual C++ 런타임 설치 유/무를 검사합니다.")
                .size(12)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),

            retry_controls,
        ]
        .spacing(12)
        .width(Length::Fill)
        .align_x(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding(24)
    .style(lpm_nav_rom_step_card_style);

    let mut sections = column![].spacing(10).width(Length::Fill);

    if self.rom_firmware_info.is_none() {
        sections = sections.push(validation_panel);
    }

    if let Some(error) = &self.rom_firmware_error {
        sections = sections.push(self.rom_firmware_error_panel(error));
    }

    if let Some(info) = &self.rom_firmware_info {
        if self.rom_show_routine_select {
            if self.rom_option_target.is_some() {
                sections = sections.push(self.rom_options_panel(info));
            } else {
                sections = sections.push(self.rom_routine_select_panel(info));
            }
        } else {
            sections = sections.push(self.rom_image_info_dashboard_panel(info, image_dir_text));
        }
    }

    let routine_select_page = self.rom_firmware_info.is_some()
        && self.rom_show_routine_select
        && self.rom_option_target.is_none();

    let section_panel = container(sections)
        .width(Length::Fixed(700.0))
        .height(if routine_select_page { Length::Fill } else { Length::Shrink })
        .padding(if self.rom_option_target.is_some() || routine_select_page {
            0
        } else {
            14
        });

    if self.rom_option_target.is_some() || routine_select_page {
        section_panel.into()
    } else {
        section_panel.style(lpm_nav_rom_step_panel_style).into()
    }
}

fn rom_image_info_dashboard_panel(
    &self,
    info: &FirmwareInfo,
    _image_dir_text: String,
) -> Element<'static, Message> {
    let validation_state = self.lpm_rom_folder_validation_state(info);
    let can_go_next = validation_state.can_continue();

    let folder_problem = validation_state.image_model_bad || validation_state.blocked_firmware;
    let connected_problem = validation_state.connected_device_unknown
        || validation_state.connected_device_bad
        || validation_state.connected_device_image_mismatch;
    let battery_problem = validation_state.battery_low;

    let folder_version = info
        .version
        .clone()
        .unwrap_or_else(|| "알 수 없음".to_string());

    let folder_type = region_label(info.region).to_string();

    let connected_model = lpm_clean_display_value(&self.dashboard_info.product_device);
    let connected_country = lpm_clean_display_value(&self.dashboard_info.country_code);

    let battery_text = self
        .dashboard_info
        .battery_level
        .map(|value| format!("{value}%"))
        .unwrap_or_else(|| "알 수 없음".to_string());

    let uptime_text = lpm_clean_display_value(&self.dashboard_info.uptime_display);

    let tablet_handle = if connected_problem {
        self.tablet_x_icon_handle.clone()
    } else {
        self.tablet_check_icon_handle.clone()
    };

    let folder_icon_widget: Element<'static, Message> = if validation_state.image_model_bad {
        container(
            column![
                text("올바른 image 폴더를\n선택해주세요.")
                    .size(10)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center),
                iced::widget::image(self.folder_check_icon_handle.clone())
                    .width(Length::Fixed(80.0))
                    .height(Length::Fixed(80.0)),
            ]
            .spacing(6)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fixed(118.0))
        .height(Length::Fixed(124.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into()
    } else {
        container(
            iced::widget::image(self.folder_check_icon_handle.clone())
                .width(Length::Fixed(105.0))
                .height(Length::Fixed(105.0)),
        )
        .width(Length::Fixed(118.0))
        .height(Length::Fixed(124.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into()
    };

    let folder_icon_line = row![
        folder_icon_widget,
        container(iced::widget::Space::new())
            .width(Length::Fixed(1.0))
            .height(Length::Fixed(130.0))
            .style(lpm_nav_divider_style),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let left_text_column = column![
        self.rom_folder_info_item("모델명", info.model.clone()),
        self.rom_folder_info_item("펌웨어 버전", folder_version),
        self.rom_folder_info_item("펌웨어 유형", folder_type),
    ]
    .spacing(12)
    .width(Length::Fixed(150.0));

    let left_card_inner = container(
        row![folder_icon_line, left_text_column]
            .spacing(16)
            .align_y(iced::Alignment::Center)
            .width(Length::Shrink),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center);

    let left_card = container(left_card_inner)
        .width(Length::Fixed(390.0))
        .height(Length::Fixed(262.0))
        .padding(14)
        .style(move |_theme: &Theme| lpm_nav_rom_status_card_style(folder_problem));

    let tablet_icon_line = row![
        container(
            iced::widget::image(tablet_handle)
                .width(Length::Fixed(68.0))
                .height(Length::Fixed(68.0)),
        )
        .width(Length::Fixed(86.0))
        .height(Length::Fixed(78.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
        container(iced::widget::Space::new())
            .width(Length::Fixed(1.2))
            .height(Length::Fixed(73.0))
            .style(lpm_nav_divider_style),
    ]
    .spacing(0)
    .align_y(iced::Alignment::Center);

    let tablet_text_column = column![
        self.rom_info_item("연결한 기기", connected_model),
        self.rom_info_item("국가 코드", connected_country),
    ]
    .spacing(10)
    .width(Length::Fixed(128.0));

    let right_top_inner = container(
        row![tablet_icon_line, tablet_text_column]
            .spacing(7)
            .align_y(iced::Alignment::Center)
            .width(Length::Shrink),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center);

    let right_top = container(right_top_inner)
        .width(Length::Fixed(275.0))
        .height(Length::Fixed(126.0))
        .padding(10)
        .style(move |_theme: &Theme| lpm_nav_rom_status_card_style(connected_problem));

    let battery_icon_line = row![
        container(
            iced::widget::image(self.battery_progress_handle.clone())
                .width(Length::Fixed(68.0))
                .height(Length::Fixed(68.0)),
        )
        .width(Length::Fixed(86.0))
        .height(Length::Fixed(78.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
        container(iced::widget::Space::new())
            .width(Length::Fixed(1.2))
            .height(Length::Fixed(73.0))
            .style(lpm_nav_divider_style),
    ]
    .spacing(0)
    .align_y(iced::Alignment::Center);

    let battery_text_column = column![
        self.rom_info_item("배터리 잔량", battery_text),
        self.rom_info_item("가동 시간", uptime_text),
    ]
    .spacing(10)
    .width(Length::Fixed(128.0));

    let right_bottom_inner = container(
        row![battery_icon_line, battery_text_column]
            .spacing(7)
            .align_y(iced::Alignment::Center)
            .width(Length::Shrink),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center);

    let right_bottom = container(right_bottom_inner)
        .width(Length::Fixed(275.0))
        .height(Length::Fixed(126.0))
        .padding(10)
        .style(move |_theme: &Theme| lpm_nav_rom_status_card_style(battery_problem));

    let right_column = column![right_top, right_bottom]
        .spacing(10)
        .width(Length::Fixed(275.0));

    let info_area = row![left_card, right_column]
        .spacing(12)
        .width(Length::Shrink)
        .align_y(iced::Alignment::Center);

    let reselect_label = lpm_translate_owned("폴더 재선택".to_string());
    let next_step_label = lpm_translate_owned("다음 단계로 이동".to_string());
    let bottom_button_width = lpm_equal_button_width(&[reselect_label.as_str(), next_step_label.as_str()], 96.0, 190.0);

    let reselect_button = button(
        container(iced_text(reselect_label).size(ROM_BOTTOM_BUTTON_FONT))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .width(Length::Fixed(bottom_button_width))
    .height(Length::Fixed(30.0))
    .padding(0.0)
    .on_press(Message::SelectImageFolder);

    let next_step_button: Element<'static, Message> = if can_go_next {
        button(
            container(iced_text(next_step_label).size(ROM_BOTTOM_BUTTON_FONT))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .width(Length::Fixed(bottom_button_width))
        .height(Length::Fixed(30.0))
        .padding(0.0)
        .on_press(Message::RomProceedToRoutine)
        .into()
    } else {
        button(
            container(iced_text(next_step_label).size(ROM_BOTTOM_BUTTON_FONT))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .width(Length::Fixed(bottom_button_width))
        .height(Length::Fixed(30.0))
        .padding(0.0)
        .style(lpm_nav_disabled_next_button_style)
        .into()
    };

    let bottom_buttons = row![
        reselect_button,
        next_step_button,
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let bottom_bar = container(bottom_buttons)
        .width(Length::Fill)
        .padding([4.0, 0.0])
        .align_x(iced::alignment::Horizontal::Center);

    let validation_log_text = self.lpm_rom_folder_validation_log_text(info);
    let validation_font_size = match active_language_option() {
        LanguageOption::Korean => 11.5,
        LanguageOption::Arabic => 10.0,
        _ => 10.5,
    };
    let validation_alignment = if active_language_option() == LanguageOption::Arabic {
        iced::alignment::Horizontal::Right
    } else {
        iced::alignment::Horizontal::Left
    };
    let mut validation_rows = column![].spacing(2).width(Length::Fill);

    for line in validation_log_text.lines().filter(|line| !line.trim().is_empty()) {
        validation_rows = validation_rows.push(
            iced_text(line.to_string())
                .size(validation_font_size)
                .width(Length::Fill)
                .align_x(validation_alignment)
                .wrapping(iced::widget::text::Wrapping::Word),
        );
    }

    let validation_log = container(validation_rows)
        .width(Length::Fixed(560.0))
        .padding([8.0, 12.0])
        .style(lpm_nav_rom_validation_log_style);

    container(
        column![
            container(info_area)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center),
            container(bottom_bar)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center),
            container(validation_log)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center),
        ]
        .spacing(12)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding(0)
    .into()
}



fn rom_folder_info_item(&self, title: &'static str, value: String) -> Element<'static, Message> {
    let value_text = if value.trim().is_empty() {
        lpm_translate_owned("알 수 없음".to_string())
    } else {
        lpm_translate_owned(compact_text(value.trim(), ROM_FOLDER_INFO_VALUE_MAX_CHARS))
    };
    let title_text = lpm_translate_owned(title.to_string());

    container(
        column![
            text(title_text)
                .size(ROM_FOLDER_INFO_TITLE_FONT)
                .font(lpm_bold_font())
                .wrapping(iced::widget::text::Wrapping::Word),
            text(value_text)
                .size(ROM_FOLDER_INFO_VALUE_FONT)
                .wrapping(iced::widget::text::Wrapping::Word),
        ]
        .spacing(2),
    )
    .width(Length::Fill)
    .into()
}

fn rom_info_item(&self, title: &'static str, value: String) -> Element<'static, Message> {
    let value_text = if value.trim().is_empty() {
        lpm_translate_owned("알 수 없음".to_string())
    } else {
        lpm_translate_owned(compact_text(value.trim(), ROM_RIGHT_INFO_VALUE_MAX_CHARS))
    };
    let title_text = if active_language_option() == LanguageOption::Russian && title == "연결한 기기" {
        "Подключённое\nустройство".to_string()
    } else {
        lpm_translate_owned(title.to_string())
    };
    let title_size = if active_language_option() == LanguageOption::Russian {
        11
    } else {
        ROM_RIGHT_INFO_TITLE_FONT
    };

    container(
        column![
            text(title_text)
                .size(title_size)
                .font(lpm_bold_font())
                .wrapping(iced::widget::text::Wrapping::Word),
            text(value_text)
                .size(ROM_RIGHT_INFO_VALUE_FONT)
                .wrapping(iced::widget::text::Wrapping::Word),
        ]
        .spacing(2),
    )
    .width(Length::Fill)
    .into()
}

fn lpm_rom_folder_validation_state(
    &self,
    info: &FirmwareInfo,
) -> LpmRomFolderValidationState {
    let image_model_bad = !is_supported_lpmbox_model(&info.model);

    let connected_device = self.dashboard_info.product_device.trim();
    let connected_device_unknown = lpm_is_unknown_text(connected_device);

    let connected_device_bad =
        !connected_device_unknown && !is_supported_lpmbox_model(connected_device);

    let image_model = normalize_lenovo_model_for_compare(&info.model);
    let connected_model = normalize_lenovo_model_for_compare(connected_device);
    let connected_device_image_mismatch = !connected_device_unknown
        && !connected_device_bad
        && !image_model_bad
        && !is_same_or_convertible_lpmbox_model_pair(&connected_model, &image_model);

let battery_low = self
    .dashboard_info
    .battery_level
    .map(|level| level <= LPMBOX_MIN_BATTERY_LEVEL)
    .unwrap_or(false);

let blocked_firmware = info.blocked_firmware_check.blocked;
let mtk_driver_missing = self.rom_mtk_driver_installed == Some(false);
let vc_runtime_missing = self.rom_vc_runtime_error.is_some()
    || self
        .rom_vc_runtime_status
        .map(|status| !status.all_installed())
        .unwrap_or(false);

LpmRomFolderValidationState {
    image_model_bad,
    connected_device_unknown,
    connected_device_bad,
    connected_device_image_mismatch,
    battery_low,
    blocked_firmware,
    mtk_driver_missing,
    vc_runtime_missing,
}
}

fn lpm_rom_folder_validation_issue_lines(
    &self,
    info: &FirmwareInfo,
) -> Vec<String> {
    let state = self.lpm_rom_folder_validation_state(info);
    let lang = active_language_option();
    let mut lines = Vec::new();

    if state.image_model_bad {
        lines.push(match lang {
            LanguageOption::Korean => "LPMBOX에서 지원하지 않는 image 폴더입니다.",
            LanguageOption::English => "This image folder is not supported by LPMBox.",
            LanguageOption::Russian => "Эта папка image не поддерживается LPMBox.",
            LanguageOption::Japanese => "このimageフォルダーはLPMBoxでサポートされていません。",
            LanguageOption::TraditionalChinese => "此 image 資料夾不受 LPMBox 支援。",
            LanguageOption::Vietnamese => "Thư mục image này không được LPMBox hỗ trợ.",
            LanguageOption::Greek => "Αυτός ο φάκελος image δεν υποστηρίζεται από το LPMBox.",
            LanguageOption::Hindi => "यह image फ़ोल्डर LPMBox द्वारा समर्थित नहीं है।",
            LanguageOption::Georgian => "ეს image საქაღალდე LPMBox-ის მიერ მხარდაჭერილი არ არის.",
            LanguageOption::Dutch => "Deze image-map wordt niet ondersteund door LPMBox.",
            LanguageOption::Arabic => "مجلد image هذا غير مدعوم في LPMBox.",
            LanguageOption::Spanish => "Esta carpeta image no es compatible con LPMBox.",
        }.to_string());
    }

    if state.blocked_firmware {
        lines.push(match lang {
            LanguageOption::Korean => "선택한 image 폴더의 펌웨어 버전에 심각한 버그가 있습니다.",
            LanguageOption::English => "The firmware version in the selected image folder has a serious bug.",
            LanguageOption::Russian => "В версии прошивки в выбранной папке image есть серьёзная ошибка.",
            LanguageOption::Japanese => "選択したimageフォルダーのファームウェアバージョンには重大な不具合があります。",
            LanguageOption::TraditionalChinese => "已選擇的 image 資料夾中的韌體版本存在嚴重錯誤。",
            LanguageOption::Vietnamese => "Phiên bản firmware trong thư mục image đã chọn có lỗi nghiêm trọng.",
            LanguageOption::Greek => "Η έκδοση firmware στον επιλεγμένο φάκελο image έχει σοβαρό σφάλμα.",
            LanguageOption::Hindi => "चयनित image फ़ोल्डर की firmware संस्करण में गंभीर त्रुटि है।",
            LanguageOption::Georgian => "არჩეულ image საქაღალდეში არსებული firmware ვერსია სერიოზულ შეცდომას შეიცავს.",
            LanguageOption::Dutch => "De firmwareversie in de geselecteerde image-map bevat een ernstige fout.",
            LanguageOption::Arabic => "إصدار firmware داخل مجلد image المحدد يحتوي على خطأ خطير.",
            LanguageOption::Spanish => "La versión de firmware de la carpeta image seleccionada tiene un error grave.",
        }.to_string());
    }

    if state.connected_device_unknown {
        lines.push(match lang {
            LanguageOption::Korean => "기기가 연결되어 있지 않거나 ADB가 감지되지 않습니다.\n1) 케이블을 PC 후면 USB 포트에 꽂아주세요. 노트북은 상관 없습니다.\n2) 올바른 데이터 케이블을 사용해주세요. Q&A를 참고하세요.",
            LanguageOption::English => "The device is not connected or ADB was not detected.\n1) Connect the cable to a rear USB port on the PC. This is not required for laptops.\n2) Use a proper data cable. See the Q&A.",
            LanguageOption::Russian => "Устройство не подключено или ADB не обнаружен.\n1) Подключите кабель к заднему USB-порту ПК. Для ноутбуков это не требуется.\n2) Используйте подходящий кабель передачи данных. См. Q&A.",
            LanguageOption::Japanese => "端末が接続されていないか、ADBが検出されていません。\n1) ケーブルをPC背面のUSBポートに接続してください。ノートPCの場合は不要です。\n2) 適切なデータケーブルを使用してください。Q&Aを参照してください。",
            LanguageOption::TraditionalChinese => "裝置未連接，或未偵測到 ADB。\n1) 請將線材接到桌機後方 USB 連接埠。筆電不需要這樣做。\n2) 請使用正確的資料傳輸線。請參考 Q&A。",
            LanguageOption::Vietnamese => "Thiết bị chưa được kết nối hoặc không phát hiện được ADB.\n1) Hãy cắm cáp vào cổng USB phía sau PC. Không cần thực hiện với laptop.\n2) Hãy dùng cáp dữ liệu phù hợp. Xem Q&A.",
            LanguageOption::Greek => "Η συσκευή δεν είναι συνδεδεμένη ή δεν εντοπίστηκε ADB.\n1) Συνδέστε το καλώδιο σε πίσω θύρα USB του PC. Δεν απαιτείται για laptop.\n2) Χρησιμοποιήστε σωστό καλώδιο δεδομένων. Δείτε το Q&A.",
            LanguageOption::Hindi => "डिवाइस कनेक्ट नहीं है या ADB नहीं मिला।\n1) केबल को PC के पीछे वाले USB पोर्ट में लगाएँ। लैपटॉप के लिए यह ज़रूरी नहीं है।\n2) सही डेटा केबल का उपयोग करें। Q&A देखें।",
            LanguageOption::Georgian => "მოწყობილობა დაკავშირებული არ არის ან ADB ვერ მოიძებნა.\n1) კაბელი შეაერთეთ PC-ის უკანა USB პორტში. ლეპტოპისთვის ეს საჭირო არ არის.\n2) გამოიყენეთ სწორი მონაცემთა კაბელი. იხილეთ Q&A.",
            LanguageOption::Dutch => "Het apparaat is niet aangesloten of ADB is niet gedetecteerd.\n1) Sluit de kabel aan op een USB-poort aan de achterkant van de pc. Dit is niet nodig bij laptops.\n2) Gebruik een geschikte datakabel. Zie de Q&A.",
            LanguageOption::Arabic => "الجهاز غير متصل أو لم يتم اكتشاف ADB.\n1) وصّل الكابل بمنفذ USB الخلفي في الكمبيوتر. لا يلزم ذلك مع أجهزة اللابتوب.\n2) استخدم كابل بيانات مناسبًا. راجع Q&A.",
            LanguageOption::Spanish => "El dispositivo no está conectado o no se detectó ADB.\n1) Conecte el cable a un puerto USB trasero del PC. No es necesario en portátiles.\n2) Use un cable de datos adecuado. Consulte las Q&A.",
        }.to_string());
    } else if state.connected_device_bad {
        let device = lpm_clean_display_value(&self.dashboard_info.product_device);

        lines.push(match lang {
            LanguageOption::Korean => format!("연결한 기기({device})에서는 LPMBox를 사용할 수 없습니다."),
            LanguageOption::English => format!("The connected device ({device}) cannot be used with LPMBox."),
            LanguageOption::Russian => format!("Подключённое устройство ({device}) нельзя использовать с LPMBox."),
            LanguageOption::Japanese => format!("接続した端末（{device}）ではLPMBoxを使用できません。"),
            LanguageOption::TraditionalChinese => format!("已連接裝置（{device}）無法使用 LPMBox。"),
            LanguageOption::Vietnamese => format!("Thiết bị đã kết nối ({device}) không thể sử dụng LPMBox."),
            LanguageOption::Greek => format!("Η συνδεδεμένη συσκευή ({device}) δεν μπορεί να χρησιμοποιήσει το LPMBox."),
            LanguageOption::Hindi => format!("कनेक्टेड डिवाइस ({device}) पर LPMBox का उपयोग नहीं किया जा सकता।"),
            LanguageOption::Georgian => format!("დაკავშირებულ მოწყობილობაზე ({device}) LPMBox-ის გამოყენება შეუძლებელია."),
            LanguageOption::Dutch => format!("Het aangesloten apparaat ({device}) kan niet met LPMBox worden gebruikt."),
            LanguageOption::Arabic => format!("لا يمكن استخدام LPMBox مع الجهاز المتصل ({device})."),
            LanguageOption::Spanish => format!("El dispositivo conectado ({device}) no se puede usar con LPMBox."),
        });
    } else if state.connected_device_image_mismatch {
        let device = lpm_clean_display_value(&self.dashboard_info.product_device);
        let image = lpm_clean_display_value(&info.model);

        lines.push(match lang {
            LanguageOption::Korean => format!("연결한 기기({device})와 선택한 image 폴더({image})가 호환되지 않습니다."),
            LanguageOption::English => format!("The connected device ({device}) is not compatible with the selected image folder ({image})."),
            LanguageOption::Russian => format!("Подключённое устройство ({device}) несовместимо с выбранной папкой image ({image})."),
            LanguageOption::Japanese => format!("接続した端末（{device}）と選択したimageフォルダー（{image}）は互換性がありません。"),
            LanguageOption::TraditionalChinese => format!("已連接裝置（{device}）與已選擇的 image 資料夾（{image}）不相容。"),
            LanguageOption::Vietnamese => format!("Thiết bị đã kết nối ({device}) không tương thích với thư mục image đã chọn ({image})."),
            LanguageOption::Greek => format!("Η συνδεδεμένη συσκευή ({device}) δεν είναι συμβατή με τον επιλεγμένο φάκελο image ({image})."),
            LanguageOption::Hindi => format!("कनेक्टेड डिवाइस ({device}) चयनित image फ़ोल्डर ({image}) के साथ संगत नहीं है।"),
            LanguageOption::Georgian => format!("დაკავშირებული მოწყობილობა ({device}) არჩეულ image საქაღალდესთან ({image}) თავსებადი არ არის."),
            LanguageOption::Dutch => format!("Het aangesloten apparaat ({device}) is niet compatibel met de geselecteerde image-map ({image})."),
            LanguageOption::Arabic => format!("الجهاز المتصل ({device}) غير متوافق مع مجلد image المحدد ({image})."),
            LanguageOption::Spanish => format!("El dispositivo conectado ({device}) no es compatible con la carpeta image seleccionada ({image})."),
        });
    }

    if state.battery_low {
        lines.push(match lang {
            LanguageOption::Korean => "기기의 배터리가 부족합니다.",
            LanguageOption::English => "The device battery level is too low.",
            LanguageOption::Russian => "Уровень заряда устройства слишком низкий.",
            LanguageOption::Japanese => "端末のバッテリー残量が不足しています。",
            LanguageOption::TraditionalChinese => "裝置電量不足。",
            LanguageOption::Vietnamese => "Mức pin của thiết bị quá thấp.",
            LanguageOption::Greek => "Η στάθμη μπαταρίας της συσκευής είναι πολύ χαμηλή.",
            LanguageOption::Hindi => "डिवाइस की बैटरी बहुत कम है।",
            LanguageOption::Georgian => "მოწყობილობის ბატარეის დონე ძალიან დაბალია.",
            LanguageOption::Dutch => "Het batterijniveau van het apparaat is te laag.",
            LanguageOption::Arabic => "مستوى بطارية الجهاز منخفض جدًا.",
            LanguageOption::Spanish => "El nivel de batería del dispositivo es demasiado bajo.",
        }.to_string());
    }

    if state.vc_runtime_missing {
        lines.push(match lang {
            LanguageOption::Korean => "Microsoft Visual C++ x86/x64 런타임 설치가 필요합니다.",
            LanguageOption::English => "Microsoft Visual C++ x86/x64 Runtime installation is required.",
            LanguageOption::Russian => "Требуется установка среды выполнения Microsoft Visual C++ x86/x64.",
            LanguageOption::Japanese => "Microsoft Visual C++ x86/x64 ランタイムのインストールが必要です。",
            LanguageOption::TraditionalChinese => "需要安裝 Microsoft Visual C++ x86/x64 執行階段。",
            LanguageOption::Vietnamese => "Cần cài đặt Microsoft Visual C++ Runtime x86/x64.",
            LanguageOption::Greek => "Απαιτείται εγκατάσταση του Microsoft Visual C++ Runtime x86/x64.",
            LanguageOption::Hindi => "Microsoft Visual C++ x86/x64 Runtime इंस्टॉल करना आवश्यक है।",
            LanguageOption::Georgian => "საჭიროა Microsoft Visual C++ x86/x64 Runtime-ის დაყენება.",
            LanguageOption::Dutch => "Installatie van Microsoft Visual C++ Runtime x86/x64 is vereist.",
            LanguageOption::Arabic => "يلزم تثبيت Microsoft Visual C++ Runtime لإصداري x86 وx64.",
            LanguageOption::Spanish => "Se requiere instalar Microsoft Visual C++ Runtime x86/x64.",
        }.to_string());
    }

    if state.mtk_driver_missing {
        lines.push(match lang {
            LanguageOption::Korean => "MTK 드라이버 설치가 필요합니다.",
            LanguageOption::English => "MTK driver installation is required.",
            LanguageOption::Russian => "Требуется установка драйвера MTK.",
            LanguageOption::Japanese => "MTKドライバーのインストールが必要です。",
            LanguageOption::TraditionalChinese => "需要安裝 MTK 驅動程式。",
            LanguageOption::Vietnamese => "Cần cài đặt driver MTK.",
            LanguageOption::Greek => "Απαιτείται εγκατάσταση του προγράμματος οδήγησης MTK.",
            LanguageOption::Hindi => "MTK ड्राइवर इंस्टॉलेशन आवश्यक है।",
            LanguageOption::Georgian => "საჭიროა MTK დრაივერის დაყენება.",
            LanguageOption::Dutch => "Installatie van de MTK-driver is vereist.",
            LanguageOption::Arabic => "يلزم تثبيت تعريف MTK.",
            LanguageOption::Spanish => "Se requiere instalar el controlador MTK.",
        }.to_string());
    }

    lines
}

fn lpm_rom_folder_validation_action_text(
    &self,
    state: LpmRomFolderValidationState,
) -> String {
    let lang = active_language_option();
    let mut actions = Vec::new();

    if state.image_model_bad {
        actions.push(match lang {
            LanguageOption::Korean => "올바른 image 폴더를 다시 선택",
            LanguageOption::English => "reselect the correct image folder",
            LanguageOption::Russian => "выберите правильную папку image повторно",
            LanguageOption::Japanese => "正しいimageフォルダーを再選択",
            LanguageOption::TraditionalChinese => "重新選擇正確的 image 資料夾",
            LanguageOption::Vietnamese => "chọn lại thư mục image đúng",
            LanguageOption::Greek => "επιλέξτε ξανά τον σωστό φάκελο image",
            LanguageOption::Hindi => "सही image फ़ोल्डर फिर से चुनें",
            LanguageOption::Georgian => "ხელახლა აირჩიეთ სწორი image საქაღალდე",
            LanguageOption::Dutch => "selecteer de juiste image-map opnieuw",
            LanguageOption::Arabic => "إعادة اختيار مجلد image الصحيح",
            LanguageOption::Spanish => "vuelva a seleccionar la carpeta image correcta",
        });
    }

    if state.blocked_firmware {
        actions.push(match lang {
            LanguageOption::Korean => "다른 버전 파일로 다시 시도",
            LanguageOption::English => "try again with a different version file",
            LanguageOption::Russian => "повторите с файлом другой версии",
            LanguageOption::Japanese => "別バージョンのファイルで再試行",
            LanguageOption::TraditionalChinese => "使用其他版本檔案重試",
            LanguageOption::Vietnamese => "thử lại bằng tệp phiên bản khác",
            LanguageOption::Greek => "δοκιμάστε ξανά με αρχείο άλλης έκδοσης",
            LanguageOption::Hindi => "किसी दूसरे संस्करण की फ़ाइल से फिर प्रयास करें",
            LanguageOption::Georgian => "სხვა ვერსიის ფაილით სცადეთ თავიდან",
            LanguageOption::Dutch => "probeer opnieuw met een ander versiebestand",
            LanguageOption::Arabic => "المحاولة مرة أخرى باستخدام ملف إصدار آخر",
            LanguageOption::Spanish => "inténtelo de nuevo con un archivo de otra versión",
        });
    }

    if state.connected_device_unknown {
        actions.push(match lang {
            LanguageOption::Korean => "기기를 연결",
            LanguageOption::English => "connect the device",
            LanguageOption::Russian => "подключите устройство",
            LanguageOption::Japanese => "端末を接続",
            LanguageOption::TraditionalChinese => "連接裝置",
            LanguageOption::Vietnamese => "kết nối thiết bị",
            LanguageOption::Greek => "συνδέστε τη συσκευή",
            LanguageOption::Hindi => "डिवाइस कनेक्ट करें",
            LanguageOption::Georgian => "დააკავშირეთ მოწყობილობა",
            LanguageOption::Dutch => "sluit het apparaat aan",
            LanguageOption::Arabic => "توصيل الجهاز",
            LanguageOption::Spanish => "conecte el dispositivo",
        });
    } else if state.connected_device_bad {
        actions.push(match lang {
            LanguageOption::Korean => "지원되는 기기를 연결",
            LanguageOption::English => "connect a supported device",
            LanguageOption::Russian => "подключите поддерживаемое устройство",
            LanguageOption::Japanese => "対応端末を接続",
            LanguageOption::TraditionalChinese => "連接受支援的裝置",
            LanguageOption::Vietnamese => "kết nối thiết bị được hỗ trợ",
            LanguageOption::Greek => "συνδέστε υποστηριζόμενη συσκευή",
            LanguageOption::Hindi => "समर्थित डिवाइस कनेक्ट करें",
            LanguageOption::Georgian => "დააკავშირეთ მხარდაჭერილი მოწყობილობა",
            LanguageOption::Dutch => "sluit een ondersteund apparaat aan",
            LanguageOption::Arabic => "توصيل جهاز مدعوم",
            LanguageOption::Spanish => "conecte un dispositivo compatible",
        });
    } else if state.connected_device_image_mismatch {
        actions.push(match lang {
            LanguageOption::Korean => "기기에 맞는 image 폴더를 선택",
            LanguageOption::English => "select an image folder that matches the device",
            LanguageOption::Russian => "выберите папку image, подходящую для устройства",
            LanguageOption::Japanese => "端末に合うimageフォルダーを選択",
            LanguageOption::TraditionalChinese => "選擇符合裝置的 image 資料夾",
            LanguageOption::Vietnamese => "chọn thư mục image phù hợp với thiết bị",
            LanguageOption::Greek => "επιλέξτε φάκελο image που ταιριάζει στη συσκευή",
            LanguageOption::Hindi => "डिवाइस से मेल खाने वाला image फ़ोल्डर चुनें",
            LanguageOption::Georgian => "აირჩიეთ მოწყობილობაზე მორგებული image საქაღალდე",
            LanguageOption::Dutch => "selecteer een image-map die bij het apparaat past",
            LanguageOption::Arabic => "اختيار مجلد image المتوافق مع الجهاز",
            LanguageOption::Spanish => "seleccione una carpeta image compatible con el dispositivo",
        });
    }

    if state.battery_low {
        actions.push(match lang {
            LanguageOption::Korean => "25% 이상 충전 후 다시 시도",
            LanguageOption::English => "charge to at least 25% and try again",
            LanguageOption::Russian => "зарядите минимум до 25% и повторите попытку",
            LanguageOption::Japanese => "25%以上充電してから再試行",
            LanguageOption::TraditionalChinese => "充電至 25% 以上後重試",
            LanguageOption::Vietnamese => "sạc lên ít nhất 25% rồi thử lại",
            LanguageOption::Greek => "φορτίστε τουλάχιστον στο 25% και δοκιμάστε ξανά",
            LanguageOption::Hindi => "कम से कम 25% चार्ज करके फिर प्रयास करें",
            LanguageOption::Georgian => "დატენეთ მინიმუმ 25%-მდე და სცადეთ თავიდან",
            LanguageOption::Dutch => "laad op tot minimaal 25% en probeer opnieuw",
            LanguageOption::Arabic => "الشحن إلى 25% على الأقل ثم المحاولة مرة أخرى",
            LanguageOption::Spanish => "cargue al menos al 25% e inténtelo de nuevo",
        });
    }

    if state.vc_runtime_missing {
        actions.push(match lang {
            LanguageOption::Korean => "Microsoft Visual C++ x86/x64 런타임을 설치",
            LanguageOption::English => "install Microsoft Visual C++ Runtime x86/x64",
            LanguageOption::Russian => "установите среду выполнения Microsoft Visual C++ x86/x64",
            LanguageOption::Japanese => "Microsoft Visual C++ x86/x64 ランタイムをインストール",
            LanguageOption::TraditionalChinese => "安裝 Microsoft Visual C++ x86/x64 執行階段",
            LanguageOption::Vietnamese => "cài đặt Microsoft Visual C++ Runtime x86/x64",
            LanguageOption::Greek => "εγκαταστήστε το Microsoft Visual C++ Runtime x86/x64",
            LanguageOption::Hindi => "Microsoft Visual C++ x86/x64 Runtime इंस्टॉल करें",
            LanguageOption::Georgian => "დააყენეთ Microsoft Visual C++ x86/x64 Runtime",
            LanguageOption::Dutch => "installeer Microsoft Visual C++ Runtime x86/x64",
            LanguageOption::Arabic => "تثبيت Microsoft Visual C++ Runtime لإصداري x86 وx64",
            LanguageOption::Spanish => "instale Microsoft Visual C++ Runtime x86/x64",
        });
    }

    if state.mtk_driver_missing {
        actions.push(match lang {
            LanguageOption::Korean => "MTK 드라이버를 설치",
            LanguageOption::English => "install the MTK driver",
            LanguageOption::Russian => "установите драйвер MTK",
            LanguageOption::Japanese => "MTKドライバーをインストール",
            LanguageOption::TraditionalChinese => "安裝 MTK 驅動程式",
            LanguageOption::Vietnamese => "cài đặt driver MTK",
            LanguageOption::Greek => "εγκαταστήστε τον οδηγό MTK",
            LanguageOption::Hindi => "MTK ड्राइवर इंस्टॉल करें",
            LanguageOption::Georgian => "დააყენეთ MTK დრაივერი",
            LanguageOption::Dutch => "installeer de MTK-driver",
            LanguageOption::Arabic => "تثبيت تعريف MTK",
            LanguageOption::Spanish => "instale el controlador MTK",
        });
    }

    if actions.is_empty() {
        String::new()
    } else {
        match lang {
            LanguageOption::Korean => format!("{}해주세요.", actions.join(", ")),
            LanguageOption::English => format!("Please {}.", actions.join(", ")),
            LanguageOption::Russian => format!("Выполните: {}.", actions.join(", ")),
            LanguageOption::Japanese => format!("{}してください。", actions.join("、")),
            LanguageOption::TraditionalChinese => format!("請{}。", actions.join("、")),
            LanguageOption::Vietnamese => format!("Vui lòng {}.", actions.join(", ")),
            LanguageOption::Greek => format!("Παρακαλώ {}.", actions.join(", ")),
            LanguageOption::Hindi => format!("कृपया {}।", actions.join(", ")),
            LanguageOption::Georgian => format!("გთხოვთ, {}.", actions.join(", ")),
            LanguageOption::Dutch => format!("Gelieve {}.", actions.join(", ")),
            LanguageOption::Arabic => format!("يرجى {}.", actions.join("، ")),
            LanguageOption::Spanish => format!("Por favor, {}.", actions.join(", ")),
        }
    }
}

fn lpm_rom_folder_validation_log_text(
    &self,
    info: &FirmwareInfo,
) -> String {
    let state = self.lpm_rom_folder_validation_state(info);
    let issue_lines = self.lpm_rom_folder_validation_issue_lines(info);

    if issue_lines.is_empty() {
        return match active_language_option() {
            LanguageOption::Korean => "다음 단계로 진행해주세요.",
            LanguageOption::English => "Proceed to the next step.",
            LanguageOption::Russian => "Перейдите к следующему шагу.",
            LanguageOption::Japanese => "次のステップへ進んでください。",
            LanguageOption::TraditionalChinese => "請前往下一步。",
            LanguageOption::Vietnamese => "Hãy chuyển sang bước tiếp theo.",
            LanguageOption::Greek => "Προχωρήστε στο επόμενο βήμα.",
            LanguageOption::Hindi => "अगले चरण पर जाएँ।",
            LanguageOption::Georgian => "გადადით შემდეგ ეტაპზე.",
            LanguageOption::Dutch => "Ga door naar de volgende stap.",
            LanguageOption::Arabic => "انتقل إلى الخطوة التالية.",
            LanguageOption::Spanish => "Continúe con el siguiente paso.",
        }
        .to_string();
    }

    let numbered = issue_lines.len() > 1;
    let mut output = Vec::new();

    for (issue_index, issue) in issue_lines.iter().enumerate() {
        for (line_index, line) in issue.lines().filter(|line| !line.trim().is_empty()).enumerate() {
            if numbered && line_index == 0 {
                output.push(format!("{}) {}", issue_index + 1, line.trim()));
            } else if numbered {
                output.push(format!("   {}", line.trim()));
            } else {
                output.push(line.trim().to_string());
            }
        }
    }

    let action = self.lpm_rom_folder_validation_action_text(state);
    if !action.is_empty()
        && !(issue_lines.len() == 1
            && (state.connected_device_unknown
                || state.mtk_driver_missing
                || state.vc_runtime_missing))
    {
        output.push(action);
    }

    output.join("\n")
}

fn rom_firmware_error_panel(&self, error: &str) -> Element<'static, Message> {
    container(
        column![
            text("펌웨어 검사 실패")
                .size(14)
                .font(lpm_bold_font()),
            text(error.to_string()).size(12),
            text("image 폴더 안에 flash.xml, scatter, DA 파일이 올바르게 있는지 확인해주세요.")
                .size(12),
        ]
        .spacing(5),
    )
    .width(Length::Fill)
    .padding(12)
    .style(lpm_nav_rom_error_card_style)
    .into()
}

fn rom_routine_select_panel(&self, info: &FirmwareInfo) -> Element<'static, Message> {
    let lang = active_language_option();
    let restore_only_mode = lpm_is_unknown_text(&self.dashboard_info.product_device);
    let installed_rom = normalize_rom_label_for_option(&self.dashboard_info.installed_rom);
    let image_rom = match info.region {
        RomRegion::Prc => "PRC",
        RomRegion::Row => "ROW",
        RomRegion::Unknown => "",
    };

    let install_disabled = restore_only_mode
        || installed_rom.is_empty()
        || image_rom.is_empty()
        || installed_rom == image_rom;

    let row_update_disabled = restore_only_mode
        || installed_rom != "ROW"
        || !matches!(info.region, RomRegion::Row);

    let install_message = if install_disabled {
        Message::Noop
    } else {
        Message::RomOpenOptions(RomSlideTarget::Install)
    };

    let row_update_message = if row_update_disabled {
        Message::Noop
    } else {
        Message::RomOpenOptions(RomSlideTarget::Update)
    };

    let row_update_description = if restore_only_mode {
        lpm_rom_routine_ui_text(lang, "기기가 연결되어 있지 않아 실행할 수 없습니다.")
    } else if installed_rom == "ROW" && matches!(info.region, RomRegion::Row) {
        lpm_rom_routine_ui_text(lang, "ROW(글로벌롬) 버전을 업데이트합니다.")
    } else if installed_rom == "PRC" {
        lpm_rom_routine_ui_text(lang, "기기가 PRC(중국 내수롬)이므로 불가능 합니다.")
    } else if matches!(info.region, RomRegion::Prc) {
        lpm_rom_routine_ui_text(lang, "image 폴더 유형이 PRC(중국 내수롬)이므로 불가능 합니다.")
    } else {
        lpm_rom_routine_ui_text(lang, "ROM 타입을 확인할 수 없어 업데이트 실행을 보류합니다.")
    };

    let install_description = if restore_only_mode {
        lpm_rom_routine_ui_text(lang, "기기가 연결되어 있지 않아 설치를 실행할 수 없습니다.")
    } else if installed_rom == "ROW" && matches!(info.region, RomRegion::Prc) {
        lpm_rom_routine_ui_text(lang, "ROW(글로벌롬)기기에 PRC(중국 내수롬)을 설치합니다.")
    } else if installed_rom == "PRC" && matches!(info.region, RomRegion::Row) {
        lpm_rom_routine_ui_text(lang, "PRC(중국 내수롬)기기에 ROW(글로벌롬)을 설치합니다.")
    } else if installed_rom == "ROW" && matches!(info.region, RomRegion::Row) {
        lpm_rom_routine_ui_text(lang, "ROW(글로벌롬) 업데이트로 진행해 주세요.")
    } else if installed_rom == "PRC" && matches!(info.region, RomRegion::Prc) {
        lpm_rom_routine_ui_text(lang, "PRC(중국 내수롬) 업데이트는 지원하지 않습니다.")
    } else {
        lpm_rom_routine_ui_text(lang, "ROM 타입을 확인할 수 없어 설치 실행을 보류합니다.")
    };

    let reinstall_description = lpm_rom_routine_ui_text(
        lang,
        "기기가 켜지지 않거나, 무한 재부팅 등 다양한 오류를 고칩니다.",
    );

    let install_row = self.rom_routine_slide_row(
        self.rom_install_icon_handle.clone(),
        self.warning_icon_handle.clone(),
        RomSlideTarget::Install,
        lpm_rom_routine_ui_text(lang, "PRC ↔ ROW 설치"),
        install_description,
        lpm_rom_routine_ui_text(lang, "PRC ↔ ROW 설치 루틴"),
        lpm_rom_routine_ui_text(lang, "중국 내수롬과 글로벌롬을 자유롭게 변경 가능"),
        lpm_rom_routine_ui_text(lang, "데이터 초기화가 필수이기 때문에"),
        lpm_rom_routine_ui_text(lang, "시작하기 전 데이터 백업 후 진행해주세요."),
        install_message,
        install_disabled,
    );

    let update_row = self.rom_routine_slide_row(
        self.rom_update_icon_handle.clone(),
        self.warning_icon_handle.clone(),
        RomSlideTarget::Update,
        lpm_rom_routine_ui_text(lang, "ROW(글로벌롬) 업데이트"),
        row_update_description,
        lpm_rom_routine_ui_text(lang, "ROW(글로벌롬) 업데이트 루틴"),
        lpm_rom_routine_ui_text(lang, "글로벌롬 펌웨어 버전을 업데이트합니다."),
        lpm_rom_routine_ui_text(lang, "기기에 설치된 버전보다 낮을 경우/초기화 O."),
        lpm_rom_routine_ui_text(lang, "기기에 설치된 버전보다 높은 경우/초기화 X."),
        row_update_message,
        row_update_disabled,
    );

    let reinstall_row = self.rom_routine_slide_row(
        self.tablet_fix_icon_handle.clone(),
        self.warning_icon_handle.clone(),
        RomSlideTarget::Reinstall,
        lpm_rom_routine_ui_text(lang, "기기 복구"),
        reinstall_description,
        lpm_rom_routine_ui_text(lang, "기기 복구 루틴"),
        "'The current system is not compatible',",
        "'Red state that restarts every 5 seconds',",
        lpm_rom_routine_ui_text(lang, "'펌웨어 설치 실패' 등 기기를 복구합니다."),
        Message::StartReinstallWipe,
        false,
    );

    let back_button = container(
        button(text("이전 메뉴로 이동").size(11))
            .on_press(Message::RomBackToImageInfo),
    )
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center);

    let routine_group = column![
        back_button,
        install_row,
        update_row,
        reinstall_row,
    ]
    .spacing(12)
    .width(Length::Fixed(ROM_ROUTINE_CARD_WIDTH))
    .align_x(iced::Alignment::Center);

    container(routine_group)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([42.0, 0.0])
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into()
}

fn apply_rom_option_wipe_rule(&mut self, target: RomSlideTarget) {
    let force = self
        .rom_firmware_info
        .as_ref()
        .and_then(|info| self.rom_option_force_wipe_reason(target, info));

    if self
        .rom_firmware_info
        .as_ref()
        .map(|info| target == RomSlideTarget::Install && matches!(info.region, RomRegion::Prc))
        .unwrap_or(false)
    {
        self.rom_option_country_code = None;
        self.rom_selected_country_code = None;
        self.rom_country_popup_open = false;
        self.rom_country_code_modal_open = false;
        self.rom_country_code_search.clear();
    }

    if force.is_some() {
        self.rom_option_data_wipe = true;
        self.rom_option_data_wipe_locked = true;
        return;
    }

    self.rom_option_data_wipe = false;
    self.rom_option_data_wipe_locked = false;
}

fn rom_country_code_locked_for_current_option(&self) -> bool {
    matches!(self.rom_option_target, Some(RomSlideTarget::Install))
        && self
            .rom_firmware_info
            .as_ref()
            .map(|info| matches!(info.region, RomRegion::Prc))
            .unwrap_or(false)
}

fn rom_option_force_wipe_reason(
    &self,
    target: RomSlideTarget,
    info: &FirmwareInfo,
) -> Option<&'static str> {
    if target == RomSlideTarget::Reinstall {
        return Some("안정성을 위해 데이터 초기화를 필수적으로 해야합니다.");
    }

    let original_rom = normalize_rom_label_for_option(&self.dashboard_info.original_rom);
    let installed_rom = normalize_rom_label_for_option(&self.dashboard_info.installed_rom);

    let image_rom = match info.region {
        RomRegion::Prc => "PRC",
        RomRegion::Row => "ROW",
        RomRegion::Unknown => "",
    };

    let connected_model = normalize_lenovo_model_for_compare(&self.dashboard_info.product_device);
    let image_model = normalize_lenovo_model_for_compare(&info.model);

    if target == RomSlideTarget::Install
        && model_rom_kind_from_suffix(&connected_model)
            .zip(model_rom_kind_from_suffix(&image_model))
            .map(|(device_kind, image_kind)| device_kind != image_kind)
            .unwrap_or(false)
    {
        return Some("안정성을 위해 데이터 초기화를 필수적으로 해야합니다");
    }

    if target == RomSlideTarget::Install && image_rom == "PRC" {
        return Some("PRC(중국 내수롬) 설치는 데이터 초기화가 필수입니다.");
    }

    if target == RomSlideTarget::Install && original_rom == "PRC" && image_rom == "ROW" {
        return Some("안정성을 위해 데이터 초기화를 필수적으로 해야합니다.");
    }

    if target == RomSlideTarget::Install && installed_rom == "ROW" && image_rom == "PRC" {
        return Some("안정성을 위해 데이터 초기화를 필수적으로 해야합니다.");
    }

    if original_rom == "PRC"
        && installed_rom == "ROW"
        && image_rom == "ROW"
        && firmware_version_is_lower(
            info.version.as_deref(),
            &self.dashboard_info.firmware_version,
        )
    {
        return Some("안정성을 위해 데이터 초기화를 필수적으로 해야합니다.");
    }

    None
}

fn rom_options_panel(&self, info: &FirmwareInfo) -> Element<'static, Message> {
    let lang = active_language_option();
    let target = self.rom_option_target.unwrap_or(RomSlideTarget::Install);
    let force = self.rom_option_force_wipe_reason(target, info);
    let wipe_locked = force.is_some();
    let wipe_enabled = if wipe_locked {
        true
    } else {
        self.rom_option_data_wipe
    };

    let detected_country = {
        let value = self.dashboard_info.country_code.trim();

        if value.is_empty() || value == "감지 전" || value == "알 수 없음" {
            lpm_rom_option_text(lang, "unknown").to_string()
        } else {
            value.to_string()
        }
    };

    let country_code_locked = target == RomSlideTarget::Install && matches!(info.region, RomRegion::Prc);

    let selected_country = if country_code_locked {
        lpm_rom_option_text(lang, "unavailable").to_string()
    } else {
        self.rom_option_country_code
            .clone()
            .unwrap_or_else(|| lpm_rom_option_text(lang, "not_selected").to_string())
    };

    let wipe_description = if target == RomSlideTarget::Update {
        lpm_rom_option_text(lang, "wipe_if_enabled")
    } else if wipe_locked {
        let reason = force.unwrap_or("안정성을 위해 데이터 초기화를 필수적으로 해야합니다.");
        if reason.contains("PRC") && reason.contains("설치") {
            lpm_rom_option_text(lang, "prc_install_wipe_required")
        } else {
            lpm_rom_option_text(lang, "wipe_required")
        }
    } else {
        lpm_rom_option_text(lang, "wipe_required")
    };

    let country_description = if country_code_locked {
        lpm_rom_option_text(lang, "country_locked")
    } else {
        lpm_rom_option_text(lang, "country_change")
    };

    let image_rom_label = region_label(info.region).to_string();
    let installed_rom_label = {
        let value = self.dashboard_info.installed_rom.trim();

        if value.is_empty() || value == "감지 전" || value == "알 수 없음" {
            lpm_rom_option_text(lang, "unknown").to_string()
        } else {
            value.to_string()
        }
    };
    let preflight_image_rom_text = lpm_option_rom_line(lang, true, &image_rom_label);
    let preflight_installed_rom_text = lpm_option_rom_line(lang, false, &installed_rom_label);
    let detected_country_text = format!("{}: {detected_country}", lpm_rom_option_text(lang, "detected_country"));
    let selected_country_text = format!("{}: {selected_country}", lpm_rom_option_text(lang, "selected_country"));

    let wipe_group = row![
        column![
            iced_text(lpm_rom_option_text(lang, "data_wipe_title"))
                .size(22)
                .font(lpm_bold_font())
                .wrapping(iced::widget::text::Wrapping::None),
            iced_text(wipe_description)
                .size(12)
                .wrapping(iced::widget::text::Wrapping::Word),
        ]
        .spacing(4)
        .width(Length::Fill),
        self.rom_toggle_switch(wipe_enabled, wipe_locked),
    ]
    .spacing(14)
    .width(Length::Fill)
    .align_y(iced::Alignment::Center);

    let country_button_content = container(
        iced_text(lpm_rom_option_text(lang, "select"))
            .size(12)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .width(Length::Shrink)
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center);

    let country_button = button(country_button_content)
        .width(Length::Shrink)
        .height(Length::Fixed(30.0))
        .padding([0.0, 16.0]);

    let country_button = if country_code_locked {
        country_button.style(lpm_nav_disabled_next_button_style)
    } else {
        country_button.on_press(Message::RomCountrySelect)
    };

    let country_group = row![
        column![
            iced_text(lpm_rom_option_text(lang, "country_title"))
                .size(20)
                .font(lpm_bold_font())
                .wrapping(iced::widget::text::Wrapping::None),
            iced_text(country_description)
                .size(12)
                .wrapping(iced::widget::text::Wrapping::Word),
            iced_text(detected_country_text)
                .size(11)
                .wrapping(iced::widget::text::Wrapping::Word),
        ]
        .spacing(4)
        .width(Length::Fill),
        column![
            container(country_button)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center),
            iced_text(selected_country_text)
                .size(10)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),
        ]
        .spacing(7)
        .width(Length::Fixed(150.0))
        .align_x(iced::Alignment::Center),
    ]
    .spacing(14)
    .width(Length::Fill)
    .align_y(iced::Alignment::Center);

    let preflight_group = column![
        iced_text(lpm_rom_option_text(lang, "before_start"))
            .size(18)
            .font(lpm_bold_font())
            .wrapping(iced::widget::text::Wrapping::None),
        iced_text(preflight_image_rom_text)
            .size(11)
            .wrapping(iced::widget::text::Wrapping::Word),
        iced_text(preflight_installed_rom_text)
            .size(11)
            .wrapping(iced::widget::text::Wrapping::Word),
    ]
    .spacing(4)
    .width(Length::Fill);

    let option_card = container(
        column![wipe_group, country_group, preflight_group]
            .spacing(20)
            .width(Length::Fill)
            .align_x(iced::Alignment::Center),
    )
    .width(Length::Fixed(590.0))
    .height(Length::Fixed(275.0))
    .padding([28.0, 38.0])
    .style(lpm_nav_rom_option_card_style);

    container(
        column![
            button(iced_text(lpm_rom_option_text(lang, "back")).size(11))
                .on_press(Message::RomBackToRoutineSelect),

            option_card,

            row![
                iced::widget::Space::new()
                    .width(Length::Fill)
                    .height(Length::Fixed(1.0)),

                button(iced_text(lpm_rom_option_text(lang, "continue")).size(12))
                    .on_press(Message::RomContinueFromOptions),
            ]
            .spacing(8)
            .width(Length::Fixed(590.0)),
        ]
        .spacing(12)
        .width(Length::Fill)
        .align_x(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding([22.0, 0.0])
    .align_x(iced::alignment::Horizontal::Center)
    .into()
}

fn should_show_dashboard_update_notice(&self) -> bool {
    self.active_nav == NavPage::Dashboard
        && self.dashboard_update_notice.as_ref().map(|info| info.update_available).unwrap_or(false)
}

fn dashboard_update_notice_view(&self) -> Element<'_, Message> {
    let version_label = self
        .dashboard_update_notice
        .as_ref()
        .map(|info| format!("현재 {} → 최신 {}", info.current_version, info.latest_version))
        .unwrap_or_else(|| "새로운 버전을 감지했습니다.".to_string());

    let popup_content = container(
        column![
            text("새로운 업데이트가 있습니다.")
                .size(18)
                .font(lpm_bold_font())
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::None),
            text("현재 버전의 문제를\n해결하고 업그레이드한 파일을\n감지했습니다.")
                .size(12)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),
            text(version_label)
                .size(10)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::None),
            row![
                button(
                    container(text("파일 업데이트 (권장)").size(11))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center),
                )
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(28.0))
                .padding(0.0)
                .style(lpm_nav_settings_move_button_style)
                .on_press(Message::OpenProgramUpdateRelease),
                button(
                    container(text("다음에 하기").size(11))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center),
                )
                .width(Length::Fixed(96.0))
                .height(Length::Fixed(28.0))
                .padding(0.0)
                .style(lpm_nav_settings_move_button_style)
                .on_press(Message::DismissProgramUpdateNotice),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(10)
        .width(Length::Fill)
        .align_x(iced::Alignment::Center),
    )
    .width(Length::Fixed(260.0))
    .height(Length::Fixed(150.0))
    .padding([18.0, 20.0])
    .style(lpm_nav_dashboard_update_popup_card_style);

    let scrim = container(iced::widget::Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(lpm_nav_dashboard_update_popup_scrim_style);

    let centered = container(popup_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);

    iced::widget::opaque(
        iced::widget::Stack::with_children(vec![scrim.into(), centered.into()])
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .into()
}

fn rom_country_popup_view(&self) -> Element<'_, Message> {
    let selected_code = self.rom_option_country_code.as_deref();
    let search = self.rom_country_code_search.trim().to_ascii_lowercase();
    let mut list = column![].spacing(2);
    let mut matched_count = 0usize;

    for (index, entry) in ROM_COUNTRY_CODES.iter().enumerate() {
        let entry_code = entry.code.to_ascii_lowercase();
        let localized_name = localized_country_name(index, self.settings_language);
        let entry_name = localized_name.to_lowercase();

        if !search.is_empty() && !entry_code.contains(&search) && !entry_name.contains(&search) {
            continue;
        }

        matched_count += 1;

        let selected = selected_code == Some(entry.code);
        let code = entry.code.to_string();
        let label = format!("{} — {}", entry.code, localized_name);

        list = list.push(
            button(text(label).size(13))
                .padding([6.0, 14.0])
                .width(Length::Fill)
                .on_press(Message::RomSelectCountry(code))
                .style(move |_theme: &Theme, status| {
                    let hovered = matches!(status, iced::widget::button::Status::Hovered);

                    iced::widget::button::Style {
                        background: if selected {
                            Some(Background::Color(Color::from_rgb8(73, 89, 255)))
                        } else if hovered {
                            Some(Background::Color(Color::from_rgb8(239, 241, 250)))
                        } else {
                            None
                        },
                        text_color: if selected {
                            Color::from_rgb8(255, 255, 255)
                        } else {
                            Color::from_rgb8(32, 35, 47)
                        },
                        border: iced::Border {
                            radius: 8.0.into(),
                            ..iced::Border::default()
                        },
                        ..iced::widget::button::Style::default()
                    }
                }),
        );
    }

    if matched_count == 0 {
        list = list.push(
            container(
                text("검색 결과가 없습니다.")
                    .size(13)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center),
            )
            .width(Length::Fill)
            .padding([18.0, 14.0]),
        );
    }

    let country_search_placeholder = lpm_translate_owned("국가 코드 또는 국가명 검색".to_string());
    let search_input = text_input(&country_search_placeholder, &self.rom_country_code_search)
        .on_input(Message::RomCountryCodeSearchChanged)
        .size(12)
        .padding([8.0, 10.0])
        .width(Length::Fill);

    let popup_content = container(
        column![
            row![
                text("국가 코드 변경")
                    .size(16)
                    .font(lpm_bold_font()),
                iced::widget::Space::new()
                    .width(Length::Fill)
                    .height(Length::Fixed(1.0)),
                button(text("취소").size(12))
                    .on_press(Message::RomCloseCountryPopup),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
            search_input,
            container(iced::widget::Space::new())
                .width(Length::Fill)
                .height(Length::Fixed(1.0))
                .style(lpm_nav_divider_style),
            scrollable(list)
                .width(Length::Fill)
                .height(Length::Fixed(300.0)),
        ]
        .spacing(10),
    )
    .width(Length::Fixed(420.0))
    .padding(20)
    .style(lpm_nav_rom_country_popup_card_style);

    let scrim = container(iced::widget::Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(lpm_nav_rom_country_popup_scrim_style);

    let centered = container(popup_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);

    iced::widget::opaque(
        iced::widget::Stack::with_children(vec![scrim.into(), centered.into()])
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .into()
}

fn rom_should_show_vc_runtime_popup(&self) -> bool {
    self.active_nav == NavPage::Rom
        && self.image_dir.is_some()
        && self.rom_firmware_info.is_some()
        && !self.rom_show_routine_select
        && self.rom_option_target.is_none()
        && !self.busy
        && (self.rom_vc_runtime_error.is_some()
            || self
                .rom_vc_runtime_status
                .map(|status| !status.all_installed())
                .unwrap_or(false))
}

fn rom_vc_runtime_popup_view(&self) -> Element<'_, Message> {
    let popup_content = container(
        column![
            text("Microsoft Visual C++ 런타임 설치가 필요합니다!")
                .size(17)
                .font(lpm_bold_font())
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),

            text("VCRUNTIME140.dll / MSVCP140.dll 오류를 방지하기 위해\nMicrosoft Visual C++ x86(32비트) 및 x64(64비트) 런타임을 설치해주세요.")
                .size(13)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),

            button(text("Visual C++ 설치").size(12))
                .padding([8.0, 18.0])
                .on_press(Message::InstallVcRuntime),
        ]
        .spacing(14)
        .width(Length::Fill)
        .align_x(iced::Alignment::Center),
    )
    .width(Length::Fixed(390.0))
    .height(Length::Fixed(196.0))
    .padding([24.0, 28.0])
    .style(lpm_nav_mtk_driver_popup_card_style);

    let scrim = container(iced::widget::Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(lpm_nav_mtk_driver_popup_scrim_style);

    let centered = container(popup_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);

    iced::widget::opaque(
        iced::widget::Stack::with_children(vec![scrim.into(), centered.into()])
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .into()
}

fn rom_should_show_mtk_driver_popup(&self) -> bool {
    self.active_nav == NavPage::Rom
        && self.image_dir.is_some()
        && self.rom_firmware_info.is_some()
        && !self.rom_show_routine_select
        && self.rom_option_target.is_none()
        && !self.busy
        && !(self.rom_vc_runtime_error.is_some()
            || self
                .rom_vc_runtime_status
                .map(|status| !status.all_installed())
                .unwrap_or(false))
        && self.rom_mtk_driver_installed == Some(false)
}

fn rom_mtk_driver_popup_view(&self) -> Element<'_, Message> {
    let popup_content = container(
        column![
            text("MTK 드라이버 설치가 필요합니다!")
                .size(17)
                .font(lpm_bold_font())
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::None),

            text("LPMBOX를 사용하기 위해선\n반드시 설치가 필요합니다\n드라이버 설치를 해주세요.")
                .size(13)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),

            button(text("드라이버 설치").size(12))
                .padding([8.0, 18.0])
                .on_press(Message::InstallMtkDriver),
        ]
        .spacing(14)
        .width(Length::Fill)
        .align_x(iced::Alignment::Center),
    )
    .width(Length::Fixed(310.0))
    .height(Length::Fixed(178.0))
    .padding([24.0, 28.0])
    .style(lpm_nav_mtk_driver_popup_card_style);

    let scrim = container(iced::widget::Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(lpm_nav_mtk_driver_popup_scrim_style);

    let centered = container(popup_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);

    iced::widget::opaque(
        iced::widget::Stack::with_children(vec![scrim.into(), centered.into()])
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .into()
}

fn rom_toggle_switch(&self, enabled: bool, locked: bool) -> Element<'static, Message> {
    let knob = container(iced::widget::Space::new())
        .width(Length::Fixed(22.0))
        .height(Length::Fixed(22.0))
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
            border: iced::Border {
                radius: 11.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..container::Style::default()
        });

    let left_space = iced::widget::Space::new()
        .width(Length::Fixed(22.0))
        .height(Length::Fixed(1.0));

    let right_space = iced::widget::Space::new()
        .width(Length::Fixed(22.0))
        .height(Length::Fixed(1.0));

    let toggle_inner = if enabled {
        row![left_space, knob]
    } else {
        row![knob, right_space]
    }
    .spacing(0)
    .align_y(iced::Alignment::Center);

    let track = container(toggle_inner)
        .width(Length::Fixed(50.0))
        .height(Length::Fixed(28.0))
        .padding(3)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(move |_theme: &Theme| {
            let background = if enabled {
                Color::from_rgb8(23, 64, 232)
            } else {
                Color::from_rgb8(218, 222, 238)
            };

            container::Style {
                background: Some(Background::Color(background)),
                border: iced::Border {
                    radius: 14.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                ..container::Style::default()
            }
        });

    if locked {
        track.into()
    } else {
        button(track)
            .padding(0)
            .on_press(Message::RomToggleDataWipe)
            .style(lpm_nav_flat_button_style)
            .into()
    }
}

fn rom_routine_slide_row(
    &self,
    icon_handle: iced::widget::image::Handle,
    slide_icon_handle: iced::widget::image::Handle,
    target: RomSlideTarget,
    title: &'static str,
    description: &'static str,
    slide_title: &'static str,
    slide_line_1: &'static str,
    slide_line_2: &'static str,
    slide_line_3: &'static str,
    message: Message,
    disabled: bool,
) -> Element<'static, Message> {
    let slide_width = match target {
        RomSlideTarget::Install => self.rom_install_slide_width,
        RomSlideTarget::Update => self.rom_update_slide_width,
        RomSlideTarget::Reinstall => self.rom_reinstall_slide_width,
    };

    let any_progress = (slide_width / ROM_ROUTINE_EXPAND_WIDTH).clamp(0.0, 1.0);

    let icon_area = container(
        iced::widget::image(icon_handle.clone())
            .width(Length::Fixed(75.0))
            .height(Length::Fixed(75.0)),
    )
    .width(Length::Fixed(92.0))
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center);

    let divider = container(iced::widget::Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fixed(58.0))
        .style(lpm_nav_divider_style);

    let text_area = column![
        iced_text(title)
            .size(lpm_routine_title_size())
            .font(lpm_bold_font())
            .wrapping(iced::widget::text::Wrapping::Word),
        iced_text(description)
            .size(lpm_routine_body_size())
            .wrapping(iced::widget::text::Wrapping::Word),
    ]
    .spacing(4)
    .width(Length::Fill);

let base_row = container(
    row![icon_area, divider, text_area]
        .spacing(18)
        .align_y(iced::Alignment::Center),
)
.width(Length::Fill)
.height(Length::Fill)
.padding(iced::Padding {
    top: 0.0,
    right: ROM_ROUTINE_HANDLE_WIDTH + 18.0,
    bottom: 0.0,
    left: 16.0,
})
.style(move |theme: &Theme| {
    if disabled {
        lpm_nav_rom_routine_row_disabled_style(theme)
    } else {
        lpm_nav_rom_routine_row_style(theme)
    }
});

    let dim_overlay = container(iced::widget::Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgba(
                222.0 / 255.0,
                225.0 / 255.0,
                236.0 / 255.0,
                ROM_DIM_ALPHA * any_progress,
            ))),
            border: iced::Border {
                radius: 14.0.into(),
                ..iced::Border::default()
            },
            ..container::Style::default()
        });

let slide_clip: Element<'static, Message> = if slide_width > 0.5 {
    let slide_text_column = column![
        iced_text(slide_title)
            .size(lpm_slide_title_size())
            .font(lpm_bold_font())
            .wrapping(iced::widget::text::Wrapping::Word),
        iced_text(slide_line_1)
            .size(lpm_slide_body_size())
            .wrapping(iced::widget::text::Wrapping::Word),
        iced_text(slide_line_2)
            .size(lpm_slide_body_size())
            .wrapping(iced::widget::text::Wrapping::Word),
        iced_text(slide_line_3)
            .size(lpm_slide_body_size())
            .wrapping(iced::widget::text::Wrapping::Word),
    ]
    .spacing(2)
    .width(Length::Fixed(ROM_ROUTINE_SLIDE_TEXT_WIDTH));

    let slide_warning_icon = container(
        iced::widget::image(slide_icon_handle)
            .width(Length::Fixed(42.0))
            .height(Length::Fixed(42.0)),
    )
    .width(Length::Fixed(54.0))
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center);

    let slide_content_fixed = container(
        row![slide_warning_icon, slide_text_column]
            .spacing(8)
            .align_y(iced::Alignment::Center),
    )
    .width(Length::Fixed(ROM_ROUTINE_EXPAND_WIDTH))
    .height(Length::Fill)
    .padding(12)
    .style(lpm_nav_rom_routine_slide_style);

    container(slide_content_fixed)
        .width(Length::Fixed(slide_width))
        .height(Length::Fixed(ROM_ROUTINE_CARD_HEIGHT))
        .clip(true)
        .into()
} else {
    iced::widget::Space::new()
        .width(Length::Fixed(0.0))
        .height(Length::Fixed(ROM_ROUTINE_CARD_HEIGHT))
        .into()
};

let slide_handle_area = container(
    iced::widget::image(self.slide_button_handle.clone())
        .width(Length::Fixed(ROM_ROUTINE_HANDLE_WIDTH))
        .height(Length::Fixed(ROM_ROUTINE_HANDLE_HEIGHT)),
)
.width(Length::Fixed(ROM_ROUTINE_HANDLE_WIDTH))
.height(Length::Fill)
.padding(0)
.align_x(iced::alignment::Horizontal::Center)
.align_y(iced::alignment::Vertical::Center);

let slide_overlay_content = row![slide_clip, slide_handle_area]
    .spacing(0)
    .height(Length::Fixed(ROM_ROUTINE_CARD_HEIGHT))
    .align_y(iced::Alignment::Center);

let slide_overlay = container(slide_overlay_content)
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(iced::Padding {
        top: 0.0,
        right: ROM_ROUTINE_HANDLE_RIGHT_PADDING,
        bottom: 0.0,
        left: 0.0,
    })
    .align_x(iced::alignment::Horizontal::Right)
    .align_y(iced::alignment::Vertical::Center);

    let mut stack_children: Vec<Element<'static, Message>> = vec![base_row.into()];

    if any_progress > 0.01 {
        stack_children.push(dim_overlay.into());
    }

    stack_children.push(slide_overlay.into());

    let stacked = iced::widget::Stack::with_children(stack_children)
        .width(Length::Fixed(ROM_ROUTINE_CARD_WIDTH))
        .height(Length::Fixed(ROM_ROUTINE_CARD_HEIGHT));

let mut card_button = button(stacked)
    .padding(0)
    .width(Length::Fixed(ROM_ROUTINE_CARD_WIDTH))
    .height(Length::Fixed(ROM_ROUTINE_CARD_HEIGHT))
    .style(lpm_nav_rom_card_button_style);

if !disabled {
    card_button = card_button.on_press(message);
}

let mouse_area = iced::widget::mouse_area(card_button);

if disabled {
    mouse_area.into()
} else {
    mouse_area
        .on_enter(Message::RomCardHoverEnter(target))
        .on_exit(Message::RomCardHoverExit(target))
        .into()
}
}

fn dashboard_info_item(&self, title: &str, value: &str) -> Element<'static, Message> {
    let title_text = lpm_translate_owned(title.to_string());

    let value_text = if value.trim().is_empty() || value.trim() == "감지 전" {
        lpm_translate_owned("알 수 없음".to_string())
    } else {
        lpm_translate_owned(value.trim().to_string())
    };

    let (title_size, value_size) = if matches!(active_language_option(), LanguageOption::Arabic) {
        (10, 10)
    } else {
        (12, 12)
    };

    container(
        column![
            iced_text(title_text)
                .size(title_size)
                .font(lpm_bold_font())
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),
            iced_text(value_text)
                .size(value_size)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),
        ]
        .spacing(1),
    )
    .width(Length::Fill)
    .into()
}

fn dashboard_action_card(
    &self,
    title: &'static str,
    description: &'static str,
    button_label: &'static str,
    message: Message,
) -> Element<'static, Message> {
    const DASHBOARD_ACTION_CARD_HEIGHT: f32 = 184.0;
    const DASHBOARD_ACTION_TITLE_HEIGHT: f32 = 38.0;
    const DASHBOARD_ACTION_DESCRIPTION_HEIGHT: f32 = 76.0;
    const DASHBOARD_ACTION_BUTTON_HEIGHT: f32 = 30.0;

    let action_button = button(
        container(text(button_label).size(lpm_dashboard_body_size()))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .width(Length::Fixed(104.0))
    .height(Length::Fixed(DASHBOARD_ACTION_BUTTON_HEIGHT))
    .padding(0.0)
    .on_press(message);

    let content = column![
        container(
            text(title)
                .size(lpm_dashboard_title_size())
                .font(lpm_bold_font())
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),
        )
        .width(Length::Fill)
        .height(Length::Fixed(DASHBOARD_ACTION_TITLE_HEIGHT))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
        container(
            text(description)
                .size(lpm_dashboard_description_size())
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word),
        )
        .width(Length::Fill)
        .height(Length::Fixed(DASHBOARD_ACTION_DESCRIPTION_HEIGHT))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
        iced::widget::Space::new()
            .width(Length::Fill)
            .height(Length::Fill),
        container(action_button)
            .width(Length::Fill)
            .height(Length::Fixed(DASHBOARD_ACTION_BUTTON_HEIGHT))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    ]
    .spacing(4)
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(iced::Alignment::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fixed(DASHBOARD_ACTION_CARD_HEIGHT))
        .padding([10.0, 10.0])
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(lpm_nav_dashboard_action_style)
        .into()
}

fn dashboard_model_image_handle(&self) -> iced::widget::image::Handle {
    match self.dashboard_model_image {
        DashboardModelImage::Tb365 => self.model_tb365_handle.clone(),
        DashboardModelImage::Tb335 => self.model_tb335_handle.clone(),
        DashboardModelImage::Tb375 | DashboardModelImage::Unknown => {
            self.model_tb375_handle.clone()
        }
    }
}

fn dashboard_model_image_from_info(
    info: &lpmbox_device::DashboardDeviceInfo,
) -> DashboardModelImage {
    let model = info.model_name.to_uppercase();

    if model.contains("TB375FC") || model.contains("TB373FU") {
        DashboardModelImage::Tb375
    } else if model.contains("TB365FC") || model.contains("TB361FU") {
        DashboardModelImage::Tb365
    } else if model.contains("TB335FC") || model.contains("TB336FU") {
        DashboardModelImage::Tb335
    } else {
        DashboardModelImage::Unknown
    }
}

fn return_to_rom_image_info_after_device_state_changed(
    &mut self,
    was_device_unknown: bool,
    now_device_unknown: bool,
) {
    if self.active_nav != NavPage::Rom
        || self.image_dir.is_none()
        || self.rom_firmware_info.is_none()
        || (!self.rom_show_routine_select && self.rom_option_target.is_none())
    {
        return;
    }

    if was_device_unknown == now_device_unknown {
        return;
    }

    self.rom_show_routine_select = false;
    self.rom_option_target = None;
    self.running_rom_target = None;
    self.rom_country_popup_open = false;
    self.rom_country_code_modal_open = false;
    self.rom_country_code_search.clear();
    self.reset_rom_routine_slide_state();
    self.reset_rom_check_loading_stack();

    if now_device_unknown {
        self.push_log("[ROM] 기기 연결이 끊겼습니다. 선택한 image 폴더 정보 화면으로 돌아갑니다.");
        self.push_log("[ROM] PC 후면 연결(노트북은 상관 없음), 올바른 데이터 케이블을 사용해주세요. (QnA 참고)");
    } else {
        self.push_log("[ROM] 기기가 다시 감지되었습니다.");
    }
}


    fn start_country_reset_flow(&mut self, selected_country_code: String) -> Task<Message> {
        if self.busy {
            self.push_log("이미 작업이 진행 중입니다.");
            return Task::none();
        }

        let Some(image_dir) = self.image_dir.clone() else {
            self.active_nav = NavPage::Log;
            self.push_log("[국가 코드 재설정] 먼저 ROM 작업에서 image 폴더를 선택해주세요.");
            self.push_log("[국가 코드 재설정] proinfo 파티션만 플래싱하려면 선택한 image 폴더의 flash.xml, scatter, DA 파일이 필요합니다.");
            return Task::none();
        };

        self.active_nav = NavPage::Log;
        self.running_rom_target = None;
        self.busy = true;
        self.progress_line_indices.clear();
        self.active_spinners.clear();
        self.spinner_tick = 0;
        self.last_spft_stage = None;
        self.active_log_flow = Some(RuntimeFlowKind::CountryReset);
        self.push_log(format!(
            "[국가 코드 재설정] 국가 코드 재설정 작업을 시작합니다: {selected_country_code}"
        ));

        let (tx, rx) = mpsc::channel::<ProinfoLiveEvent>();
        self.live_rx = Some(rx);

        thread::spawn(move || {
            run_country_reset_flow(image_dir, selected_country_code, tx);
        });

        Task::none()
    }

    fn cleanup_before_process_exit(&mut self) {
        self.live_rx = None;
        self.busy = false;
        self.active_log_flow = None;
        self.progress_line_indices.clear();
        self.active_spinners.clear();
        cleanup_external_processes_on_exit();
    }

    fn push_log(&mut self, message: impl Into<String>) {
        let message = message.into();
        self.log_lines.push(LogLine {
            timestamp: current_timestamp(),
            message,
        });
        self.refresh_log_content();
    }

        fn export_log_to_file(&mut self) -> std::io::Result<PathBuf> {
            let log_dir = lpmbox_core::app_paths::log_dir();
            std::fs::create_dir_all(&log_dir)?;

            let file_name = format!("LPMBox_log_{}.txt", Local::now().format("%Y-%m-%d_%H-%M"));
            let log_path = log_dir.join(file_name);

            self.push_log(format!(
                "[Log] 텍스트 파일을 {}에 저장합니다.",
                log_path.display()
            ));

            std::fs::write(&log_path, build_log_export_text(&self.log_lines))?;

            Ok(log_path)
        }

        fn save_completed_flow_log_to_file(&mut self, flow: RuntimeFlowKind) -> std::io::Result<PathBuf> {
            let log_dir = lpmbox_core::app_paths::log_dir();
            std::fs::create_dir_all(&log_dir)?;

            let base_name = format!(
                "LPMBox_{}_{}",
                flow.file_stem(),
                Local::now().format("%Y-%m-%d_%H-%M")
            );

            let mut log_path = log_dir.join(format!("{base_name}.txt"));
            let mut duplicate_index = 2usize;

            while log_path.exists() {
                log_path = log_dir.join(format!("{base_name}_{duplicate_index}.txt"));
                duplicate_index += 1;
            }

            self.push_log(format!(
                "[Log] {} 작업 로그를 {}에 저장합니다.",
                flow.label(),
                log_path.display()
            ));

            std::fs::write(&log_path, build_log_export_text(&self.log_lines))?;

            Ok(log_path)
        }

        fn push_common_failure_guidance(&mut self) {
            self.push_log("[Error] 1) 데이터 케이블을 PC 후면에 연결한 뒤 다시 시도해주세요. (노트북은 상관 없음)");
            self.push_log("[Error] 2) LPMBOX QnA에 설명드린 케이블로 시도해주세요.");
            self.push_log("[Error] 3) 케이블 PC 후면 연결, 설명한 케이블로 시도했음에도 실패할 경우 다른 PC 또는 노트북으로 시도해주세요.");
        }

        fn refresh_log_content(&mut self) {
            self.log_cache_dirty = true;

            if !self.log_refresh_suspended {
                self.flush_log_content_if_needed();
            }
        }

        fn flush_log_content_if_needed(&mut self) {
            if !self.log_cache_dirty {
                return;
            }

            self.log_text_cache = build_log_text(&self.log_lines);
            self.log_display_rows_cache = build_log_display_rows(&self.log_lines);
            self.log_cache_dirty = false;
        }

    fn start_mtk_driver_package_prepare_if_needed(&mut self) {
        if self.live_rx.is_some() {
            return;
        }

        self.push_log("[Driver] MTK 드라이버 설치 파일 준비를 시작합니다.");

        let (tx, rx) = mpsc::channel::<ProinfoLiveEvent>();
        self.live_rx = Some(rx);

        thread::spawn(move || {
            run_mtk_driver_prepare_flow(tx);
        });
    }

    fn handle_live_event(&mut self, event: ProinfoLiveEvent) -> bool {
    match event {
ProinfoLiveEvent::Log(message) => {
    if let Some((key, message)) = parse_spinner_log(&message) {
        self.push_or_update_spinner_log(&key, message);
        return false;
    }

    if message.contains("[Driver] MTK 드라이버 설치 작업이 완료되었습니다.")
        || message.contains("[Driver] MTK 드라이버 설치 감지 완료")
    {
        self.rom_mtk_driver_installed = Some(true);
        self.rom_mtk_driver_error = None;
    }

    if message.contains("[Runtime] Microsoft Visual C++ 런타임 설치 작업이 완료되었습니다.")
        || message.contains("[Runtime] Microsoft Visual C++ x86/x64 런타임 설치 감지 완료")
    {
        self.rom_vc_runtime_status = Some(lpmbox_device::VcRuntimeStatus {
            x86_installed: true,
            x64_installed: true,
        });
        self.rom_vc_runtime_error = None;
    }

    let flow_completed = is_runtime_flow_completion_log(&message);

    if message.contains("[완료]")
                || message.contains("SPFlashToolV6 작업 완료")
                || message.contains("All command exec done")
            {
                self.finalize_last_spft_stage();
            }

            if let Some(message) = normalize_log_message(&message) {
                self.push_log(message);
            }

            flow_completed
        }

        ProinfoLiveEvent::Progress(progress) => {
            self.push_or_update_progress_log(&progress);
            false
        }

        ProinfoLiveEvent::Finished(result) => {
            self.push_proinfo_backup_finished_logs(&result);
            true
        }

        ProinfoLiveEvent::Error(err) => {
            let should_show_failure_guidance = self.active_log_flow.is_some();

            if is_usb_debugging_shell_error(&err) {
                self.push_log("작업 오류: USB 디버깅 활성화 설정 후 다시 시도해주세요.");
            } else if err.contains("Fastboot 기기 감지 실패")
                || err.contains("Fastboot 기기 감지 시간 초과")
            {
                self.push_log("작업 오류: Fastboot 기기 감지 실패");
            } else if err.contains("NO_USB_ADB_DEVICE")
                || err.contains("USB ADB 기기가 감지되지 않았습니다")
                || err.contains("태블릿 확인에 실패했습니다")
            {
                self.push_log(
                    "[경고] 작업 오류: 태블릿 확인에 실패했습니다, 올바른 데이터 케이블을 사용해주세요.",
                );
            } else if err.contains("ADB_UNAUTHORIZED_RETRY")
                || err.contains("ADB unauthorized")
                || err.contains("unauthorized")
            {

            } else {
                self.push_log(format!("작업 오류: {err}"));
            }

            if should_show_failure_guidance {
                self.push_common_failure_guidance();
            }

            true
        }
    }
}

    fn push_or_update_spinner_log(&mut self, key: &str, message: String) {
        let key = format!("spinner:{key}");
        let (base_message, active) = split_spinner_message(&message);

        if let Some(index) = self.progress_line_indices.get(&key).copied() {
            if let Some(line) = self.log_lines.get_mut(index) {
                line.timestamp = current_timestamp();
                line.message = message;
            }

            if active {
                self.active_spinners.insert(
                    key,
                    SpinnerState {
                        line_index: index,
                        base_message,
                    },
                );
            } else {
                self.active_spinners.remove(&key);
            }

            self.refresh_log_content();
            return;
        }

        let index = self.log_lines.len();

        self.log_lines.push(LogLine {
            timestamp: current_timestamp(),
            message,
        });

        self.progress_line_indices.insert(key.clone(), index);

        if active {
            self.active_spinners.insert(
                key,
                SpinnerState {
                    line_index: index,
                    base_message,
                },
            );
        }

        self.refresh_log_content();
    }

    fn tick_active_spinners(&mut self) {
        if self.active_spinners.is_empty() {
            return;
        }

        self.spinner_tick = self.spinner_tick.wrapping_add(1);
        let frame = UI_SPINNER_FRAMES[self.spinner_tick % UI_SPINNER_FRAMES.len()];

        let states = self
            .active_spinners
            .values()
            .cloned()
            .collect::<Vec<SpinnerState>>();

        for state in states {
            if let Some(line) = self.log_lines.get_mut(state.line_index) {
                line.timestamp = current_timestamp();
                line.message = format!("{} {}", state.base_message, frame);
            }
        }

        self.refresh_log_content();
    }

            fn push_or_update_progress_log(&mut self, progress: &SpftProgress) {
            if let Some(previous_stage) = self.last_spft_stage.clone() {
                if previous_stage != progress.stage {
                    self.finalize_spft_stage(&previous_stage);
                }
            }

            self.last_spft_stage = Some(progress.stage.clone());

            let key = format!("spft:{}", progress.stage);
            let stage_label = spft_stage_label(&progress.stage);

            let message = format!(
                "[SPFT] {} {}",
                stage_label,
                progress_bar(progress.percent)
            );

            if let Some(index) = self.progress_line_indices.get(&key).copied() {
                if let Some(line) = self.log_lines.get_mut(index) {
                    line.timestamp = current_timestamp();
                    line.message = message;
                    self.refresh_log_content();
                    return;
                }
            }

            let index = self.log_lines.len();
            self.log_lines.push(LogLine {
                timestamp: current_timestamp(),
                message,
            });

            self.progress_line_indices.insert(key, index);

           self.refresh_log_content();
        }

fn finalize_last_spft_stage(&mut self) {
    if let Some(stage) = self.last_spft_stage.clone() {
        self.finalize_spft_stage(&stage);
    }
}

fn finalize_spft_stage(&mut self, stage: &str) {
    let key = format!("spft:{stage}");

    let Some(index) = self.progress_line_indices.get(&key).copied() else {
        return;
    };

    let stage_label = spft_stage_label(stage);
    let message = format!("[SPFT] {} {}", stage_label, progress_bar(100));

    if let Some(line) = self.log_lines.get_mut(index) {
        line.timestamp = current_timestamp();
        line.message = message;
    }

    self.refresh_log_content();
}

    fn push_blocked_firmware_check_logs(&mut self, check: &BlockedFirmwareCheck) {
        self.push_log(format!("온라인 차단 펌웨어 규칙 검사: {}", check.message));

        if !check.blocked_versions.is_empty() {
            self.push_log(format!(
                "차단 버전 목록 / {}: {}",
                check.model,
                check.blocked_versions.join(", ")
            ));
        }

        if check.blocked {
            self.push_log(
                "[!] 차단 펌웨어 감지: 설치/업데이트 실행 단계에서 이 펌웨어는 차단됩니다.",
            );
        }
    }

    fn push_firmware_result_logs(&mut self, info: &FirmwareInfo) {
        self.push_log("펌웨어 검사 완료");
        self.push_log(format!("[Image] 모델명: {}", info.model));

        if let Some(version) = &info.version {
            self.push_log(format!("[Image] 버전: {version}"));
        } else {
            self.push_log("[Image] 버전: 감지 실패");
        }

        self.push_log(format!("[Image] ROM 타입: {}", region_label(info.region)));

        if let Some(platform) = &info.platform {
            self.push_log(format!("[Image] 플랫폼: {platform}"));
        } else {
            self.push_log("[Image] 플랫폼: 감지 실패");
        }

        self.push_blocked_firmware_check_logs(&info.blocked_firmware_check);

        let Some(scatter_xml_info) = &info.scatter_xml_info else {
            self.push_log("scatter XML 파싱: 확인 불가");
            self.push_log("partition 목록 읽기: 확인 불가");
            self.push_log("partition 상세 정보 읽기: 확인 불가");
            self.push_log("필수 partition 검사: 확인 불가");
            self.push_log("patch plan 생성: 확인 불가");
            self.push_log("patch plan 적용: 확인 불가");
            self.push_log("patch 결과 재검증: 확인 불가");
            return;
        };

        self.push_log(format!(
            "scatter XML 파싱: 성공 / root: {} / size: {} bytes",
            scatter_xml_info.root_name, scatter_xml_info.xml_size
        ));

        self.push_log(format!(
            "partition 목록 읽기: 성공 / {}개",
            scatter_xml_info.partition_count
        ));

        self.push_log(format!(
            "partition 상세 정보 읽기: 성공 / {}개",
            scatter_xml_info.partitions.len()
        ));

        self.push_log(format!(
            "필수 partition 검사: {}",
            required_check_label(&scatter_xml_info.required_check)
        ));

        self.push_log(format!(
            "필수 partition 상세: {}",
            required_check_detail(&scatter_xml_info.required_check)
        ));

        self.push_log(format!(
            "patch plan 생성: 성공 / {}개",
            scatter_xml_info.patch_plans.len()
        ));

        self.push_log(format!(
            "patch plan 미리보기: {}",
            preview_patch_plans(&scatter_xml_info.patch_plans)
        ));

        self.push_log(format!(
            "patch plan 적용: 성공 / {}개",
            scatter_xml_info.patched_snapshots.len()
        ));

        self.push_log(format!(
            "patch 적용 미리보기: {}",
            preview_patched_snapshots(&scatter_xml_info.patched_snapshots)
        ));

        self.push_log(format!(
            "patch 결과 재검증: {}",
            patch_validation_summary(&scatter_xml_info.patch_validations)
        ));

        self.push_log(format!(
            "patch 결과 재검증 미리보기: {}",
            preview_patch_validations(&scatter_xml_info.patch_validations)
        ));

        for validation in &scatter_xml_info.patch_validations {
            for error in &validation.errors {
                self.push_log(format!("검증 오류 / {}: {error}", validation.title));
            }

            for warning in &validation.warnings {
                self.push_log(format!("검증 경고 / {}: {warning}", validation.title));
            }
        }
    }

    fn push_proinfo_backup_finished_logs(&mut self, result: &ProinfoReadbackResult) {
        let plan = &result.plan;

        self.push_log(format!("Root: {}", plan.root_dir.display()));
        self.push_log(format!("image 폴더: {}", plan.image_dir.display()));
        self.push_log(format!(
            "ROOT 경로에 SPFlashToolV6 준비 완료: {}",
            plan.embedded_tool_dir.display()
        ));
        self.push_log(format!("backup 폴더: {}", plan.backup_dir.display()));
        self.push_log(format!("proinfo 출력 경로: {}", plan.proinfo_out.display()));

        if result.proinfo_exists && result.proinfo_size > 0 {
            self.push_log(format!(
                "proinfo 파티션 추출 성공: {} / {} bytes",
                plan.proinfo_out.display(),
                result.proinfo_size
            ));
        } else {
            self.push_log(
                "proinfo 백업 실패: backup\\proinfo 파일이 생성되지 않았거나 크기가 0입니다.",
            );
        }

        self.push_log("[3단계] 태블릿 재부팅 중...");
    }
}


impl Drop for App {
    fn drop(&mut self) {
        cleanup_external_processes_on_exit();
    }
}

fn cleanup_external_processes_on_exit() {
    lpmbox_device::terminate_adb_fastboot_processes();
    try_kill_spflashtoolv6_for_file_unlock();
}

fn ensure_block_firmware_rules_for_flow(tx: &mpsc::Sender<ProinfoLiveEvent>) -> bool {
    let source = lpmbox_firmware::block_firmware_rules_url();
    let _ = tx.send(ProinfoLiveEvent::Log(format!(
        "[Image] 온라인 차단 펌웨어 규칙을 확인합니다: {source}"
    )));

    match lpmbox_firmware::refresh_block_firmware_rules() {
        Ok(size) => {
            let _ = tx.send(ProinfoLiveEvent::Log(format!(
                "[Image] 온라인 차단 펌웨어 규칙 확인 완료: {size} bytes"
            )));
            true
        }
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "온라인 차단 펌웨어 규칙 확인 실패: {err}"
            )));
            false
        }
    }
}

fn run_convert_wipe_flow(
    image_dir: PathBuf,
    selected_country_code: Option<String>,
    tx: mpsc::Sender<ProinfoLiveEvent>,
) {
    if !ensure_block_firmware_rules_for_flow(&tx) {
        return;
    }

    if !ensure_mtk_driver_installed_for_flow(&tx) {
        return;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[1단계] ADB 기기 감지 및 기기 정보 확인".to_string(),
    ));

    let device = match lpmbox_device::probe_adb_device_for_convert_wipe(|message| {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    }) {
        Ok(device) => device,
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "ADB 기기 확인 실패: {err}"
            )));
            return;
        }
    };

        let _ = tx.send(ProinfoLiveEvent::Log(
            "[2단계] image 폴더 검사 및 플래싱 준비".to_string(),
        ));

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[2단계] image 폴더 기본 검사".to_string(),
    ));

    let firmware = match lpmbox_firmware::inspect_firmware(&image_dir) {
        Ok(firmware) => firmware,
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!("image 폴더 검사 실패: {err}")));
            return;
        }
    };

    let route_message = match validate_device_and_firmware_for_convert_wipe(&device, &firmware) {
        Ok(message) => message,
        Err(message) => {
            let _ = tx.send(ProinfoLiveEvent::Log(message));
            return;
        }
    };

    let _ = tx.send(ProinfoLiveEvent::Log(route_message));

    let country_code = selected_country_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if !run_optional_country_proinfo_backup(&image_dir, country_code, &tx) {
        return;
    }

    if country_code.is_some() {
        remove_spft_log_folder_after_proinfo_backup(&tx);
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[3단계] Flash Plan 준비 및 작업용 scatter/xml 생성".to_string(),
    ));

    let output = match lpmbox_firmware::prepare_flash_plan_with_country_code_log(
        &image_dir,
        InstallMode::ConvertWipe,
        country_code,
        |message| {
            let _ = tx.send(ProinfoLiveEvent::Log(message));
        },
    ) {
        Ok(output) => output,
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!("플래싱 준비 실패: {err}")));
            return;
        }
    };

    for message in flash_prepared_log_messages(&output) {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[4단계] current slot A 설정".to_string(),
    ));

    if let Err(err) = lpmbox_device::run_current_slot_a_stage(|message| {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    }) {
        let err_text = err.to_string();

        if err_text.contains("Fastboot 기기 감지 실패")
            || err_text.contains("Fastboot 기기 감지 시간 초과")
        {
            let _ = tx.send(ProinfoLiveEvent::Error(
                "Fastboot 기기 감지 실패".to_string(),
            ));
        } else {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "current slot A 설정 실패: {err_text}"
            )));
        }

        return;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[5단계] ROM 설치".to_string(),
    ));

    lpmbox_device::trigger_rom_install_reboot_commands(|message| {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    });

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[6단계] MediaTek PreLoader 포트 감지".to_string(),
    ));

    let _ = tx.send(ProinfoLiveEvent::Log(
        internal_spinner_message("preloader_detect", "[Port] PreLoader 포트 감지 중... │"),
    ));

    let detect = lpmbox_device::detect_preloader_until_timeout(30);

    if !detect.detected {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "PreLoader 포트 감지 실패: 30초 안에 MediaTek PreLoader USB VCOM 포트를 찾지 못했습니다. 감지 문자열: {}",
            detect.checked_tokens.join(" / ")
        )));
        return;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        internal_spinner_message("preloader_detect", "[Port] PreLoader 포트 감지 완료"),
    ));

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[7단계] SPFlashToolV6 ROM 설치".to_string(),
    ));

    let result = lpmbox_spft::execute_firmware_download_streaming(&output.plan, |event| {
        let _ = tx.send(event);
    });

    match result {
        Ok(()) => {
            let _ = tx.send(ProinfoLiveEvent::Log(
                "[완료] 1번 옵션 PRC/ROW 펌웨어 설치 작업이 완료되었습니다.".to_string(),
            ));
        }
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "SPFlashToolV6 download 실패: {err}"
            )));
        }
    }
}

fn run_row_update_keep_data_flow(
    image_dir: PathBuf,
    selected_country_code: Option<String>,
    tx: mpsc::Sender<ProinfoLiveEvent>,
) {
    if !ensure_block_firmware_rules_for_flow(&tx) {
        return;
    }

    if !ensure_mtk_driver_installed_for_flow(&tx) {
        return;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[1단계] ADB 기기 감지 및 기기 정보 확인".to_string(),
    ));

    let device = match lpmbox_device::probe_adb_device_for_convert_wipe(|message| {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    }) {
        Ok(device) => device,
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "ADB 기기 확인 실패: {err}"
            )));
            return;
        }
    };

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[2단계] image 폴더 기본 검사".to_string(),
    ));

    let firmware = match lpmbox_firmware::inspect_firmware(&image_dir) {
        Ok(firmware) => firmware,
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!("image 폴더 검사 실패: {err}")));
            return;
        }
    };

    let route_message = match validate_device_and_firmware_for_row_update_keep_data(&device, &firmware) {
        Ok(message) => message,
        Err(message) => {
            let _ = tx.send(ProinfoLiveEvent::Log(message));
            return;
        }
    };

    let _ = tx.send(ProinfoLiveEvent::Log(route_message));

    let country_code = selected_country_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if !run_optional_country_proinfo_backup(&image_dir, country_code, &tx) {
        return;
    }

    if country_code.is_some() {
        remove_spft_log_folder_after_proinfo_backup(&tx);
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[3단계] Flash Plan 준비 및 작업용 scatter/xml 생성".to_string(),
    ));

    let output = match lpmbox_firmware::prepare_flash_plan_with_country_code_log(
        &image_dir,
        InstallMode::RowUpdateKeepData,
        country_code,
        |message| {
            let _ = tx.send(ProinfoLiveEvent::Log(message));
        },
    ) {
        Ok(output) => output,
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!("플래싱 준비 실패: {err}")));
            return;
        }
    };

    for message in flash_prepared_log_messages(&output) {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[4단계] current slot A 설정".to_string(),
    ));

    if let Err(err) = lpmbox_device::run_current_slot_a_stage(|message| {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    }) {
        let err_text = err.to_string();

        if err_text.contains("Fastboot 기기 감지 실패")
            || err_text.contains("Fastboot 기기 감지 시간 초과")
        {
            let _ = tx.send(ProinfoLiveEvent::Error(
                "Fastboot 기기 감지 실패".to_string(),
            ));
        } else {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "current slot A 설정 실패: {err_text}"
            )));
        }

        return;
    }

    let _ = tx.send(ProinfoLiveEvent::Log("[5단계] ROM 설치".to_string()));

    lpmbox_device::trigger_rom_install_reboot_commands(|message| {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    });

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[6단계] MediaTek PreLoader 포트 감지".to_string(),
    ));

    let _ = tx.send(ProinfoLiveEvent::Log(
        internal_spinner_message("preloader_detect", "[Port] PreLoader 포트 감지 중... │"),
    ));

    let detect = lpmbox_device::detect_preloader_until_timeout(30);

    if !detect.detected {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "PreLoader 포트 감지 실패: 30초 안에 MediaTek PreLoader USB VCOM 포트를 찾지 못했습니다. 감지 문자열: {}",
            detect.checked_tokens.join(" / ")
        )));
        return;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        internal_spinner_message("preloader_detect", "[Port] PreLoader 포트 감지 완료"),
    ));

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[7단계] SPFlashToolV6 ROM 설치".to_string(),
    ));

    let result = lpmbox_spft::execute_firmware_download_streaming(&output.plan, |event| {
        let _ = tx.send(event);
    });

    match result {
        Ok(()) => {
            let _ = tx.send(ProinfoLiveEvent::Log(
                "[완료] 2번 옵션 ROW(글로벌) 펌웨어 업데이트 [데이터 유지] 작업이 완료되었습니다."
                    .to_string(),
            ));
        }
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "SPFlashToolV6 download 실패: {err}"
            )));
        }
    }
}

fn run_reinstall_wipe_flow(image_dir: PathBuf, tx: mpsc::Sender<ProinfoLiveEvent>) {
    let _ = tx.send(ProinfoLiveEvent::Log(
        "[1단계] 사전 파일 준비".to_string(),
    ));

    if !ensure_block_firmware_rules_for_flow(&tx) {
        return;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[2단계] MTK 드라이버 확인".to_string(),
    ));

    if !ensure_mtk_driver_installed_for_flow(&tx) {
        return;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[3단계] image 폴더 기본 검사 및 image 파일 모델/플랫폼 확인".to_string(),
    ));

    let firmware = match lpmbox_firmware::inspect_firmware(&image_dir) {
        Ok(firmware) => firmware,
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "image 폴더 검사 실패: {err}"
            )));
            return;
        }
    };

    let _ = tx.send(ProinfoLiveEvent::Log(format!(
        "[Image] 모델명: {}",
        firmware.model
    )));

    if let Some(version) = &firmware.version {
        let _ = tx.send(ProinfoLiveEvent::Log(format!(
            "[Image] 버전: {version}"
        )));
    }

    let _ = tx.send(ProinfoLiveEvent::Log(format!(
        "[Image] ROM 타입: {}",
        region_label(firmware.region)
    )));

    if let Some(platform) = &firmware.platform {
        let _ = tx.send(ProinfoLiveEvent::Log(format!(
            "[Image] 플랫폼: {platform}"
        )));
    }

    if firmware.blocked_firmware_check.blocked {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "설치 금지 펌웨어입니다: {}",
            firmware.blocked_firmware_check.message
        )));
        return;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[4단계] 기기 복구 준비".to_string(),
    ));

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[Recovery] 옵션 선택 페이지와 국가 코드 변경 없이 진행합니다.".to_string(),
    ));

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[Recovery] 데이터 유지/초기화 정책: 데이터 초기화".to_string(),
    ));

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[5단계] Flash Plan 준비 및 작업용 scatter/xml 생성".to_string(),
    ));

    let output = match lpmbox_firmware::prepare_flash_plan_with_log(
        &image_dir,
        InstallMode::ReinstallWipe,
        |message| {
            let _ = tx.send(ProinfoLiveEvent::Log(message));
        },
    ) {
        Ok(output) => output,
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!("기기 복구 준비 실패: {err}")));
            return;
        }
    };

    for message in flash_prepared_log_messages(&output) {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[Recovery] proinfo 파티션은 기기 복구 루틴에서 비활성화합니다.".to_string(),
    ));

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[6단계] MediaTek PreLoader 포트 감지 (제한 시간 30초)".to_string(),
    ));

    lpmbox_device::trigger_rom_install_reboot_commands(|message| {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    });

    let _ = tx.send(ProinfoLiveEvent::Log(
        internal_spinner_message("preloader_detect", "[Port] PreLoader 포트 감지 중... (30초) │"),
    ));

    let detect = lpmbox_device::detect_preloader_until_timeout(30);

    if !detect.detected {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "PreLoader 포트 감지 실패: 30초 안에 MediaTek PreLoader USB VCOM 포트를 찾지 못했습니다. 감지 문자열: {}",
            detect.checked_tokens.join(" / ")
        )));
        return;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        internal_spinner_message("preloader_detect", "[Port] PreLoader 포트 감지 완료"),
    ));

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[7단계] SPFlashToolV6 ROM 설치".to_string(),
    ));

    let result = lpmbox_spft::execute_firmware_download_streaming(&output.plan, |event| {
        let _ = tx.send(event);
    });

    match result {
        Ok(()) => {
            let _ = tx.send(ProinfoLiveEvent::Log(
                "[완료] 기기 복구 [데이터 초기화] 작업이 완료되었습니다.".to_string(),
            ));
        }
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "SPFlashToolV6 기기 복구 실패: {err}"
            )));
        }
    }
}

fn run_mtk_driver_prepare_flow(tx: mpsc::Sender<ProinfoLiveEvent>) {
    let result = lpmbox_device::prepare_mtk_driver_package(|message| {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    });

    match result {
        Ok(()) => {
            let _ = tx.send(ProinfoLiveEvent::Log(
                "[Driver] MTK 드라이버 설치 파일 준비가 완료되었습니다.".to_string(),
            ));
        }
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "MTK 드라이버 설치 파일 준비 실패: {err}"
            )));
        }
    }
}

fn run_mtk_driver_install_flow(tx: mpsc::Sender<ProinfoLiveEvent>) {
    let result = lpmbox_device::install_mtk_driver(|message| {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    });

    match result {
        Ok(()) => {
            let _ = tx.send(ProinfoLiveEvent::Log(
                "[Driver] MTK 드라이버 설치 작업이 완료되었습니다.".to_string(),
            ));
        }
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "MTK 드라이버 설치 실패: {err}"
            )));
        }
    }
}

fn run_vc_runtime_install_flow(tx: mpsc::Sender<ProinfoLiveEvent>) {
    let result = lpmbox_device::install_vc_runtimes(|message| {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    });

    match result {
        Ok(()) => {
            let _ = tx.send(ProinfoLiveEvent::Log(
                "[Runtime] Microsoft Visual C++ 런타임 설치 작업이 완료되었습니다."
                    .to_string(),
            ));
        }
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "Microsoft Visual C++ 런타임 설치 실패: {err}"
            )));
        }
    }
}

fn run_country_reset_flow(
    image_dir: PathBuf,
    selected_country_code: String,
    tx: mpsc::Sender<ProinfoLiveEvent>,
) {
    if !ensure_block_firmware_rules_for_flow(&tx) {
        return;
    }

    if !ensure_mtk_driver_installed_for_flow(&tx) {
        return;
    }

    let country_code = normalize_selected_country_code(&selected_country_code);

    if country_code.len() != 2 {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "국가 코드 재설정 실패: 올바르지 않은 국가 코드입니다: {selected_country_code}"
        )));
        return;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[1단계] ADB 기기 감지 및 기기 정보 확인".to_string(),
    ));

    if let Err(err) = lpmbox_device::probe_adb_device_for_convert_wipe(|message| {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    }) {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "ADB 기기 확인 실패: {err}"
        )));
        return;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[2단계] image 폴더 기본 검사".to_string(),
    ));

    if let Err(err) = lpmbox_firmware::inspect_firmware(&image_dir) {
        let _ = tx.send(ProinfoLiveEvent::Error(format!("image 폴더 검사 실패: {err}")));
        return;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[3단계] 국가 코드 재설정용 재부팅 및 MediaTek PreLoader 포트 감지".to_string(),
    ));

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[4단계] proinfo 백업 및 선택한 국가 코드로 수정".to_string(),
    ));

    if !run_optional_country_proinfo_backup(&image_dir, Some(country_code.as_str()), &tx) {
        return;
    }

    remove_spft_log_folder_after_proinfo_backup(&tx);

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[5단계] proinfo 전용 Flash Plan 준비 및 작업용 scatter/xml 생성".to_string(),
    ));

    let output = match lpmbox_firmware::prepare_flash_plan_with_country_code_log(
        &image_dir,
        InstallMode::CountryReset,
        Some(country_code.as_str()),
        |message| {
            let _ = tx.send(ProinfoLiveEvent::Log(message));
        },
    ) {
        Ok(output) => output,
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "국가 코드 재설정 플래싱 준비 실패: {err}"
            )));
            return;
        }
    };

    for message in flash_prepared_log_messages(&output) {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[6단계] 수정한 proinfo 플래싱용 재부팅".to_string(),
    ));

    lpmbox_device::trigger_rom_install_reboot_commands(|message| {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    });

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[7단계] MediaTek PreLoader 포트 감지 (제한 시간 30초)".to_string(),
    ));

    let _ = tx.send(ProinfoLiveEvent::Log(
        internal_spinner_message("country_reset_preloader_detect", "[Port] PreLoader 포트 감지 중... (30초) │"),
    ));

    let detect = lpmbox_device::detect_preloader_until_timeout(30);

    if !detect.detected {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "PreLoader 포트 감지 실패: 30초 안에 MediaTek PreLoader USB VCOM 포트를 찾지 못했습니다. 감지 문자열: {}",
            detect.checked_tokens.join(" / ")
        )));
        return;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        internal_spinner_message("country_reset_preloader_detect", "[Port] PreLoader 포트 감지 완료"),
    ));

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[8단계] SPFlashToolV6 proinfo 파티션만 플래싱".to_string(),
    ));

    let result = lpmbox_spft::execute_firmware_download_streaming(&output.plan, |event| {
        let _ = tx.send(event);
    });

    match result {
        Ok(()) => {
            let _ = tx.send(ProinfoLiveEvent::Log(format!(
                "[완료] 국가 코드 재설정 작업이 완료되었습니다: {country_code}"
            )));
        }
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "SPFlashToolV6 국가 코드 재설정 실패: {err}"
            )));
        }
    }
}

fn run_ota_disable_flow(tx: mpsc::Sender<ProinfoLiveEvent>) {
    let result = lpmbox_device::disable_ota_updates(|message| {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    });

    match result {
        Ok(()) => {
            let _ = tx.send(ProinfoLiveEvent::Log(
                "[완료] OTA(업데이트) 비활성화 작업이 완료되었습니다.".to_string(),
            ));
        }
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "OTA(업데이트) 비활성화 실패: {err}"
            )));
        }
    }
}

fn run_ota_enable_flow(tx: mpsc::Sender<ProinfoLiveEvent>) {
    let result = lpmbox_device::enable_ota_updates(|message| {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    });

    match result {
        Ok(()) => {
            let _ = tx.send(ProinfoLiveEvent::Log(
                "[완료] OTA(업데이트) 활성화 작업이 완료되었습니다.".to_string(),
            ));
        }
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "OTA(업데이트) 활성화 실패: {err}"
            )));
        }
    }
}

fn ensure_mtk_driver_installed_for_flow(tx: &mpsc::Sender<ProinfoLiveEvent>) -> bool {
    match lpmbox_device::check_mtk_driver_installed() {
        Ok(true) => true,
        Ok(false) => {
            let _ = tx.send(ProinfoLiveEvent::Log(
                "[Driver] MediaTek 드라이버가 설치 되어있지 않습니다, MTK 드라이버 설치 후 다시 시도해주세요."
                    .to_string(),
            ));
            false
        }
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Log(format!(
                "[Driver] MTK 드라이버 설치 여부 확인 실패: {err}"
            )));
            false
        }
    }
}

fn remove_spft_log_folder_after_proinfo_backup(tx: &mpsc::Sender<ProinfoLiveEvent>) {
    let spft_log_dir = lpmbox_core::app_paths::runtime_root()
        .join("backup")
        .join("spft_log");

    if !spft_log_dir.exists() {
        return;
    }

    match std::fs::remove_dir_all(&spft_log_dir) {
        Ok(()) => {
            let _ = tx.send(ProinfoLiveEvent::Log(format!(
                "[backup] Flash Plan 준비 전 spft_log 폴더를 제거했습니다: {}",
                spft_log_dir.display()
            )));
        }
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Log(format!(
                "[backup] spft_log 폴더 제거 실패: {} / {err}",
                spft_log_dir.display()
            )));
        }
    }
}

fn run_optional_country_proinfo_backup(
    image_dir: &Path,
    selected_country_code: Option<&str>,
    tx: &mpsc::Sender<ProinfoLiveEvent>,
) -> bool {
    let Some(country_code) = selected_country_code else {
        return true;
    };

    let country_code = normalize_selected_country_code(country_code);

    if country_code.is_empty() {
        return true;
    }

    let backup_dir = lpmbox_core::app_paths::runtime_root().join("backup");
    let root_backup_proinfo = backup_dir.join("proinfo");

    if let Err(err) = std::fs::create_dir_all(&backup_dir) {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "[backup] backup 폴더 생성 실패: {} / {err}",
            backup_dir.display()
        )));
        return false;
    }

    if let Err(err) = remove_file_if_exists_with_retry(&root_backup_proinfo, 20, 250) {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "[backup] 기존 proinfo 백업 파일 제거 실패: {err}"
        )));
        return false;
    }

    let image_proinfo = image_dir.join("proinfo");

    if let Err(err) = remove_file_if_exists_with_retry(&image_proinfo, 20, 250) {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "[backup] image 폴더의 기존 proinfo 파일 제거 실패: {err}"
        )));
        return false;
    }

    let download_agent_proinfo = image_dir.join("download_agent").join("proinfo");

    if let Err(err) = remove_file_if_exists_with_retry(&download_agent_proinfo, 20, 250) {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "[backup] download_agent 폴더의 기존 proinfo 파일 제거 실패: {err}"
        )));
        return false;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[backup] 국가 코드 변경을 위해 proinfo 파티션을 백업합니다.".to_string(),
    ));

    lpmbox_device::trigger_rom_install_reboot_commands(|message| {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    });

    let _ = tx.send(ProinfoLiveEvent::Log(
        internal_spinner_message("backup_preloader_detect", "[backup] PreLoader 포트 감지 중... │"),
    ));

    let detect = lpmbox_device::detect_preloader_until_timeout(30);

    if !detect.detected {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "[backup] proinfo 백업 실패: 30초 안에 MediaTek PreLoader USB VCOM 포트를 찾지 못했습니다. 감지 문자열: {}",
            detect.checked_tokens.join(" / ")
        )));
        return false;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        internal_spinner_message("backup_preloader_detect", "[backup] PreLoader 포트 감지 완료"),
    ));

    let readback_result = lpmbox_spft::execute_proinfo_readback_streaming(image_dir, |event| {
        match event {
            ProinfoLiveEvent::Log(message) => {
                if message.contains("DA 모드 진입 시작") {
                    let _ = tx.send(ProinfoLiveEvent::Log(
                        "[backup] DA 모드 진입 시작".to_string(),
                    ));
                }
            }

            ProinfoLiveEvent::Progress(mut progress) => {
                progress.stage = format!("[backup] {}", progress.stage);
                let _ = tx.send(ProinfoLiveEvent::Progress(progress));
            }

            ProinfoLiveEvent::Finished(_result) => {}

            ProinfoLiveEvent::Error(err) => {
                let _ = tx.send(ProinfoLiveEvent::Log(format!("[backup] {err}")));
            }
        }
    });

    try_kill_spflashtoolv6_for_file_unlock();

    wait_with_backup_spinner(
        tx,
        "backup_file_unlock_wait",
        "[backup] 안정성을 위해 5초 대기...",
        5,
    );

    let result = match readback_result {
        Ok(result) => result,
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "[backup] proinfo 백업 실패: {err}"
            )));
            return false;
        }
    };

    if !result.proinfo_exists || result.proinfo_size == 0 {
        let _ = tx.send(ProinfoLiveEvent::Error(
            "[backup] proinfo 백업 실패: backup\\proinfo 파일이 생성되지 않았거나 크기가 0입니다."
                .to_string(),
        ));
        return false;
    }

    if let Err(err) = ensure_readback_proinfo_at_root_backup(
        &result.plan.proinfo_out,
        &root_backup_proinfo,
    ) {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "[backup] proinfo 백업 파일 저장 실패: {err}"
        )));
        return false;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[backup] proinfo 파티션 백업을 성공했습니다.".to_string(),
    ));

    let _ = tx.send(ProinfoLiveEvent::Log(format!(
        "[backup] 사용자가 선택한 국가 코드:{country_code}"
    )));

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[backup] proinfo에 국가 코드를 변경합니다.".to_string(),
    ));

    let patch_message = match patch_proinfo_country_file(&root_backup_proinfo, &country_code) {
        Ok(message) => message,
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "[backup] proinfo 국가 코드 변경 실패: {err}"
            )));
            return false;
        }
    };

    let _ = tx.send(ProinfoLiveEvent::Log(format!(
        "[backup] proinfo 국가 코드 변경 완료: {patch_message}"
    )));

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[backup] 수정한 proinfo 파일을 image 폴더로 이동합니다.".to_string(),
    ));

    if let Err(err) = remove_file_if_exists_with_retry(&image_proinfo, 20, 250) {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "[backup] image 폴더의 기존 proinfo 파일 제거 실패: {err}"
        )));
        return false;
    }

    if let Err(err) = copy_file_with_retry(&root_backup_proinfo, &image_proinfo, 20, 250) {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "[backup] 수정한 proinfo 파일을 image 폴더로 이동하지 못했습니다: {err}"
        )));
        return false;
    }

    if let Err(err) = remove_file_if_exists_with_retry(&download_agent_proinfo, 20, 250) {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "[backup] download_agent 폴더의 proinfo 파일 제거 실패: {err}"
        )));
        return false;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[backup] proinfo 파일에 국가 코드가 설정 되었는지 확인합니다.".to_string(),
    ));

    if !verify_proinfo_country_file(&image_proinfo, &country_code) {
        if let Err(err) = patch_proinfo_country_file(&image_proinfo, &country_code) {
            let _ = tx.send(ProinfoLiveEvent::Error(format!(
                "[backup] image 폴더 proinfo 국가 코드 재수정 실패: {err}"
            )));
            return false;
        }
    }

    if !verify_proinfo_country_file(&root_backup_proinfo, &country_code) {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "[backup] LPMBOX backup 폴더의 proinfo 파일에 국가 코드 {country_code}XX가 설정되지 않았습니다."
        )));
        return false;
    }

    if !verify_proinfo_country_file(&image_proinfo, &country_code) {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "[backup] image 폴더의 proinfo 파일에 국가 코드 {country_code}XX가 설정되지 않았습니다."
        )));
        return false;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(format!(
        "[backup] proinfo 국가 코드 확인 완료: {country_code}XX"
    )));

    true
}

fn wait_with_backup_spinner(
    tx: &mpsc::Sender<ProinfoLiveEvent>,
    key: &'static str,
    message: &'static str,
    seconds: u64,
) {
    let total_ticks = seconds.saturating_mul(10).max(1);

    for tick in 0..total_ticks {
        let frame = UI_SPINNER_FRAMES[(tick as usize) % UI_SPINNER_FRAMES.len()];

        let _ = tx.send(ProinfoLiveEvent::Log(internal_spinner_message(
            key,
            &format!("{message} {frame}"),
        )));

        thread::sleep(Duration::from_millis(100));
    }

    let _ = tx.send(ProinfoLiveEvent::Log(internal_spinner_message(key, message)));
}

#[cfg(windows)]
fn try_kill_spflashtoolv6_for_file_unlock() {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let _ = Command::new("taskkill")
        .args(["/f", "/im", "SPFlashToolV6.exe"])
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(not(windows))]
fn try_kill_spflashtoolv6_for_file_unlock() {}

fn ensure_readback_proinfo_at_root_backup(
    readback_proinfo: &Path,
    root_backup_proinfo: &Path,
) -> std::result::Result<(), String> {
    if same_existing_file_path(readback_proinfo, root_backup_proinfo) {
        return Ok(());
    }

    remove_file_if_exists_with_retry(root_backup_proinfo, 20, 250)?;
    copy_file_with_retry(readback_proinfo, root_backup_proinfo, 20, 250)?;

    Ok(())
}

fn same_existing_file_path(left: &Path, right: &Path) -> bool {
    let Ok(left) = std::fs::canonicalize(left) else {
        return false;
    };

    let Ok(right) = std::fs::canonicalize(right) else {
        return false;
    };

    left == right
}

fn remove_file_if_exists_with_retry(
    path: &Path,
    attempts: usize,
    delay_ms: u64,
) -> std::result::Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    let mut last_error = String::new();

    for _ in 0..attempts.max(1) {
        match std::fs::remove_file(path) {
            Ok(_) => return Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(err) => {
                last_error = err.to_string();
                thread::sleep(Duration::from_millis(delay_ms));
            }
        }
    }

    Err(format!("{} / {last_error}", path.display()))
}

fn copy_file_with_retry(
    source: &Path,
    target: &Path,
    attempts: usize,
    delay_ms: u64,
) -> std::result::Result<(), String> {
    let mut last_error = String::new();

    for _ in 0..attempts.max(1) {
        match std::fs::copy(source, target) {
            Ok(_) => return Ok(()),
            Err(err) => {
                last_error = err.to_string();
                thread::sleep(Duration::from_millis(delay_ms));
            }
        }
    }

    Err(format!(
        "{} → {} / {last_error}",
        source.display(),
        target.display()
    ))
}

fn write_file_with_retry(
    path: &Path,
    data: &[u8],
    attempts: usize,
    delay_ms: u64,
) -> std::result::Result<(), String> {
    let mut last_error = String::new();

    for _ in 0..attempts.max(1) {
        match std::fs::write(path, data) {
            Ok(_) => return Ok(()),
            Err(err) => {
                last_error = err.to_string();
                thread::sleep(Duration::from_millis(delay_ms));
            }
        }
    }

    Err(format!("{} / {last_error}", path.display()))
}

fn normalize_selected_country_code(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .take(2)
        .collect::<String>()
        .to_ascii_uppercase()
}

fn patch_proinfo_country_file(
    path: &Path,
    selected_country_code: &str,
) -> std::result::Result<String, String> {
    let country_code = normalize_selected_country_code(selected_country_code);

    if country_code.len() != 2 {
        return Err(format!("올바르지 않은 국가 코드입니다: {selected_country_code}"));
    }

    let mut data = std::fs::read(path)
        .map_err(|err| format!("{} 읽기 실패 / {err}", path.display()))?;

    let new_token = format!("{country_code}XX").into_bytes();

    let Some((index, old_token)) = find_proinfo_country_token(&data) else {
        return Err("proinfo 내부에서 KRXX, JPXX, USXX 같은 국가 코드 토큰을 찾지 못했습니다.".to_string());
    };

    data[index..index + 4].copy_from_slice(&new_token);

    write_file_with_retry(path, &data, 20, 250)?;

    Ok(format!(
        "{} → {} / offset 0x{:X}",
        ascii_token_to_string(&old_token),
        ascii_token_to_string(&new_token),
        index
    ))
}

fn verify_proinfo_country_file(path: &Path, selected_country_code: &str) -> bool {
    let country_code = normalize_selected_country_code(selected_country_code);

    if country_code.len() != 2 {
        return false;
    }

    let Ok(data) = std::fs::read(path) else {
        return false;
    };

    let token = format!("{country_code}XX").into_bytes();

    data.windows(token.len()).any(|window| window == token.as_slice())
}

fn find_proinfo_country_token(data: &[u8]) -> Option<(usize, Vec<u8>)> {
    for code in proinfo_country_code_candidates() {
        let token = format!("{code}XX").into_bytes();

        if let Some(index) = find_bytes(data, &token) {
            return Some((index, token));
        }
    }

    None
}

fn proinfo_country_code_candidates() -> Vec<&'static str> {
    let mut codes: Vec<&'static str> = ROM_COUNTRY_CODES
        .iter()
        .map(|entry| entry.code)
        .collect();

    for code in ["CN", "KR", "JP", "US", "ZA"] {
        if !codes.iter().any(|value| value.eq_ignore_ascii_case(code)) {
            codes.push(code);
        }
    }

    codes
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }

    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn ascii_token_to_string(token: &[u8]) -> String {
    String::from_utf8_lossy(token).to_string()
}



fn image_folder_mismatch_message() -> String {
    "[Image] 기기에 맞는 image 폴더가 아닙니다, 올바른 파일을 선택해서 다시 시도해주세요."
        .to_string()
}

fn normalize_lenovo_model_for_compare(model: &str) -> String {
    let upper = model.trim().to_ascii_uppercase();

    for known in [
        "TB375FC", "TB373FU", "TB365FC", "TB361FU", "TB335FC", "TB336FU",
    ] {
        if upper.contains(known) {
            return known.to_string();
        }
    }

    upper
}

fn is_same_or_convertible_lpmbox_model_pair(device_model: &str, image_model: &str) -> bool {
    let device = normalize_lenovo_model_for_compare(device_model);
    let image = normalize_lenovo_model_for_compare(image_model);

    if device.eq_ignore_ascii_case(&image) {
        return true;
    }

    matches!(
        (device.as_str(), image.as_str()),
        ("TB373FU", "TB375FC")
            | ("TB375FC", "TB373FU")
            | ("TB361FU", "TB365FC")
            | ("TB365FC", "TB361FU")
            | ("TB336FU", "TB335FC")
            | ("TB335FC", "TB336FU")
    )
}

fn validate_device_and_firmware_for_convert_wipe(
    device: &lpmbox_device::AdbDeviceProbe,
    firmware: &FirmwareInfo,
) -> Result<String, String> {
    let platform = firmware
        .platform
        .as_deref()
        .unwrap_or_default();

    if !device.platform.eq_ignore_ascii_case(platform) {
        return Err(image_folder_mismatch_message());
    }

    let device_model = normalize_lenovo_model_for_compare(&device.model);
    let image_model = normalize_lenovo_model_for_compare(&firmware.model);

    if !is_same_or_convertible_lpmbox_model_pair(&device_model, &image_model) {
        return Err(image_folder_mismatch_message());
    }

    match (device.region, firmware.region) {
        (RomRegion::Prc, RomRegion::Row) | (RomRegion::Row, RomRegion::Prc) => {
            Ok(format!(
                "[Image] 기기에 {}을 설치합니다.",
                region_label(firmware.region)
            ))
        }

        (RomRegion::Row, RomRegion::Row) => Err(
            "[Image] 2. ROW(글로벌롬) 펌웨어 업데이트 [데이터 유지]를 시도해주세요."
                .to_string(),
        ),

        (RomRegion::Prc, RomRegion::Prc) => Err(
            "[Image] LPMBOX는 PRC(중국 내수롬) 업데이트를 지원하지 않습니다.".to_string(),
        ),

        _ => Err(image_folder_mismatch_message()),
    }
}

fn validate_device_and_firmware_for_row_update_keep_data(
    device: &lpmbox_device::AdbDeviceProbe,
    firmware: &FirmwareInfo,
) -> Result<String, String> {
    let platform = firmware
        .platform
        .as_deref()
        .unwrap_or_default();

    if !device.platform.eq_ignore_ascii_case(platform) {
        return Err(image_folder_mismatch_message());
    }

    let device_model = normalize_lenovo_model_for_compare(&device.model);
    let image_model = normalize_lenovo_model_for_compare(&firmware.model);

    if !is_same_or_convertible_lpmbox_model_pair(&device_model, &image_model) {
        return Err(image_folder_mismatch_message());
    }

    match (device.region, firmware.region) {
        (RomRegion::Row, RomRegion::Row) => {
            Ok("[Image] 기기에 ROW(글로벌롬) 버전을 업데이트 합니다.".to_string())
        }

        (RomRegion::Prc, RomRegion::Prc) => Err(
            "[Image] LPMBOX는 PRC(중국 내수롬) 업데이트를 지원하지 않습니다.".to_string(),
        ),

        (RomRegion::Prc, RomRegion::Row) | (RomRegion::Row, RomRegion::Prc) => {
            Err("[Image] 1. PRC/ROW 설치 [데이터 초기화]를 시도해주세요.".to_string())
        }

        _ => Err(image_folder_mismatch_message()),
    }
}


fn run_proinfo_backup_flow(image_dir: PathBuf, tx: mpsc::Sender<ProinfoLiveEvent>) {
    if !ensure_mtk_driver_installed_for_flow(&tx) {
        return;
    }

    lpmbox_device::trigger_rom_install_reboot_commands(|message| {
        let _ = tx.send(ProinfoLiveEvent::Log(message));
    });

    let _ = tx.send(ProinfoLiveEvent::Log(
        internal_spinner_message("preloader_detect", "[Port] PreLoader 포트 감지 중... │"),
    ));

    let detect = lpmbox_device::detect_preloader_until_timeout(30);

    if !detect.detected {
        let _ = tx.send(ProinfoLiveEvent::Error(format!(
            "PreLoader 포트 감지 실패: 30초 안에 MediaTek PreLoader USB VCOM 포트를 찾지 못했습니다. 감지 문자열: {}",
            detect.checked_tokens.join(" / ")
        )));
        return;
    }

    let _ = tx.send(ProinfoLiveEvent::Log(
        internal_spinner_message("preloader_detect", "[Port] PreLoader 포트 감지 완료"),
    ));

    let _ = tx.send(ProinfoLiveEvent::Log(
        "[2단계] proinfo 파티션 추출 중...".to_string(),
    ));

    let result = lpmbox_spft::execute_proinfo_readback_streaming(&image_dir, |event| {
        let _ = tx.send(event);
    });

    match result {
        Ok(result) => {
            let _ = tx.send(ProinfoLiveEvent::Finished(result));
        }
        Err(err) => {
            let _ = tx.send(ProinfoLiveEvent::Error(err.to_string()));
        }
    }
}

fn flash_prepared_log_messages(output: &FlashPreparedOutput) -> Vec<String> {
    let plan = &output.plan;
    let mut messages = Vec::new();

    messages.push(format!("[Image] 모델명: {}", plan.model));

    if let Some(version) = &plan.image_version {
        messages.push(format!("[Image] 버전: {version}"));
    }

    messages.push(format!("[Image] ROM 타입: {}", region_label(plan.image_region)));
    messages.push(format!("[Image] 플랫폼: {}", plan.platform));

    messages.push(format!("[Plan] patch 변경 수: {}개", output.changed_count));

    messages.push(format!(
        "[Plan] 작업 scatter XML 재파싱: 성공 / root: {} / partition {}개",
        output.generated_scatter_info.root_name, output.generated_scatter_info.partition_count
    ));

    messages.push(format!(
        "[Plan] 데이터 유지 여부: {}",
        if plan.keep_user_data { "유지" } else { "초기화" }
    ));

    messages.push(format!(
        "[Plan] current slot stage 필요 여부: {}",
        yes_no(plan.requires_current_slot_stage)
    ));

    messages.push(format!(
        "[Plan] 기기 ADB 단계 필요 여부: {}",
        yes_no(plan.requires_device_stage)
    ));

    messages.push(format!(
        "[Plan] proinfo 패치 필요 여부: {}",
        yes_no(plan.requires_proinfo_patch)
    ));

    if let Some(validation) = &output.selected_patch_validation {
        for warning in &validation.warnings {
            messages.push(format!("검증 경고 / {}: {warning}", validation.title));
        }

        for error in &validation.errors {
            messages.push(format!("검증 오류 / {}: {error}", validation.title));
        }
    }

    messages
}

async fn check_firmware_worker(image_dir: PathBuf) -> Result<RomFirmwareCheckResult, String> {
    tokio::task::spawn_blocking(move || {
        let driver_check = lpmbox_device::check_mtk_driver_installed();
        let vc_runtime_check = lpmbox_device::check_vc_runtime_installed();

        let firmware = lpmbox_firmware::inspect_firmware(&image_dir)
            .map_err(|err| err.to_string())?;

        let (mtk_driver_installed, mtk_driver_error) = match driver_check {
            Ok(installed) => (Some(installed), None),
            Err(err) => (None, Some(err.to_string())),
        };

        let (vc_runtime_status, vc_runtime_error) = match vc_runtime_check {
            Ok(status) => (Some(status), None),
            Err(err) => (None, Some(err.to_string())),
        };

        Ok(RomFirmwareCheckResult {
            firmware,
            mtk_driver_installed,
            mtk_driver_error,
            vc_runtime_status,
            vc_runtime_error,
        })
    })
    .await
    .unwrap_or_else(|err| Err(format!("펌웨어 검사 작업 스레드 오류: {err}")))
}

async fn load_dashboard_snapshot_worker() -> Result<DashboardSnapshot, String> {
    tokio::task::spawn_blocking(|| {
        let info = lpmbox_device::read_dashboard_device_info()
            .map_err(|err| err.to_string())?;

        Ok(DashboardSnapshot { info })
    })
    .await
    .unwrap_or_else(|err| Err(format!("대시보드 갱신 작업 스레드 오류: {err}")))
}

fn build_loading_progress_handles() -> Vec<iced::widget::image::Handle> {
    let mut stacked_image = image_crate::RgbaImage::from_pixel(
        LOADING_PROGRESS_SIZE,
        LOADING_PROGRESS_SIZE,
        image_crate::Rgba([255, 255, 255, 255]),
    );

    let mut handles = Vec::with_capacity(LOADING_PROGRESS_FRAME_BYTES.len());

    for bytes in LOADING_PROGRESS_FRAME_BYTES {
        let Some(frame_image) = load_loading_progress_active_dot_png(bytes) else {
            continue;
        };

        overlay_loading_progress_active_dot(&mut stacked_image, &frame_image);

        if let Some(handle) = loading_progress_handle_from_rgba(&stacked_image) {
            handles.push(handle);
        }
    }

    if handles.is_empty() {
        handles.push(smooth_png_handle(
            LOADING_PROGRESS_FRAME_01_BYTES,
            LOADING_PROGRESS_SIZE,
            LOADING_PROGRESS_SIZE,
        ));
    }

    handles
}

fn load_loading_progress_active_dot_png(bytes: &'static [u8]) -> Option<image_crate::RgbaImage> {
    let image = image_crate::load_from_memory(bytes).ok()?.to_rgba8();

    let resized = image_crate::imageops::resize(
        &image,
        LOADING_PROGRESS_SIZE,
        LOADING_PROGRESS_SIZE,
        image_crate::imageops::FilterType::Lanczos3,
    );

    let mut active_only = image_crate::RgbaImage::from_pixel(
        LOADING_PROGRESS_SIZE,
        LOADING_PROGRESS_SIZE,
        image_crate::Rgba([0, 0, 0, 0]),
    );

    for y in 0..LOADING_PROGRESS_SIZE {
        for x in 0..LOADING_PROGRESS_SIZE {
            let pixel = *resized.get_pixel(x, y);
            let alpha = pixel.0[3];

            if alpha >= LOADING_PROGRESS_ACTIVE_ALPHA_THRESHOLD {
                active_only.put_pixel(x, y, image_crate::Rgba([0, 0, 0, alpha]));
            }
        }
    }

    Some(active_only)
}

fn overlay_loading_progress_active_dot(
    base: &mut image_crate::RgbaImage,
    layer: &image_crate::RgbaImage,
) {
    for y in 0..LOADING_PROGRESS_SIZE {
        for x in 0..LOADING_PROGRESS_SIZE {
            let src = *layer.get_pixel(x, y);

            if src.0[3] == 0 {
                continue;
            }

            base.put_pixel(x, y, image_crate::Rgba([0, 0, 0, src.0[3]]));
        }
    }
}

fn smooth_png_handle(bytes: &'static [u8], width: u32, height: u32) -> iced::widget::image::Handle {
    let Ok(image) = image_crate::load_from_memory(bytes) else {
        return iced::widget::image::Handle::from_bytes(bytes.to_vec());
    };

    let rgba = image.to_rgba8();

    let resized = image_crate::imageops::resize(
        &rgba,
        width,
        height,
        image_crate::imageops::FilterType::Lanczos3,
    );

    iced::widget::image::Handle::from_rgba(width, height, resized.into_raw())
}

fn loading_progress_handle_from_rgba(
    image: &image_crate::RgbaImage,
) -> Option<iced::widget::image::Handle> {
    Some(iced::widget::image::Handle::from_rgba(
        LOADING_PROGRESS_SIZE,
        LOADING_PROGRESS_SIZE,
        image.clone().into_raw(),
    ))
}

fn build_battery_progress_handle(level: Option<u8>) -> iced::widget::image::Handle {
    let size: u32 = 96;

    let Some(gray_ring) = load_battery_ring_png(BATTERY_RING_GRAY_BYTES, size) else {
        return iced::widget::image::Handle::from_bytes(Vec::new());
    };

    let Some(green_ring) = load_battery_ring_png(BATTERY_RING_GREEN_BYTES, size) else {
        return iced::widget::image::Handle::from_bytes(Vec::new());
    };

    let mut output = gray_ring;
    let level = level.unwrap_or(0).min(100);

    if level > 0 {
        let filled_angle = 360.0 * (level as f32 / 100.0);
        let center = size as f32 / 2.0;

        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 + 0.5 - center;
                let dy = y as f32 + 0.5 - center;

                let mut angle = dy.atan2(dx).to_degrees() + 90.0;

                if angle < 0.0 {
                    angle += 360.0;
                }

                if angle <= filled_angle {
                    let green_pixel = *green_ring.get_pixel(x, y);

                    if green_pixel.0[3] > 0 {
                        output.put_pixel(x, y, green_pixel);
                    }
                }
            }
        }
    }

    let mut cursor = std::io::Cursor::new(Vec::new());

    if image_crate::DynamicImage::ImageRgba8(output)
        .write_to(&mut cursor, image_crate::ImageFormat::Png)
        .is_err()
    {
        return iced::widget::image::Handle::from_bytes(Vec::new());
    }

    iced::widget::image::Handle::from_bytes(cursor.into_inner())
}

fn load_battery_ring_png(bytes: &'static [u8], size: u32) -> Option<image_crate::RgbaImage> {
    let image = image_crate::load_from_memory(bytes).ok()?.to_rgba8();

    Some(image_crate::imageops::resize(
        &image,
        size,
        size,
        image_crate::imageops::FilterType::Lanczos3,
    ))
}

fn build_log_text(log_lines: &[LogLine]) -> String {
    if log_lines.is_empty() {
        return lpm_translate_owned("로그가 없습니다.".to_string());
    }

    let start = log_lines.len().saturating_sub(LOG_VISIBLE_MAX_LINES);
    let visible_lines = &log_lines[start..];
    let mut text = String::new();

    if start > 0 {
        text.push_str(&lpm_omitted_log_message(start));
    }

    for line in visible_lines {
        if !text.is_empty() {
            text.push('\n');
        }

        let localized_message = lpm_translate_owned(line.message.clone());
        text.push_str(&format_log_message(&localized_message));
    }

    text
}

fn build_log_export_text(log_lines: &[LogLine]) -> String {
    if log_lines.is_empty() {
        return format!("{}\r\n", lpm_translate_owned("로그가 없습니다.".to_string()));
    }

    let mut text = log_lines
        .iter()
        .map(|line| lpm_translate_owned(line.message.clone()))
        .collect::<Vec<_>>()
        .join("\r\n");

    text.push_str("\r\n");
    text
}

fn build_log_display_rows(log_lines: &[LogLine]) -> Vec<String> {
    if log_lines.is_empty() {
        return vec![lpm_translate_owned("로그가 없습니다.".to_string())];
    }

    let start = log_lines.len().saturating_sub(LOG_VISIBLE_MAX_LINES);
    let visible_lines = &log_lines[start..];
    let mut rows = Vec::new();

    if start > 0 {
        rows.extend(split_log_display_message(&format_log_message(&lpm_omitted_log_message(start))));
    }

    for line in visible_lines {
        let localized_message = lpm_translate_owned(line.message.clone());
        rows.extend(split_log_display_message(&format_log_message(&localized_message)));
    }

    rows
}

fn split_log_display_message(message: &str) -> Vec<String> {
    let mut rows = Vec::new();

    for row in message.lines() {
        rows.push(row.to_string());
    }

    if rows.is_empty() {
        rows.push(String::new());
    }

    rows
}

fn format_log_message(message: &str) -> String {
    if message.contains("PC(노트북)와 연결한 태블릿에 잠금 해제")
        || message.contains("메세지 창 왼쪽 중간 체크 박스 체크")
        || message.contains("Allow(허용)")
    {
        return message.to_string();
    }

    visual_wrap(message, LOG_WRAP_CHARS)
}

fn internal_spinner_message(key: &str, message: &str) -> String {
    format!("{INTERNAL_SPINNER_PREFIX}{key}|{message}")
}

fn parse_spinner_log(message: &str) -> Option<(String, String)> {
    let rest = message.strip_prefix(INTERNAL_SPINNER_PREFIX)?;
    let (key, message) = rest.split_once('|')?;
    Some((key.to_string(), message.to_string()))
}

fn split_spinner_message(message: &str) -> (String, bool) {
    let trimmed = message.trim_end();

    for frame in UI_SPINNER_FRAMES {
        if let Some(base) = trimmed.strip_suffix(frame) {
            return (base.trim_end().to_string(), true);
        }
    }

    for frame in ["|", "/", "-", "\\"] {
        if let Some(base) = trimmed.strip_suffix(frame) {
            return (base.trim_end().to_string(), true);
        }
    }

    (message.to_string(), false)
}

fn is_usb_debugging_shell_error(message: &str) -> bool {
    message.contains("ADB shell 실패")
        || message.contains("Open session failed")
        || message.contains("got CLSE in response instead of OKAY")
        || message.contains("getprop ro.build.version.release")
}

fn is_runtime_flow_completion_log(message: &str) -> bool {
    message.contains("[완료] 1번 옵션 PRC/ROW 펌웨어 설치 작업이 완료되었습니다.")
        || message.contains("[완료] 2번 옵션 ROW(글로벌) 펌웨어 업데이트 [데이터 유지] 작업이 완료되었습니다.")
        || message.contains("[완료] 기기 복구 [데이터 초기화] 작업이 완료되었습니다.")
        || message.contains("[완료] 국가 코드 재설정 작업이 완료되었습니다")
        || message.contains("[완료] OTA(업데이트) 비활성화 작업이 완료되었습니다.")
        || message.contains("[완료] OTA(업데이트) 활성화 작업이 완료되었습니다.")
}

fn normalize_log_message(message: &str) -> Option<String> {
    let trimmed = message.trim();

    if is_usb_debugging_shell_error(trimmed) {
        return Some("작업 오류: 개발자 옵션에서 'USB 디버깅'을 활성화로 설정 한 다음 다시 시도해주세요.".to_string());
    }

    let hide_patterns = [
        "원본 flash.xml:",
        "원본 scatter:",
        "작업 flash.xml:",
        "작업 scatter.xml:",
        "DA 파일:",
        "SPFlashToolV6.exe:",
        "flash.xml:",
        "SPFlashToolV6 download 실행 시작",
        "SPFlashToolV6 download 완료",
        "[ADB] 현재 slot:",
        "[Reboot] Fastboot reboot 실행",
        "[ADB] 현재 slot:",
        "[Reboot] Fastboot reboot 실행",
        "SPFlashToolV6 경고: Warning:DA SLA Disabled",
        "SPFlashToolV6 경고: Warning: DA SLA Disabled",
        "DA SLA Disabled",
        "Manifest file do not exists",
        "Manifast file do not exists",
        "PreLoader 감지 방식:",
        "SPFlashToolV6 readback 실행 시작",
        "SPFlashToolV6 작업 완료",
        "[2단계] SPFlashToolV6 proinfo readback 완료",
        "내장 SPFlashToolV6 도구 폴더:",
        "SPFlashToolV6 종료 코드:",
        "포트가 감지되면 SPFlashToolV6 readback을 바로 실행합니다.",
        "주의: 이 작업은 실제 SPFlashToolV6 download를 실행합니다.",
        "SPFlashToolV6 예정 명령:",
        "ADB USB 직접 연결 실패",
        "cann ot find USB devices matching the signature of an ADB device",
        "오류: ADB 오류: ADB USB 직접 연결 실패",
        "외부 adb.exe server가 USB를 점유 중일 수 있습니다",
        "외부 adb server가 감지되어",
    ];

    for pattern in hide_patterns {
        if trimmed.contains(pattern) {
            return None;
        }
    }

    if trimmed.contains("[Slot] current slot A 강제 단계를 시작합니다.") {
        return None;
    }

    if trimmed.contains("[Slot] ADB bootctl set-active-boot-slot 0 실행") {
        return Some("[ADB] ADB 명령어로 Slot 설정 중...".to_string());
    }

    if trimmed.contains("[Slot] ADB bootctl set-active-boot-slot 0 완료") {
        return Some("[ADB] ADB 명령어로 Slot 설정 완료".to_string());
    }

    if trimmed.contains("[ADB] adb reboot bootloader 실행") {
        return Some("[ADB] bootloader 모드 설정".to_string());
    }

    if trimmed.contains("[Fastboot] Fastboot 기기 재감지 중") {
        return Some("[Fastboot] 기기 감지 중".to_string());
    }

    if trimmed.starts_with("PreLoader 포트 감지: 성공") {
        return Some("[Port] PreLoader 포트 감지 완료".to_string());
    }

    if trimmed.contains("SPFlashToolV6 proinfo readback 실행") {
        return Some("proinfo 파티션 추출 중...".to_string());
    }

    if trimmed.contains("SPFlashToolV6 proinfo readback 완료") {
        return Some("proinfo 파티션 추출 성공".to_string());
    }

    if trimmed.contains("SPFlashToolV6 도구 방식: 프로그램 내장 ZIP 자동 추출") {
        return Some("ROOT 경로에 SPFlashToolV6 저장하는 중...".to_string());
    }

    if trimmed.contains(
        "[3단계] 재시작 명령은 readback_config.xml 안의 REBOOT COLD-RESET으로 수행됩니다.",
    ) {
        return Some("[3단계] 태블릿 재부팅 중...".to_string());
    }

    if trimmed == "SPFlashToolV6 download 실행 시작" {
        return Some("[SPFT] SPFlashToolV6 download 실행 시작".to_string());
    }

    if trimmed == "DA 모드 진입 시작" {
        return Some("[SPFT] DA 모드 진입 시작".to_string());
    }

    if trimmed.starts_with("PreLoader 포트 감지: 성공") {
        return Some("[Port] PreLoader 포트 감지 완료".to_string());
    }

    Some(trimmed.to_string())
}

fn normalize_rom_label_for_option(value: &str) -> &'static str {
    let upper = value.trim().to_ascii_uppercase();

    if upper.contains("PRC") || upper.contains("중국") || upper.contains("내수") {
        "PRC"
    } else if upper.contains("ROW") || upper.contains("GLOBAL") || upper.contains("글로벌") {
        "ROW"
    } else {
        ""
    }
}

fn model_rom_kind_from_suffix(model: &str) -> Option<&'static str> {
    let normalized = normalize_lenovo_model_for_compare(model);

    if normalized.ends_with("FC") {
        Some("PRC")
    } else if normalized.ends_with("FU") {
        Some("ROW")
    } else {
        None
    }
}

fn is_supported_lpmbox_model(model: &str) -> bool {
    let upper = model.trim().to_ascii_uppercase();

    [
        "TB375FC",
        "TB373FU",
        "TB365FC",
        "TB361FU",
        "TB335FC",
        "TB336FU",
    ]
    .iter()
    .any(|supported| upper.contains(supported))
}

#[derive(Debug, Clone, Copy)]
struct LpmRomFolderValidationState {
    image_model_bad: bool,
    connected_device_unknown: bool,
    connected_device_bad: bool,
    connected_device_image_mismatch: bool,
    battery_low: bool,
    blocked_firmware: bool,
    mtk_driver_missing: bool,
    vc_runtime_missing: bool,
}

impl LpmRomFolderValidationState {
    fn has_error(&self) -> bool {
        self.image_model_bad
            || self.connected_device_unknown
            || self.connected_device_bad
            || self.connected_device_image_mismatch
            || self.battery_low
            || self.blocked_firmware
            || self.mtk_driver_missing
            || self.vc_runtime_missing
    }

    fn has_blocking_error_for_next_step(&self) -> bool {
        self.image_model_bad
            || self.connected_device_bad
            || self.connected_device_image_mismatch
            || self.battery_low
            || self.blocked_firmware
            || self.mtk_driver_missing
            || self.vc_runtime_missing
    }

    fn can_continue(&self) -> bool {
        !self.has_blocking_error_for_next_step()
    }
}

fn lpm_is_unknown_text(value: &str) -> bool {
    let value = value.trim();

    value.is_empty()
        || value == "알 수 없음"
        || value == "감지 전"
        || value.eq_ignore_ascii_case("unknown")
        || value.eq_ignore_ascii_case("unknown device")
        || value.eq_ignore_ascii_case("unknown model")
}

fn lpm_clean_display_value(value: &str) -> String {
    if lpm_is_unknown_text(value) {
        "알 수 없음".to_string()
    } else {
        value.trim().to_string()
    }
}


fn firmware_version_is_lower(image_version: Option<&str>, device_version: &str) -> bool {
    let Some(image_numbers) = extract_version_numbers(image_version.unwrap_or_default()) else {
        return false;
    };

    let Some(device_numbers) = extract_version_numbers(device_version) else {
        return false;
    };

    image_numbers < device_numbers
}

fn extract_version_numbers(value: &str) -> Option<Vec<u32>> {
    let mut numbers = Vec::new();
    let mut current = String::new();

    for ch in value.chars() {
        if ch.is_ascii_digit() {
            current.push(ch);
        } else if !current.is_empty() {
            if let Ok(number) = current.parse::<u32>() {
                numbers.push(number);
            }
            current.clear();
        }
    }

    if !current.is_empty() {
        if let Ok(number) = current.parse::<u32>() {
            numbers.push(number);
        }
    }

    if numbers.is_empty() {
        None
    } else {
        Some(numbers)
    }
}

fn spft_stage_label(stage: &str) -> String {
    stage.to_string()
}

fn progress_bar(percent: u8) -> String {
    let percent = percent.min(100);
    let total = 20usize;
    let filled = ((percent as usize * total) + 50) / 100;
    let empty = total.saturating_sub(filled);

    format!("[{}{}] {}%", "█".repeat(filled), "·".repeat(empty), percent)
}

fn lpm_nav_rom_validation_log_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(238, 238, 238))),
        text_color: Some(Color::from_rgb8(20, 20, 20)),
        border: iced::Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..container::Style::default()
    }
}

fn lpm_nav_disabled_next_button_style(
    _theme: &Theme,
    _status: iced::widget::button::Status,
) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb8(148, 148, 148))),
        text_color: Color::from_rgb8(255, 255, 255),
        border: iced::Border {
            radius: 4.0.into(),
            ..iced::Border::default()
        },
        ..button::Style::default()
    }
}

fn lpm_nav_rom_status_card_style(has_error: bool) -> container::Style {
    if has_error {
        container::Style {
            background: Some(Background::Color(Color::from_rgb8(255, 170, 170))),
            text_color: Some(Color::from_rgb8(32, 35, 47)),
            border: iced::Border {
                radius: 14.0.into(),
                width: 1.0,
                color: Color::from_rgb8(230, 138, 138),
            },
            ..container::Style::default()
        }
    } else {
        container::Style {
            background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
            text_color: Some(Color::from_rgb8(32, 35, 47)),
            border: iced::Border {
                radius: 14.0.into(),
                width: 1.0,
                color: Color::from_rgb8(222, 225, 236),
            },
            ..container::Style::default()
        }
    }
}



fn lpm_nav_rom_option_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border {
            radius: 12.0.into(),
            width: 1.0,
            color: Color::from_rgb8(222, 225, 236),
        },
        ..container::Style::default()
    }
}

fn lpm_nav_rom_country_popup_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border {
            radius: 18.0.into(),
            width: 1.0,
            color: Color::from_rgb8(222, 225, 236),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.22),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        ..container::Style::default()
    }
}

fn lpm_nav_rom_country_popup_scrim_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.35))),
        ..container::Style::default()
    }
}

fn lpm_nav_dashboard_update_popup_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border {
            radius: 8.0.into(),
            width: 1.0,
            color: Color::from_rgb8(222, 225, 236),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        ..container::Style::default()
    }
}

fn lpm_nav_dashboard_update_popup_scrim_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.48))),
        ..container::Style::default()
    }
}

fn lpm_nav_mtk_driver_popup_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border {
            radius: 13.0.into(),
            width: 2.0,
            color: Color::from_rgb8(177, 181, 198),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
            offset: iced::Vector::new(0.0, 10.0),
            blur_radius: 26.0,
        },
        ..container::Style::default()
    }
}

fn lpm_nav_mtk_driver_popup_scrim_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.48))),
        ..container::Style::default()
    }
}


fn lpm_nav_rom_routine_row_disabled_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 235, 235))),
        text_color: Some(Color::from_rgb8(120, 45, 45)),
        border: iced::Border {
            radius: 14.0.into(),
            width: 1.0,
            color: Color::from_rgb8(238, 150, 150),
        },
        ..container::Style::default()
    }
}

fn lpm_nav_rom_routine_row_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border {
            radius: 14.0.into(),
            width: 1.0,
            color: Color::from_rgb8(222, 225, 236),
        },
        ..container::Style::default()
    }
}

fn lpm_nav_rom_routine_slide_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border {
            radius: 14.0.into(),
            width: 1.0,
            color: Color::from_rgb8(222, 225, 236),
        },
        ..container::Style::default()
    }
}





fn lpm_nav_additional_option_button_style(
    _theme: &Theme,
    _status: iced::widget::button::Status,
) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Color::from_rgb8(32, 35, 47),
        border: iced::Border {
            radius: 14.0.into(),
            width: 1.0,
            color: Color::from_rgb8(222, 225, 236),
        },
        shadow: iced::Shadow {
            color: Color::TRANSPARENT,
            offset: iced::Vector::new(0.0, 0.0),
            blur_radius: 0.0,
        },
        ..button::Style::default()
    }
}

fn lpm_nav_flat_button_style(
    _theme: &Theme,
    _status: iced::widget::button::Status,
) -> button::Style {
    button::Style {
        background: None,
        text_color: Color::from_rgb8(32, 35, 47),
        border: iced::Border::default(),
        ..button::Style::default()
    }
}

fn lpm_nav_rom_select_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border {
            radius: 14.0.into(),
            width: 1.2,
            color: Color::from_rgb8(222, 225, 236),
        },
        ..container::Style::default()
    }
}

fn lpm_nav_rom_step_panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border {
            radius: 14.0.into(),
            width: 1.2,
            color: Color::from_rgb8(222, 225, 236),
        },
        ..container::Style::default()
    }
}

fn lpm_nav_rom_step_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border {
            radius: 12.0.into(),
            width: 1.0,
            color: Color::from_rgb8(222, 225, 236),
        },
        ..container::Style::default()
    }
}

fn lpm_nav_rom_error_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 246, 246))),
        text_color: Some(Color::from_rgb8(122, 32, 32)),
        border: iced::Border {
            radius: 12.0.into(),
            width: 1.0,
            color: Color::from_rgb8(240, 198, 198),
        },
        ..container::Style::default()
    }
}

fn lpm_nav_language_pick_list_style(
    _theme: &Theme,
    status: iced::widget::pick_list::Status,
) -> iced::widget::pick_list::Style {
    let active = iced::widget::pick_list::Style {
        text_color: Color::from_rgb8(32, 35, 47),
        placeholder_color: Color::from_rgb8(92, 96, 112),
        handle_color: Color::from_rgb8(32, 35, 47),
        background: Background::Color(Color::from_rgb8(227, 227, 227)),
        border: iced::Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
    };

    match status {
        iced::widget::pick_list::Status::Active => active,
        iced::widget::pick_list::Status::Hovered | iced::widget::pick_list::Status::Opened { .. } => {
            iced::widget::pick_list::Style {
                background: Background::Color(Color::from_rgb8(227, 227, 227)),
                ..active
            }
        }
    }
}

fn lpm_nav_language_pick_list_menu_style(_theme: &Theme) -> iced::widget::overlay::menu::Style {
    iced::widget::overlay::menu::Style {
        background: Background::Color(Color::from_rgb8(255, 255, 255)),
        border: iced::Border {
            radius: 0.0.into(),
            width: 1.0,
            color: Color::from_rgb8(222, 225, 236),
        },
        text_color: Color::from_rgb8(32, 35, 47),
        selected_text_color: Color::from_rgb8(32, 35, 47),
        selected_background: Background::Color(Color::from_rgb8(238, 240, 250)),
        shadow: iced::Shadow::default(),
    }
}

fn lpm_nav_settings_move_button_style(
    _theme: &Theme,
    _status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: Some(Background::Color(Color::from_rgb8(84, 91, 241))),
        text_color: Color::from_rgb8(255, 255, 255),
        border: iced::Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: iced::Shadow {
            color: Color::TRANSPARENT,
            offset: iced::Vector::new(0.0, 0.0),
            blur_radius: 0.0,
        },
        ..iced::widget::button::Style::default()
    }
}
fn lpm_nav_rom_card_button_style(
    _theme: &Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let shadow_offset = if matches!(status, iced::widget::button::Status::Pressed) {
        iced::Vector::new(0.0, 0.0)
    } else {
        iced::Vector::new(0.0, 0.0)
    };

    iced::widget::button::Style {
        background: None,
        text_color: Color::from_rgb8(25, 27, 36),
        border: iced::Border::default(),
        shadow: iced::Shadow {
            color: Color::TRANSPARENT,
            offset: shadow_offset,
            blur_radius: 0.0,
        },
        ..iced::widget::button::Style::default()
    }
}

fn lpm_nav_dashboard_black_screen_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(0, 0, 0))),
        text_color: Some(Color::from_rgb8(255, 255, 255)),
        border: iced::Border {
            radius: 8.0.into(),
            ..iced::Border::default()
        },
        ..container::Style::default()
    }
}

fn lpm_nav_app_background_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(248, 248, 252))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        ..container::Style::default()
    }
}

fn lpm_nav_menu_panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(246, 246, 251))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        ..container::Style::default()
    }
}

fn lpm_nav_section_label_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(239, 239, 246))),
        text_color: Some(Color::from_rgb8(113, 116, 130)),
        ..container::Style::default()
    }
}

fn lpm_nav_divider_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(210, 212, 224))),
        ..container::Style::default()
    }
}

fn lpm_nav_footer_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(246, 246, 251))),
        text_color: Some(Color::from_rgb8(45, 48, 62)),
        ..container::Style::default()
    }
}

fn lpm_nav_header_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(28, 31, 45)),
        border: iced::Border {
            radius: 0.0.into(),
            width: 1.2,
            color: Color::from_rgb8(222, 225, 236),
        },
        ..container::Style::default()
    }
}

fn lpm_nav_settings_panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border {
            radius: 12.0.into(),
            width: 1.0,
            color: Color::from_rgb8(222, 225, 236),
        },
        ..container::Style::default()
    }
}

fn lpm_nav_panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        ..container::Style::default()
    }
}


fn lpm_nav_extra_option_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border {
            radius: 14.0.into(),
            width: 1.0,
            color: Color::from_rgb8(222, 225, 236),
        },
        ..container::Style::default()
    }
}


fn lpm_nav_status_idle_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(222, 244, 231))),
        text_color: Some(Color::from_rgb8(23, 104, 59)),
        ..container::Style::default()
    }
}

fn lpm_nav_status_busy_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 238, 214))),
        text_color: Some(Color::from_rgb8(150, 82, 20)),
        ..container::Style::default()
    }
}


fn lpm_nav_dashboard_inner_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border {
            radius: 12.0.into(),
            width: 1.0,
            color: Color::from_rgb8(222, 225, 236),
        },
        ..container::Style::default()
    }
}

fn lpm_nav_dashboard_action_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 255, 255))),
        text_color: Some(Color::from_rgb8(32, 35, 47)),
        border: iced::Border {
            radius: 12.0.into(),
            width: 1.0,
            color: Color::from_rgb8(210, 214, 228),
        },
        ..container::Style::default()
    }
}

fn log_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(238, 238, 238))),
        text_color: Some(Color::from_rgb8(20, 20, 20)),
        ..container::Style::default()
    }
}

async fn check_program_update_worker(
    current_version: String,
) -> Result<ProgramUpdateCheckResult, String> {
    check_program_update_blocking(&current_version)
}

fn check_program_update_blocking(current_version: &str) -> Result<ProgramUpdateCheckResult, String> {
    let response = ureq::get(LPMBOX_RELEASES_API_URL)
        .set("User-Agent", "LPMBox")
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|err| format!("GitHub 릴리즈 정보를 가져오지 못했습니다: {err}"))?;

    let body = response
        .into_string()
        .map_err(|err| format!("GitHub 응답을 읽지 못했습니다: {err}"))?;

    let releases: serde_json::Value = serde_json::from_str(&body)
        .map_err(|err| format!("GitHub 응답 JSON 파싱 실패: {err}"))?;

    let releases = releases
        .as_array()
        .ok_or_else(|| "GitHub 릴리즈 응답 형식이 예상과 다릅니다.".to_string())?;

    let mut latest_tag: Option<String> = None;
    let mut latest_url: Option<String> = None;
    let mut latest_asset_name: Option<String> = None;
    let mut latest_version: Option<semver::Version> = None;

    for release in releases {
        if release
            .get("draft")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            continue;
        }

        if release
            .get("prerelease")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            continue;
        }

        let Some(tag) = release.get("tag_name").and_then(|value| value.as_str()) else {
            continue;
        };

        let Some(parsed) = parse_lpm_version(tag) else {
            continue;
        };

        if latest_version.as_ref().map_or(true, |version| parsed > *version) {
            latest_version = Some(parsed);
            latest_tag = Some(tag.to_string());
            latest_url = release
                .get("html_url")
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned);
            latest_asset_name = release
                .get("assets")
                .and_then(|value| value.as_array())
                .and_then(|assets| find_lpm_release_zip_asset_name(assets.as_slice()));
        }
    }

    let latest_version_text = latest_tag
        .clone()
        .ok_or_else(|| "확인 가능한 LPMBox 릴리즈 버전을 찾지 못했습니다.".to_string())?;

    let current_parsed = parse_lpm_version(current_version)
        .ok_or_else(|| format!("현재 버전 값을 해석하지 못했습니다: {current_version}"))?;

    let latest_parsed = latest_version
        .ok_or_else(|| "최신 릴리즈 버전을 해석하지 못했습니다.".to_string())?;

    Ok(ProgramUpdateCheckResult {
        current_version: current_version.to_string(),
        latest_version: latest_version_text,
        update_available: latest_parsed > current_parsed,
        release_url: latest_url.unwrap_or_else(|| LPMBOX_RELEASES_URL.to_string()),
        asset_name: latest_asset_name,
    })
}

fn find_lpm_release_zip_asset_name(assets: &[serde_json::Value]) -> Option<String> {
    let mut candidates: Vec<(i32, String)> = Vec::new();

    for asset in assets {
        let Some(name) = asset.get("name").and_then(|value| value.as_str()) else {
            continue;
        };

        let lower = name.to_ascii_lowercase();

        if !lower.ends_with(".zip") || lower.contains("source") {
            continue;
        }

        let mut score = 0;

        if lower.contains("lpmbox") {
            score += 10;
        }

        for token in ["win", "windows", "x64", "amd64", "x86", "x86-x64", "x86_x64"] {
            if lower.contains(token) {
                score += 2;
            }
        }

        candidates.push((score, name.to_string()));
    }

    candidates.sort_by(|left, right| right.0.cmp(&left.0));
    candidates.into_iter().map(|(_, name)| name).next()
}

fn parse_lpm_version(value: &str) -> Option<semver::Version> {
    let start = value.find(|ch: char| ch.is_ascii_digit())?;
    let mut candidate = String::new();

    for ch in value[start..].chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '+') {
            candidate.push(ch);
        } else {
            break;
        }
    }

    if candidate.is_empty() {
        return None;
    }

    let (core, suffix) = split_semver_suffix(&candidate);
    let mut parts = core.split('.').collect::<Vec<_>>();

    while parts.len() < 3 {
        parts.push("0");
    }

    if parts.len() > 3 {
        return None;
    }

    let normalized = format!("{}{}", parts.join("."), suffix);
    semver::Version::parse(&normalized).ok()
}

fn split_semver_suffix(value: &str) -> (&str, &str) {
    let dash = value.find('-');
    let plus = value.find('+');

    match (dash, plus) {
        (Some(left), Some(right)) => {
            let index = left.min(right);
            (&value[..index], &value[index..])
        }
        (Some(index), None) | (None, Some(index)) => (&value[..index], &value[index..]),
        (None, None) => (value, ""),
    }
}

fn current_timestamp() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

fn preview_patch_plans(plans: &[PatchPlan]) -> String {
    if plans.is_empty() {
        return "-".to_string();
    }

    plans
        .iter()
        .map(|plan| {
            let status = if plan.available { "사용 가능" } else { "사용 불가" };

            let warning_text = if plan.warnings.is_empty() {
                "".to_string()
            } else {
                format!(", 경고 {}개", plan.warnings.len())
            };

            format!(
                "{}: {} / actions {}개{}",
                plan.title,
                status,
                plan.actions.len(),
                warning_text
            )
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

fn preview_patched_snapshots(snapshots: &[PatchedPartitionSnapshot]) -> String {
    if snapshots.is_empty() {
        return "-".to_string();
    }

    snapshots
        .iter()
        .map(|snapshot| {
            let status = if snapshot.available { "적용됨" } else { "미적용" };

            format!(
                "{}: {} / 변경 {}개",
                snapshot.title, status, snapshot.changed_count
            )
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

fn patch_validation_summary(validations: &[PatchValidationResult]) -> String {
    if validations.is_empty() {
        return "확인 불가".to_string();
    }

    let passed_count = validations
        .iter()
        .filter(|validation| validation.passed)
        .count();

    let failed_count = validations.len().saturating_sub(passed_count);

    if failed_count == 0 {
        format!("성공 / {}개 모두 통과", validations.len())
    } else {
        format!("실패 / 통과 {}개 / 실패 {}개", passed_count, failed_count)
    }
}

fn preview_patch_validations(validations: &[PatchValidationResult]) -> String {
    if validations.is_empty() {
        return "-".to_string();
    }

    validations
        .iter()
        .map(|validation| {
            let status = if validation.passed { "통과" } else { "실패" };

            let error_text = if validation.errors.is_empty() {
                "".to_string()
            } else {
                format!(", 오류 {}개", validation.errors.len())
            };

            let warning_text = if validation.warnings.is_empty() {
                "".to_string()
            } else {
                format!(", 경고 {}개", validation.warnings.len())
            };

            format!(
                "{}: {}{}{}",
                validation.title, status, error_text, warning_text
            )
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

fn required_check_label(check: &RequiredPartitionCheck) -> &'static str {
    if check.all_required_ok { "성공" } else { "누락 있음" }
}

fn required_check_detail(check: &RequiredPartitionCheck) -> String {
    if check.all_required_ok {
        return format!(
            "proinfo={}, userdata={}, super={}, boot_ab={}, vendor_boot_ab={}, init_boot_ab={}, vbmeta_ab={}, vbmeta_system_ab={}, vbmeta_vendor_ab={}, lk_ab={}, dtbo_ab={}",
            yes_no(check.has_proinfo),
            yes_no(check.has_userdata),
            yes_no(check.has_super),
            yes_no(check.has_boot_ab),
            yes_no(check.has_vendor_boot_ab),
            yes_no(check.has_init_boot_ab),
            yes_no(check.has_vbmeta_ab),
            yes_no(check.has_vbmeta_system_ab),
            yes_no(check.has_vbmeta_vendor_ab),
            yes_no(check.has_lk_ab),
            yes_no(check.has_dtbo_ab),
        );
    }

    format!("누락: {}", check.missing_partitions.join(", "))
}

fn visual_wrap(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return text.to_string();
    }

    let mut result = String::new();
    let mut count = 0usize;

    for ch in text.chars() {
        if ch == '\n' {
            result.push(ch);
            count = 0;
            continue;
        }

        if count >= max_chars {
            result.push('\n');
            count = 0;
        }

        result.push(ch);
        count += 1;
    }

    result
}

fn compact_text(text: &str, max_chars: usize) -> String {
    let char_count = text.chars().count();

    if char_count <= max_chars {
        return text.to_string();
    }

    let keep_front = max_chars / 2;
    let keep_back = max_chars.saturating_sub(keep_front + 5);

    let front: String = text.chars().take(keep_front).collect();

    let back: String = text
        .chars()
        .rev()
        .take(keep_back)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    format!("{front} ... {back}")
}


fn yes_no(value: bool) -> &'static str {
    if value { "있음" } else { "없음" }
}

fn region_label(region: RomRegion) -> &'static str {
    match region {
        RomRegion::Prc => "PRC(중국 내수롬)",
        RomRegion::Row => "ROW(글로벌롬)",
        RomRegion::Unknown => "Unknown",
    }
}

#[cfg(test)]
mod lpm_i18n_tests {
    use super::*;

    #[test]
    fn initial_language_prefers_saved_config() {
        let (lang, source) = resolve_initial_language_option(Some("ja"), Some("ko-KR"));
        assert_eq!(lang, LanguageOption::Japanese);
        assert_eq!(source, InitialLanguageSource::SavedConfig);
    }

    #[test]
    fn initial_language_uses_os_locale_when_no_saved_config() {
        let (lang, source) = resolve_initial_language_option(None, Some("ko-KR"));
        assert_eq!(lang, LanguageOption::Korean);
        assert_eq!(source, InitialLanguageSource::WindowsOsLocale);

        let (lang, source) = resolve_initial_language_option(None, Some("ja-JP"));
        assert_eq!(lang, LanguageOption::Japanese);
        assert_eq!(source, InitialLanguageSource::WindowsOsLocale);
    }

    #[test]
    fn initial_language_falls_back_to_english_for_unsupported_locale() {
        let (lang, source) = resolve_initial_language_option(None, Some("fr-FR"));
        assert_eq!(lang, LanguageOption::English);
        assert_eq!(source, InitialLanguageSource::DefaultEnglish);
    }

    #[test]
    fn saved_language_code_aliases_are_supported() {
        assert_eq!(LanguageOption::from_code("ko-KR"), Some(LanguageOption::Korean));
        assert_eq!(LanguageOption::from_code("zh-TW"), Some(LanguageOption::TraditionalChinese));
        assert_eq!(LanguageOption::from_code("jp"), Some(LanguageOption::Japanese));
        assert_eq!(LanguageOption::from_code("es"), Some(LanguageOption::Spanish));
    }

    #[test]
    fn canonical_language_codes_are_written() {
        assert_eq!(LanguageOption::Japanese.code(), "jp");
        assert_eq!(LanguageOption::TraditionalChinese.code(), "zh-TW");
    }

    #[test]
    fn simplified_chinese_locales_fall_back_to_english() {
        assert_eq!(LanguageOption::from_locale("zh-CN"), Some(LanguageOption::English));
        assert_eq!(LanguageOption::from_locale("zh-Hans"), Some(LanguageOption::English));
        assert_eq!(LanguageOption::from_locale("zh-TW"), Some(LanguageOption::TraditionalChinese));
    }

    #[test]
    fn localization_purity_gate_removes_hangul_for_non_korean_languages() {
        for language in LANGUAGE_OPTIONS {
            if language.is_korean() {
                continue;
            }

            let output = enforce_selected_language_output(
                language,
                "ADB 기기 감지 실패: TB375FC",
                "ADB 기기 감지 실패: TB375FC".to_string(),
            );
            assert!(!contains_hangul(&output));
            assert!(output.contains("TB375FC"));
        }
    }

    #[test]
    fn spinner_control_messages_never_use_public_marker_text() {
        let message = internal_spinner_message("preloader_detect", "PreLoader");
        let public_marker = ["__", "SPINNER", "__"].concat();
        assert!(!message.contains(&public_marker));
        assert_eq!(
            parse_spinner_log(&message),
            Some(("preloader_detect".to_string(), "PreLoader".to_string()))
        );
    }

    #[test]
    fn v312_runtime_messages_are_fully_localized() {
        let messages = [
            "[ADB] adb reboot 1회 실행 중...",
            "[ADB] adb reboot 1회 실행 완료",
            "[ADB] adb reboot 1회 실행 실패, Fastboot 명령을 계속 시도합니다.",
            "[Fastboot] fastboot reboot 1회 실행 중...",
            "[Fastboot] fastboot reboot 1회 실행 완료",
            "[Fastboot] fastboot reboot 1회 실행 실패, PreLoader 감지를 계속합니다.",
            "[Image] 온라인 차단 펌웨어 규칙을 확인합니다: https://example.invalid/rules.ini",
            "[Image] 온라인 차단 펌웨어 규칙 확인 완료: 128 bytes",
            "온라인 차단 펌웨어 규칙 확인 실패: 파일을 찾을 수 없습니다: 차단 펌웨어 규칙 응답이 비어 있습니다.",
        ];

        for language in LANGUAGE_OPTIONS {
            if language.is_korean() {
                continue;
            }

            for message in messages {
                let translated = lpm_translate_v312_runtime_message(language, message)
                    .expect("v3.1.2 runtime message translation");
                assert!(!contains_hangul(&translated));
            }
        }
    }
    #[test]
    fn japanese_language_code_and_label_use_jp() {
        assert_eq!(LanguageOption::Japanese.code(), "jp");
        assert_eq!(LanguageOption::from_code("jp"), Some(LanguageOption::Japanese));
        assert_eq!(LanguageOption::from_code("ja"), Some(LanguageOption::Japanese));
        assert_eq!(LanguageOption::Japanese.to_string(), "日本語 (jp)");
    }

    #[test]
    fn localized_whitespace_cleanup_preserves_explicit_line_breaks() {
        let output = normalize_localized_whitespace_preserving_lines(
            "First  line\nSecond   line\nThird line".to_string(),
        );
        assert_eq!(output, "First line\nSecond line\nThird line");
    }

    #[test]
    fn known_runtime_logs_do_not_leave_hangul_in_non_korean_languages() {
        let messages = [
            "[1단계] ADB 기기 감지 및 기기 정보 확인",
            "[Image] image 폴더에 lk_a, lk_b, dtbo_a, dtbo_b 파일을 복사합니다.",
            "[Image] TB375FC lk, dtbo 파일을 다운로드 합니다.",
            "[Plan] 작업 scatter XML 재파싱: 성공 / root: root / partition 86개",
            "[Log] PRC ↔ ROW 설치 작업 로그를 C:\\Temp\\log.txt에 저장합니다.",
            "TB375FC ZUI_17.5.10.060_ST 버전은 설치 금지 목록에 포함되어 있습니다.",
        ];

        for language in LANGUAGE_OPTIONS {
            if language.is_korean() {
                continue;
            }

            for message in messages {
                let translated = lpm_translate_known_runtime_log(language, message)
                    .expect("known runtime log translation");
                assert!(
                    !contains_hangul(&translated),
                    "{language:?}: {translated}"
                );
            }
        }
    }

    #[test]
    fn spinner_frames_match_gbst_and_replace_by_key() {
        assert_eq!(UI_SPINNER_FRAMES, ["│", "╱", "━", "╲"]);
        for frame in UI_SPINNER_FRAMES {
            let (base, active) = split_spinner_message(&format!("[ADB] Detecting... {frame}"));
            assert!(active);
            assert_eq!(base, "[ADB] Detecting...");
        }
    }

}
