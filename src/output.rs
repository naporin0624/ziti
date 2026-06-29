use crate::song::Song;
use chrono::Local;
use std::io::IsTerminal;

pub fn recognized_lines(now_hms: &str, song: &Song) -> String {
    format!(
        "[{}] \u{266a} recognized\n           {} - {}",
        now_hms, song.artist, song.title
    )
}

pub fn sent_line(host: &str, port: u16, address: &str, dry_run: bool) -> String {
    if dry_run {
        format!("   \u{2192} (dry-run) would send OSC udp://{host}:{port} {address}")
    } else {
        format!("   \u{2192} \u{2713} OSC udp://{host}:{port} {address}")
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

#[cfg(test)]
mod tests {
    use super::*;

    fn song() -> Song {
        Song {
            artist: "Mirin Sheeno".into(),
            title: "Harmony".into(),
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
    fn error_line_has_cross_marker() {
        assert_eq!(error_line("boom"), "\u{2717} boom");
    }
}
