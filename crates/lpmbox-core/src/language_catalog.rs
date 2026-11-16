// SPDX-License-Identifier: GPL-3.0-only
//! GBST v2.0.0 style locale resources for common LPMBox UI labels.
//! Existing firmware-operation translations remain available as fallback.
use std::collections::HashMap;
use std::sync::OnceLock;
const CATALOGS: [(&str, &str); 12] = [
    ("en", include_str!("../lang/en.json")),
    ("ko", include_str!("../lang/ko.json")),
    ("ja", include_str!("../lang/ja.json")),
    ("zh-TW", include_str!("../lang/zh-TW.json")),
    ("ru", include_str!("../lang/ru.json")),
    ("vi", include_str!("../lang/vi.json")),
    ("es", include_str!("../lang/es.json")),
    ("el", include_str!("../lang/el.json")),
    ("hi", include_str!("../lang/hi.json")),
    ("ka", include_str!("../lang/ka.json")),
    ("nl", include_str!("../lang/nl.json")),
    ("ar", include_str!("../lang/ar.json")),
];
const LEGACY_CATALOGS: [(&str, &str); 12] = [
    ("en", include_str!("../lang/legacy-en.json")),
    ("ko", include_str!("../lang/legacy-ko.json")),
    ("ja", include_str!("../lang/legacy-ja.json")),
    ("zh-TW", include_str!("../lang/legacy-zh-TW.json")),
    ("ru", include_str!("../lang/legacy-ru.json")),
    ("vi", include_str!("../lang/legacy-vi.json")),
    ("es", include_str!("../lang/legacy-es.json")),
    ("el", include_str!("../lang/legacy-el.json")),
    ("hi", include_str!("../lang/legacy-hi.json")),
    ("ka", include_str!("../lang/legacy-ka.json")),
    ("nl", include_str!("../lang/legacy-nl.json")),
    ("ar", include_str!("../lang/legacy-ar.json")),
];
static LEGACY_RESOURCES: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();
fn legacy_resources() -> &'static HashMap<&'static str, HashMap<String, String>> {
    LEGACY_RESOURCES.get_or_init(|| LEGACY_CATALOGS.into_iter().map(|(locale, json)| {
        (locale, serde_json::from_str(json).expect("Valid LPMBox legacy locale JSON"))
    }).collect())
}
pub fn translate_legacy(language: &str, source_korean: &str) -> Option<String> {
    let locale = if language == "jp" { "ja" } else { language };
    legacy_resources().get(locale).and_then(|map| map.get(source_korean)).cloned()
}
const FIXED_CATALOGS: [(&str, &str); 12] = [
    ("en", include_str!("../lang/fixed-en.json")),
    ("ko", include_str!("../lang/fixed-ko.json")),
    ("ja", include_str!("../lang/fixed-ja.json")),
    ("zh-TW", include_str!("../lang/fixed-zh-TW.json")),
    ("ru", include_str!("../lang/fixed-ru.json")),
    ("vi", include_str!("../lang/fixed-vi.json")),
    ("es", include_str!("../lang/fixed-es.json")),
    ("el", include_str!("../lang/fixed-el.json")),
    ("hi", include_str!("../lang/fixed-hi.json")),
    ("ka", include_str!("../lang/fixed-ka.json")),
    ("nl", include_str!("../lang/fixed-nl.json")),
    ("ar", include_str!("../lang/fixed-ar.json")),
];
static FIXED_RESOURCES: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();
fn fixed_resources() -> &'static HashMap<&'static str, HashMap<String, String>> {
    FIXED_RESOURCES.get_or_init(|| FIXED_CATALOGS.into_iter().map(|(locale, json)| {
        (locale, serde_json::from_str(json).expect("Valid LPMBox fixed-text locale JSON"))
    }).collect())
}
/// Returns the selected locale's resource for an original fixed Korean string.
/// Dynamic device output and incomplete translations continue through the legacy runtime path.
pub fn translate_fixed(language: &str, original: &str) -> Option<String> {
    let locale = if language == "jp" { "ja" } else { language };
    fixed_resources().get(locale).and_then(|map| map.get(original))
        .filter(|text| !text.is_empty()).cloned()
}

