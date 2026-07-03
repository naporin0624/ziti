use crate::format::offset_seconds;
use crate::song::{Device, Song};
use chrono::Local;
use std::io::IsTerminal;

pub fn recognized_lines(now_hms: &str, song: &Song) -> String {
    let offset = song
        .offset
        .map(|secs| format!(" ({}s)", offset_seconds(secs)))
        .unwrap_or_default();
    format!(
        "[{}] \u{266a} recognized\n           {} - {}{}",
        now_hms, song.artist, song.title, offset
    )
}

pub fn sent_line(host: &str, port: u16, address: &str, dry_run: bool) -> String {
    if dry_run {
        format!("   \u{2192} (dry-run) would send OSC udp://{host}:{port} {address}")
    } else {
        format!("   \u{2192} \u{2713} OSC udp://{host}:{port} {address}")
    }
}

pub fn sent_float_line(host: &str, port: u16, address: &str, value: f64, dry_run: bool) -> String {
    let value = offset_seconds(value);
    if dry_run {
        format!("   \u{2192} (dry-run) would send OSC udp://{host}:{port} {address} {value}")
    } else {
        format!("   \u{2192} \u{2713} OSC udp://{host}:{port} {address} {value}")
    }
}

pub fn device_lines(devices: &[Device]) -> String {
    let Some(first) = devices.first() else {
        return "No audio devices found.".to_string();
    };
    let mut out = String::from("Audio devices:\n");
    for device in devices {
        if device.name.is_empty() {
            out.push_str(&format!("\n    {}", device.id));
        } else {
            out.push_str(&format!("\n  {}\n    {}", device.name, device.id));
        }
    }
    out.push_str(&format!("\n\nPass an id to -d, e.g. ziti -d {}", first.id));
    out
}

pub fn print_devices(devices: &[Device]) {
    for line in device_lines(devices).split('\n') {
        if line.starts_with("    ") || line.starts_with("Pass an id") {
            println!("{}", dim(line));
        } else {
            println!("{line}");
        }
    }
}

fn color_enabled() -> bool {
    std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

fn dim(s: &str) -> String {
    if color_enabled() {
        format!("\u{1b}[2m{s}\u{1b}[0m")
    } else {
        s.to_string()
    }
}

fn green(s: &str) -> String {
    if color_enabled() {
        format!("\u{1b}[32m{s}\u{1b}[0m")
    } else {
        s.to_string()
    }
}

fn red(s: &str) -> String {
    if color_enabled() {
        format!("\u{1b}[31m{s}\u{1b}[0m")
    } else {
        s.to_string()
    }
}

pub fn error_line(message: &str) -> String {
    format!("\u{2717} {message}")
}

pub fn print_error(message: &str) {
    eprintln!("{}", red(&error_line(message)));
}

pub fn print_recognized(song: &Song) {
    let hms = Local::now().format("%H:%M:%S").to_string();
    let block = recognized_lines(&hms, song);
    match block.split_once('\n') {
        Some((head, body)) => println!("{}\n{}", dim(head), body),
        None => println!("{block}"),
    }
}

pub fn print_sent(host: &str, port: u16, address: &str, dry_run: bool) {
    let line = sent_line(host, port, address, dry_run);
    if dry_run {
        println!("{}", dim(&line));
    } else {
        println!("{}", green(&line));
    }
}

pub fn print_sent_float(host: &str, port: u16, address: &str, value: f64, dry_run: bool) {
    let line = sent_float_line(host, port, address, value, dry_run);
    if dry_run {
        println!("{}", dim(&line));
    } else {
        println!("{}", green(&line));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn song() -> Song {
        Song {
            artist: "Mirin Sheeno".into(),
            title: "Harmony".into(),
            offset: None,
        }
    }

    #[test]
    fn recognized_is_two_lines_with_marker() {
        let out = recognized_lines("12:30:34", &song());
        assert_eq!(
            out,
            "[12:30:34] \u{266a} recognized\n           Mirin Sheeno - Harmony"
        );
    }

    #[test]
    fn sent_line_shows_check_and_target() {
        let out = sent_line("127.0.0.1", 9100, "/cannelloni/search", false);
        assert_eq!(
            out,
            "   \u{2192} \u{2713} OSC udp://127.0.0.1:9100 /cannelloni/search"
        );
    }

    #[test]
    fn sent_line_dry_run_is_marked() {
        let out = sent_line("127.0.0.1", 9100, "/cannelloni/search", true);
        assert!(out.contains("(dry-run)"));
        assert!(!out.contains("\u{2713}"));
    }

    #[test]
    fn recognized_appends_offset_when_present() {
        let song = Song {
            offset: Some(88.6),
            ..song()
        };
        assert_eq!(
            recognized_lines("12:30:34", &song),
            "[12:30:34] \u{266a} recognized\n           Mirin Sheeno - Harmony (88.6s)"
        );
    }

    #[test]
    fn sent_float_line_shows_value() {
        let out = sent_float_line("127.0.0.1", 9100, "/ziti/offset", 88.6, false);
        assert_eq!(
            out,
            "   \u{2192} \u{2713} OSC udp://127.0.0.1:9100 /ziti/offset 88.6"
        );
    }

    #[test]
    fn sent_float_line_dry_run_shows_value() {
        let out = sent_float_line("127.0.0.1", 9100, "/ziti/offset", 88.6, true);
        assert!(out.contains("(dry-run)"));
        assert!(out.ends_with("/ziti/offset 88.6"));
        assert!(!out.contains("\u{2713}"));
    }

    #[test]
    fn error_line_has_cross_marker() {
        assert_eq!(error_line("boom"), "\u{2717} boom");
    }

    fn device(id: &str, name: &str) -> Device {
        Device {
            id: id.into(),
            name: name.into(),
        }
    }

    #[test]
    fn device_lines_renders_two_tier_layout() {
        let devices = vec![
            device("coreaudio:A", "Mic A"),
            device("coreaudio:B", "Mic B"),
        ];
        assert_eq!(
            device_lines(&devices),
            "Audio devices:\n\n  Mic A\n    coreaudio:A\n  Mic B\n    coreaudio:B\n\nPass an id to -d, e.g. ziti -d coreaudio:A"
        );
    }

    #[test]
    fn device_lines_empty_slice_reports_none() {
        assert_eq!(device_lines(&[]), "No audio devices found.");
    }

    #[test]
    fn device_lines_skips_name_header_when_empty() {
        let devices = vec![device("coreaudio:Bare", "")];
        assert_eq!(
            device_lines(&devices),
            "Audio devices:\n\n    coreaudio:Bare\n\nPass an id to -d, e.g. ziti -d coreaudio:Bare"
        );
    }
}
