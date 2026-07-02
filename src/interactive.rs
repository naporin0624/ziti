use crate::settings::{Mode, Settings};
use crate::songrec;
use anyhow::Result;
use dialoguer::{Confirm, Input, Select};

const RUN_LABEL: &str = "▶ Run";
const QUIT_LABEL: &str = "Quit";

fn mode_value(mode: Mode) -> &'static str {
    match mode {
        Mode::Once => "Recognize once",
        Mode::Watch => "Watch",
    }
}

fn device_value(device: &Option<String>) -> String {
    match device {
        Some(d) if !d.is_empty() => d.clone(),
        _ => "(none)".to_string(),
    }
}

pub fn menu_items(s: &Settings) -> Vec<String> {
    vec![
        RUN_LABEL.to_string(),
        format!("{:<14}{}", "Mode", mode_value(s.mode)),
        format!("{:<14}{}", "Device", device_value(&s.device)),
        format!("{:<14}{}", "Interval", format!("{}s", s.interval)),
        format!("{:<14}{}", "Format", s.format),
        format!("{:<14}{}", "OSC host", s.osc_host),
        format!("{:<14}{}", "OSC port", s.osc_port),
        format!("{:<14}{}", "OSC address", s.osc_address),
        format!("{:<14}{}", "OSC offset", s.osc_offset_address),
        format!(
            "{:<14}{}",
            "Offset send",
            if s.osc_offset_enabled { "on" } else { "off" }
        ),
        format!("{:<14}{}", "Dry run", if s.dry_run { "on" } else { "off" }),
        QUIT_LABEL.to_string(),
    ]
}

fn edit_mode(current: Mode) -> Result<Mode> {
    let options = [Mode::Once, Mode::Watch];
    let labels = [mode_value(Mode::Once), mode_value(Mode::Watch)];
    let default = if current == Mode::Watch { 1 } else { 0 };
    let idx = Select::new()
        .with_prompt("Mode")
        .items(&labels)
        .default(default)
        .interact()?;
    Ok(options[idx])
}

fn edit_device(current: Option<String>) -> Result<Option<String>> {
    let devices = match songrec::fetch_devices() {
        Ok(d) => d,
        Err(err) => {
            eprintln!("Could not list devices: {err:#}");
            Vec::new()
        }
    };
    let mut labels: Vec<String> = vec!["(none / songrec default)".to_string()];
    for d in &devices {
        if d.name.is_empty() {
            labels.push(d.id.clone());
        } else {
            labels.push(format!("{} ({})", d.name, d.id));
        }
    }
    labels.push("Enter manually…".to_string());

    let idx = Select::new()
        .with_prompt("Device")
        .items(&labels)
        .default(0)
        .interact()?;

    if idx == 0 {
        Ok(None)
    } else if idx == labels.len() - 1 {
        let entered: String = Input::new()
            .with_prompt("Device ID")
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
        .with_prompt("Interval (seconds)")
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

fn edit_offset_enabled(current: bool) -> Result<bool> {
    Ok(Confirm::new()
        .with_prompt("Send the in-track offset float")
        .default(current)
        .interact()?)
}

fn edit_dry_run(current: bool) -> Result<bool> {
    Ok(Confirm::new()
        .with_prompt("Dry run (print only, don't send)")
        .default(current)
        .interact()?)
}

pub fn run(mut settings: Settings) -> Result<Option<Settings>> {
    loop {
        let items = menu_items(&settings);
        let choice = Select::new()
            .with_prompt("ziti — select a field to edit, ▶ Run to start")
            .items(&items)
            .default(0)
            .interact()?;
        match choice {
            0 => return Ok(Some(settings)),
            1 => settings.mode = edit_mode(settings.mode)?,
            2 => settings.device = edit_device(settings.device)?,
            3 => settings.interval = edit_interval(settings.interval)?,
            4 => settings.format = edit_text("Format", &settings.format)?,
            5 => settings.osc_host = edit_text("OSC host", &settings.osc_host)?,
            6 => settings.osc_port = edit_port(settings.osc_port)?,
            7 => settings.osc_address = edit_text("OSC address", &settings.osc_address)?,
            8 => {
                settings.osc_offset_address =
                    edit_text("OSC offset address", &settings.osc_offset_address)?
            }
            9 => settings.osc_offset_enabled = edit_offset_enabled(settings.osc_offset_enabled)?,
            10 => settings.dry_run = edit_dry_run(settings.dry_run)?,
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
        assert_eq!(items.len(), 12);
        assert_eq!(items[0], "▶ Run");
        assert_eq!(items[1], "Mode          Watch");
        assert_eq!(items[2], "Device        (none)");
        assert_eq!(items[3], "Interval      10s");
        assert_eq!(items[8], "OSC offset    /ziti/offset");
        assert_eq!(items[9], "Offset send   on");
        assert_eq!(items[11], "Quit");
    }

    #[test]
    fn menu_reflects_settings_values() {
        let s = Settings {
            mode: Mode::Once,
            device: Some("coreaudio:X".to_string()),
            osc_offset_address: "/myapp/offset".to_string(),
            osc_offset_enabled: false,
            dry_run: true,
            ..Settings::default()
        };
        let items = menu_items(&s);
        assert_eq!(items[1], "Mode          Recognize once");
        assert_eq!(items[2], "Device        coreaudio:X");
        assert_eq!(items[8], "OSC offset    /myapp/offset");
        assert_eq!(items[9], "Offset send   off");
        assert_eq!(items[10], "Dry run       on");
    }
}
