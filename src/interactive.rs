use crate::settings::{Mode, Settings};
use crate::songrec;
use anyhow::Result;
use dialoguer::{Confirm, Input, Select};

const RUN_LABEL: &str = "▶ 実行";
const QUIT_LABEL: &str = "終了";

fn mode_value(mode: Mode) -> &'static str {
    match mode {
        Mode::Once => "1回認識",
        Mode::Watch => "監視 (watch)",
    }
}

fn device_value(device: &Option<String>) -> String {
    match device {
        Some(d) if !d.is_empty() => d.clone(),
        _ => "(指定なし)".to_string(),
    }
}

pub fn menu_items(s: &Settings) -> Vec<String> {
    vec![
        RUN_LABEL.to_string(),
        format!("モード        {}", mode_value(s.mode)),
        format!("デバイス      {}", device_value(&s.device)),
        format!("間隔          {}s", s.interval),
        format!("フォーマット   {}", s.format),
        format!("OSC host      {}", s.osc_host),
        format!("OSC port      {}", s.osc_port),
        format!("OSC address   {}", s.osc_address),
        format!("dry-run       {}", if s.dry_run { "on" } else { "off" }),
        QUIT_LABEL.to_string(),
    ]
}

fn edit_mode(current: Mode) -> Result<Mode> {
    let options = [Mode::Once, Mode::Watch];
    let labels = [mode_value(Mode::Once), mode_value(Mode::Watch)];
    let default = if current == Mode::Watch { 1 } else { 0 };
    let idx = Select::new()
        .with_prompt("モード")
        .items(&labels)
        .default(default)
        .interact()?;
    Ok(options[idx])
}

fn edit_device(current: Option<String>) -> Result<Option<String>> {
    let devices = match songrec::fetch_devices() {
        Ok(d) => d,
        Err(err) => {
            eprintln!("デバイス一覧を取得できませんでした: {err:#}");
            Vec::new()
        }
    };
    let mut labels: Vec<String> = vec!["(指定なし / songrec デフォルト)".to_string()];
    for d in &devices {
        if d.name.is_empty() {
            labels.push(d.id.clone());
        } else {
            labels.push(format!("{} ({})", d.name, d.id));
        }
    }
    labels.push("手入力…".to_string());

    let idx = Select::new()
        .with_prompt("デバイス")
        .items(&labels)
        .default(0)
        .interact()?;

    if idx == 0 {
        Ok(None)
    } else if idx == labels.len() - 1 {
        let entered: String = Input::new()
            .with_prompt("デバイス ID")
            .allow_empty(true)
            .with_initial_text(current.unwrap_or_default())
            .interact_text()?;
        Ok(if entered.is_empty() {
            None
        } else {
            Some(entered)
        })
    } else {
        Ok(Some(devices[idx - 1].id.clone()))
    }
}

fn edit_interval(current: u64) -> Result<u64> {
    Ok(Input::<u64>::new()
        .with_prompt("認識間隔（秒）")
        .with_initial_text(current.to_string())
        .interact_text()?)
}

fn edit_port(current: u16) -> Result<u16> {
    Ok(Input::<u16>::new()
        .with_prompt("OSC port")
        .with_initial_text(current.to_string())
        .interact_text()?)
}

fn edit_text(prompt: &str, current: &str) -> Result<String> {
    Ok(Input::<String>::new()
        .with_prompt(prompt)
        .with_initial_text(current.to_string())
        .interact_text()?)
}

fn edit_dry_run(current: bool) -> Result<bool> {
    Ok(Confirm::new()
        .with_prompt("dry-run（送信せず表示のみ）")
        .default(current)
        .interact()?)
}

pub fn run(mut settings: Settings) -> Result<Option<Settings>> {
    loop {
        let items = menu_items(&settings);
        let choice = Select::new()
            .with_prompt("ziti — 項目を選んで編集、▶ 実行で開始")
            .items(&items)
            .default(0)
            .interact()?;
        match choice {
            0 => return Ok(Some(settings)),
            1 => settings.mode = edit_mode(settings.mode)?,
            2 => settings.device = edit_device(settings.device)?,
            3 => settings.interval = edit_interval(settings.interval)?,
            4 => settings.format = edit_text("送信フォーマット", &settings.format)?,
            5 => settings.osc_host = edit_text("OSC host", &settings.osc_host)?,
            6 => settings.osc_port = edit_port(settings.osc_port)?,
            7 => settings.osc_address = edit_text("OSC address", &settings.osc_address)?,
            8 => settings.dry_run = edit_dry_run(settings.dry_run)?,
            _ => return Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{Mode, Settings};

    #[test]
    fn menu_items_has_run_fields_and_quit_in_order() {
        let items = menu_items(&Settings::default());
        assert_eq!(items.len(), 10);
        assert_eq!(items[0], "▶ 実行");
        assert_eq!(items[1], "モード        監視 (watch)");
        assert_eq!(items[2], "デバイス      (指定なし)");
        assert_eq!(items[3], "間隔          10s");
        assert_eq!(items[9], "終了");
    }

    #[test]
    fn menu_reflects_settings_values() {
        let s = Settings {
            mode: Mode::Once,
            device: Some("coreaudio:X".to_string()),
            dry_run: true,
            ..Settings::default()
        };
        let items = menu_items(&s);
        assert_eq!(items[1], "モード        1回認識");
        assert_eq!(items[2], "デバイス      coreaudio:X");
        assert_eq!(items[8], "dry-run       on");
    }
}
