use crate::settings::{Mode, Settings};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub mode: String,
    pub device: Option<String>,
    pub interval: u64,
    pub format: String,
    pub osc_host: String,
    pub osc_port: u16,
    pub osc_address: String,
    pub osc_offset_address: String,
    pub dry_run: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config::from(&Settings::default())
    }
}

impl From<&Settings> for Config {
    fn from(s: &Settings) -> Self {
        Config {
            mode: match s.mode {
                Mode::Once => "once",
                Mode::Watch => "watch",
            }
            .to_string(),
            device: s.device.clone(),
            interval: s.interval,
            format: s.format.clone(),
            osc_host: s.osc_host.clone(),
            osc_port: s.osc_port,
            osc_address: s.osc_address.clone(),
            osc_offset_address: s.osc_offset_address.clone(),
            dry_run: s.dry_run,
        }
    }
}

impl Config {
    pub fn into_settings(self) -> Settings {
        Settings {
            mode: if self.mode == "once" {
                Mode::Once
            } else {
                Mode::Watch
            },
            device: self.device,
            interval: self.interval,
            format: self.format,
            osc_host: self.osc_host,
            osc_port: self.osc_port,
            osc_address: self.osc_address,
            osc_offset_address: self.osc_offset_address,
            dry_run: self.dry_run,
        }
    }
}

pub fn config_path_from(xdg: Option<&OsStr>, home: Option<&OsStr>) -> Option<PathBuf> {
    if let Some(x) = xdg {
        if !x.is_empty() {
            return Some(PathBuf::from(x).join("ziti").join("config.toml"));
        }
    }
    let home = home?;
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("ziti")
            .join("config.toml"),
    )
}

fn config_path() -> Option<PathBuf> {
    config_path_from(
        std::env::var_os("XDG_CONFIG_HOME").as_deref(),
        std::env::var_os("HOME").as_deref(),
    )
}

pub fn load() -> Settings {
    let Some(path) = config_path() else {
        return Settings::default();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Settings::default();
    };
    match toml::from_str::<Config>(&text) {
        Ok(config) => config.into_settings(),
        Err(_) => Settings::default(),
    }
}

pub fn save(settings: &Settings) -> Result<()> {
    let path = config_path().context("config パスを解決できません（HOME 未設定）")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("ディレクトリ作成に失敗: {}", parent.display()))?;
    }
    let text =
        toml::to_string_pretty(&Config::from(settings)).context("config のシリアライズに失敗")?;
    std::fs::write(&path, text).with_context(|| format!("書き込みに失敗: {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{Mode, Settings};
    use std::ffi::OsStr;

    #[test]
    fn path_prefers_xdg_when_set() {
        let p = config_path_from(Some(OsStr::new("/x/cfg")), Some(OsStr::new("/home/u"))).unwrap();
        assert_eq!(p, std::path::PathBuf::from("/x/cfg/ziti/config.toml"));
    }

    #[test]
    fn path_falls_back_to_home_dot_config() {
        let p = config_path_from(Some(OsStr::new("")), Some(OsStr::new("/home/u"))).unwrap();
        assert_eq!(
            p,
            std::path::PathBuf::from("/home/u/.config/ziti/config.toml")
        );
    }

    #[test]
    fn path_is_none_without_home() {
        assert!(config_path_from(None, None).is_none());
    }

    #[test]
    fn default_config_round_trips_to_default_settings() {
        let settings = Config::default().into_settings();
        assert_eq!(settings, Settings::default());
    }

    #[test]
    fn settings_survive_toml_round_trip() {
        let original = Settings {
            mode: Mode::Once,
            device: Some("coreaudio:UID".to_string()),
            interval: 12,
            format: "{title} / {artist}".to_string(),
            osc_host: "10.0.0.2".to_string(),
            osc_port: 9000,
            osc_address: "/x/y".to_string(),
            osc_offset_address: "/x/offset".to_string(),
            dry_run: true,
        };
        let text = toml::to_string_pretty(&Config::from(&original)).unwrap();
        let parsed: Config = toml::from_str(&text).unwrap();
        assert_eq!(parsed.into_settings(), original);
    }

    #[test]
    fn missing_keys_fall_back_to_defaults() {
        let parsed: Config = toml::from_str("interval = 30").unwrap();
        let s = parsed.into_settings();
        assert_eq!(s.interval, 30);
        assert_eq!(s.mode, Mode::Watch); // default mode
        assert_eq!(s.osc_port, 9100);
        assert_eq!(s.osc_offset_address, "/ziti/offset");
    }
}