static RESOURCES: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();
fn resources() -> &'static HashMap<&'static str, HashMap<String, String>> {
    RESOURCES.get_or_init(|| CATALOGS.into_iter().map(|(locale, json)| {
        (locale, serde_json::from_str(json).expect("Valid LPMBox locale JSON"))
    }).collect())
}
pub fn translate(language: &str, key: &str) -> String {
    let locale = if language == "jp" { "ja" } else { language };
    resources().get(locale).and_then(|map| map.get(key))
        .or_else(|| resources().get("en").and_then(|map| map.get(key)))
        .cloned().unwrap_or_else(|| key.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_language_catalogs_have_matching_keys() {
        let first = resources().get("en").unwrap();
        for (name, dictionary) in resources() {
            assert_eq!(dictionary.len(), first.len(), "{name}");
            for key in first.keys() { assert!(dictionary.contains_key(key), "{name}: {key}"); }
        }
    }
    #[test]
    fn legacy_json_translations_are_complete() {
        let first = legacy_resources().get("en").unwrap();
        assert!(first.len() >= 150);
        for (name, dictionary) in legacy_resources() {
            assert_eq!(dictionary.len(), first.len(), "{name}");
            for key in first.keys() { assert!(dictionary.contains_key(key), "{name}: {key}"); }
        }
    }
    #[test]
    fn fixed_json_catalogs_cover_all_source_literals() {
        let first = fixed_resources().get("ko").unwrap();
        assert!(first.len() >= 1500, "Expected all fixed source strings");
        for (name, dictionary) in fixed_resources() {
            assert_eq!(dictionary.len(), first.len(), "fixed locale {name}");
            for key in first.keys() { assert!(dictionary.contains_key(key), "{name}: {key}"); }
        }
        assert!(translate_fixed("jp", "이미 작업이 진행 중입니다.").is_some());
    }
    // Inspired by LTBox 3.3.5's locale_guards: translation must never
    // corrupt named formatting placeholders used by device/status messages.
    fn named_placeholders(input: &str) -> Vec<String> {
        let mut params = Vec::new();
        for tail in input.split('{').skip(1) {
            if let Some((name, _)) = tail.split_once('}') {
                let mut chars = name.chars();
                if chars.next().is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
                    && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
                {
                    params.push(name.to_string());
                }
            }
        }
        params
    }

    #[test]
    fn all_localized_named_placeholders_match_english() {
        for source in [
            resources(),
            fixed_resources(),
            legacy_resources(),
        ] {
            let english = source.get("en").unwrap();
            for (locale, dictionary) in source {
                for (key, default_text) in english {
                    let translated = dictionary.get(key)
                        .unwrap_or_else(|| panic!("missing {locale}/{key}"));
                    assert_eq!(
                        named_placeholders(default_text),
                        named_placeholders(translated),
                        "placeholder changed in {locale}/{key}"
                    );
                }
            }
        }
    }

    #[test]
    fn legacy_japanese_code_and_fallback() {
        assert_eq!(translate("jp", "appearance.theme"), "テーマ");
        assert_eq!(translate("invalid", "appearance.theme"), "Theme");
    }

    #[test]
    fn rom_ui_fixed_titles_and_recovery_lk_notice_have_all_translations() {
        let titles = [
            "image 폴더 선택",
            "선택한 image 폴더 정보",
            "작업 선택",
            "로그 관리",
            "ROM 작업",
            "기기 복구",
            "아래 작업을 선택하여 기기에 적용합니다.",
            "image 폴더를 선택해주세요.",
            "작업 로그를 확인하고 텍스트 파일로 저장합니다.",
            "[Recovery LK] 일반 ROM 설치와 동일한 모델·지역·버전별 LK 패치 정책을 적용합니다.",
        ];
        for (locale, dictionary) in fixed_resources() {
            for source in titles {
                let value = dictionary.get(source).unwrap_or_else(||
                    panic!("missing fixed text {locale} / {source}")
                );
                assert!(!value.trim().is_empty(), "empty {locale} / {source}");
                if *locale != "ko" {
                    assert_ne!(value.as_str(), source,
                        "untranslated fixed UI string {locale} / {source}");
                }
            }
        }
    }
}
