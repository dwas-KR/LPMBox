// SPDX-License-Identifier: GPL-3.0-only
//! User-facing GBST appearance preferences. No device operation is performed.
use iced::theme::Palette;
use iced::{Color, Theme};
use std::fmt;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeChoice {
    System,
    Light,
    Dark,
}
impl ThemeChoice {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];
    pub fn code(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
    pub fn from_code(s: &str) -> Self {
        match s {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::System,
        }
    }
    pub fn localized(self, lang: &str) -> &'static str {
        match (lang, self) {
            ("ko", Self::System) => "시스템 설정",
            ("ko", Self::Light) => "라이트",
            ("ko", Self::Dark) => "다크",
            ("ja", Self::System) => "システム",
            ("ja", Self::Light) => "ライト",
            ("ja", Self::Dark) => "ダーク",
            ("zh-TW", Self::System) => "系統設定",
            ("zh-TW", Self::Light) => "淺色",
            ("zh-TW", Self::Dark) => "深色",
            (_, Self::System) => "System",
            (_, Self::Light) => "Light",
            (_, Self::Dark) => "Dark",
        }
    }
}
impl fmt::Display for ThemeChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccentChoice {
    Indigo,
    Teal,
    Rose,
}
impl AccentChoice {
    pub const ALL: [Self; 3] = [Self::Indigo, Self::Teal, Self::Rose];
    pub fn code(self) -> &'static str {
        match self {
            Self::Indigo => "indigo",
            Self::Teal => "teal",
            Self::Rose => "rose",
        }
    }
    pub fn from_code(s: &str) -> Self {
        match s {
            "teal" => Self::Teal,
            "rose" => Self::Rose,
            _ => Self::Indigo,
        }
    }
    pub fn localized(self, lang: &str) -> &'static str {
        match (lang, self) {
            ("ko", Self::Indigo) => "인디고",
            ("ko", Self::Teal) => "틸",
            ("ko", Self::Rose) => "로즈",
            ("ja", Self::Indigo) => "インディゴ",
            ("ja", Self::Teal) => "ティール",
            ("ja", Self::Rose) => "ローズ",
            ("zh-TW", Self::Indigo) => "靛藍",
            ("zh-TW", Self::Teal) => "藍綠",
            ("zh-TW", Self::Rose) => "玫瑰",
            (_, Self::Indigo) => "Indigo",
            (_, Self::Teal) => "Teal",
            (_, Self::Rose) => "Rose",
        }
    }
    pub fn color(self) -> Color {
        match self {
            Self::Indigo => Color::from_rgb8(70, 90, 170),
            Self::Teal => Color::from_rgb8(20, 139, 145),
            Self::Rose => Color::from_rgb8(178, 75, 122),
        }
    }
}
impl fmt::Display for AccentChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    pub theme: ThemeChoice,
    pub accent: AccentChoice,
}
impl Default for Appearance {
    fn default() -> Self {
        Self {
            theme: ThemeChoice::System,
            accent: AccentChoice::Indigo,
        }
    }
}

pub fn settings_file() -> std::path::PathBuf {
    lpmbox_core::app_paths::lpmbox_data_root().join("appearance.json")
}
impl Appearance {
    pub fn load() -> Self {
        let Ok(data) = fs::read_to_string(settings_file()) else {
            return Self::default();
        };
        let Ok(obj) = serde_json::from_str::<serde_json::Value>(&data) else {
            return Self::default();
        };
        Self {
            theme: ThemeChoice::from_code(
                obj.get("theme")
                    .and_then(|v| v.as_str())
                    .unwrap_or("system"),
            ),
            accent: AccentChoice::from_code(
                obj.get("accent")
                    .and_then(|v| v.as_str())
                    .unwrap_or("indigo"),
            ),
        }
    }
    pub fn save(&self) -> std::io::Result<()> {
        let path = settings_file();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?
        }
        let temp = path.with_extension("json.tmp");
        let value = serde_json::json!({"theme":self.theme.code(),"accent":self.accent.code()});
        fs::write(&temp, serde_json::to_vec_pretty(&value)?)?;
        if let Err(err) = fs::rename(&temp, &path) {
            if path.is_file() {
                fs::remove_file(&path)?;
                fs::rename(&temp, &path)?;
            } else {
                return Err(err);
            }
        }
        Ok(())
    }
    pub fn as_theme(&self, system_dark: bool) -> Theme {
        let dark = match self.theme {
            ThemeChoice::System => system_dark,
            ThemeChoice::Dark => true,
            ThemeChoice::Light => false,
        };
        let mut palette = if dark { Palette::DARK } else { Palette::LIGHT };
        palette.background = if dark {
            Color::from_rgb8(27, 30, 42)
        } else {
            Color::from_rgb8(251, 248, 253)
        };
        palette.text = if dark {
            Color::from_rgb8(228, 225, 233)
        } else {
            Color::from_rgb8(27, 27, 33)
        };
        palette.primary = self.accent.color();
        Theme::custom(
            format!("LPMBox-{}-{}", self.theme.code(), self.accent.code()),
            palette,
        )
    }
}

/// Reads Windows AppsUseLightTheme once at startup (no elevated privileges).
#[cfg(target_os = "windows")]
pub fn system_prefers_dark() -> bool {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    std::process::Command::new("reg.exe")
        .creation_flags(CREATE_NO_WINDOW)
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
            "/v",
            "AppsUseLightTheme",
        ])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .is_some_and(|output| {
            let raw = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
            raw.lines().any(|line| {
                line.contains("appsuselighttheme")
                    && (line.contains("0x0") || line.ends_with("    0"))
            })
        })
}
#[cfg(not(target_os = "windows"))]
pub fn system_prefers_dark() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_and_invalid_values_are_safe() {
        assert_eq!(ThemeChoice::from_code("???"), ThemeChoice::System);
        assert_eq!(AccentChoice::from_code("???"), AccentChoice::Indigo);
    }
    #[test]
    fn colors_are_distinct() {
        assert_ne!(AccentChoice::Indigo.color(), AccentChoice::Rose.color());
        assert_ne!(AccentChoice::Rose.color(), AccentChoice::Teal.color());
    }
}
