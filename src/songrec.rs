use crate::song::Song;
use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

pub fn parse_song_json(line: &str) -> Option<Song> {
    let value: serde_json::Value = serde_json::from_str(line).ok()?;
    let title = value["track"]["title"].as_str()?;
    let artist = value["track"]["subtitle"].as_str()?;
    // Shazam reports a negative offset when the analyzed window starts before
    // the track head; clamp to 0 since it is meaningless as a playback position.
    let offset = value["matches"][0]["offset"]
        .as_f64()
        .map(|secs| secs.max(0.0));
    Some(Song {
        artist: artist.to_string(),
        title: title.to_string(),
        offset,
    })
}

fn recognize_args<'a>(sub: &'a str, device: Option<&'a str>, interval: u64) -> Vec<String> {
    let mut args = vec![
        sub.to_string(),
        "-j".to_string(),
        "-i".to_string(),
        interval.to_string(),
    ];
    if let Some(dev) = device {
        args.push("-d".to_string());
        args.push(dev.to_string());
    }
    args
}

pub fn parse_device_line(line: &str) -> Option<crate::song::Device> {
    const MARKER: &str = "Available device: ";
    let start = line.find(MARKER)? + MARKER.len();
    let remainder = line[start..].trim_end();
    let (id, name) = match remainder.find(" (") {
        Some(pos) => {
            let raw_name = remainder[pos + 2..]
                .strip_suffix(')')
                .unwrap_or(&remainder[pos + 2..]);
            (&remainder[..pos], raw_name)
        }
        None => (remainder, ""),
    };
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    let name = name
        .replace(['\u{200e}', '\u{200f}'], "")
        .trim()
        .to_string();
    Some(crate::song::Device {
        id: id.to_string(),
        name,
    })
}

pub fn parse_device_list(stderr: &str) -> Vec<crate::song::Device> {
    stderr.lines().filter_map(parse_device_line).collect()
}

pub fn fetch_devices() -> Result<Vec<crate::song::Device>> {
    let output = Command::new("songrec")
        .args(["recognize", "-l"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .context("failed to run `songrec` — is it installed and on PATH?")?;
    anyhow::ensure!(
        output.status.success(),
        "songrec exited with failure while listing devices"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(parse_device_list(&stderr))
}

pub fn list_devices() -> Result<()> {
    let devices = fetch_devices()?;
    crate::output::print_devices(&devices);
    Ok(())
}

pub fn recognize_once(device: Option<&str>, interval: u64) -> Result<Option<Song>> {
    let mut child = Command::new("songrec")
        .args(recognize_args("recognize", device, interval))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to run `songrec` — is it installed and on PATH?")?;
    let stdout = child.stdout.take().expect("stdout was piped");
    let mut song = None;
    for line in BufReader::new(stdout).lines() {
        let line = line?;
        if let Some(parsed) = parse_song_json(&line) {
            song = Some(parsed);
            break;
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    Ok(song)
}

pub fn stream_listen(
    device: Option<&str>,
    interval: u64,
    on_song: &mut dyn FnMut(Song) -> Result<()>,
) -> Result<()> {
    let mut child = Command::new("songrec")
        .args(recognize_args("listen", device, interval))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to run `songrec` — is it installed and on PATH?")?;
    let stdout = child.stdout.take().expect("stdout was piped");
    for line in BufReader::new(stdout).lines() {
        let Ok(line) = line else { continue };
        if let Some(song) = parse_song_json(&line) {
            on_song(song)?;
        }
    }
    let _ = child.wait();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const MATCH: &str = r#"{"matches":[{"id":"1"}],"timestamp":1700000000,"track":{"key":"123","title":"Harmony","subtitle":"Mirin Sheeno","images":{"coverart":"https://x"}}}"#;

    /// Trimmed from a real `songrec -j` Shazam response.
    const SHAZAM_RESPONSE: &str = r#"{
        "location": {"accuracy": 0.01},
        "matches": [
            {"frequencyskew": -0.00044292212, "id": "877191404", "offset": 88.55546875, "timeskew": -0.0005968213},
            {"frequencyskew": -0.0003221631, "id": "508449286", "offset": 89.958476562, "timeskew": -0.0005222559}
        ],
        "tagid": "f8ee3421-4906-46d0-809d-ae01975773b2",
        "timestamp": 601233287,
        "timezone": "Europe/Paris",
        "track": {
            "isrc": "QM42K2079668",
            "key": "508449286",
            "subtitle": "EmoCosine",
            "title": "This Club Is Not 4 U",
            "type": "MUSIC",
            "url": "https://www.shazam.com/track/508449286/this-club-is-not-4-u"
        }
    }"#;

    #[test]
    fn parses_artist_and_title() {
        let song = parse_song_json(MATCH).unwrap();
        assert_eq!(song.artist, "Mirin Sheeno");
        assert_eq!(song.title, "Harmony");
    }

    #[test]
    fn match_without_offset_yields_none_offset() {
        let song = parse_song_json(MATCH).unwrap();
        assert_eq!(song.offset, None);
    }

    #[test]
    fn parses_offset_from_real_shazam_response() {
        let song = parse_song_json(SHAZAM_RESPONSE).unwrap();
        assert_eq!(song.artist, "EmoCosine");
        assert_eq!(song.title, "This Club Is Not 4 U");
        assert_eq!(song.offset, Some(88.55546875));
    }

    #[test]
    fn negative_offset_is_clamped_to_zero() {
        let json = r#"{"matches":[{"id":"1","offset":-5.0}],"track":{"title":"Harmony","subtitle":"Mirin Sheeno"}}"#;
        let song = parse_song_json(json).unwrap();
        assert_eq!(song.offset, Some(0.0));
    }

    #[test]
    fn no_track_is_none() {
        assert!(parse_song_json(r#"{"matches":[],"timestamp":1700000000}"#).is_none());
    }

    #[test]
    fn invalid_json_is_none() {
        assert!(parse_song_json("not json").is_none());
    }

    #[test]
    fn args_include_device_when_present() {
        let args = recognize_args("listen", Some("coreaudio:UID"), 12);
        assert_eq!(
            args,
            vec!["listen", "-j", "-i", "12", "-d", "coreaudio:UID"]
        );
    }

    #[test]
    fn args_omit_device_when_absent() {
        let args = recognize_args("recognize", None, 10);
        assert_eq!(args, vec!["recognize", "-j", "-i", "10"]);
    }

    #[test]
    fn missing_title_or_subtitle_is_none() {
        assert!(parse_song_json(r#"{"track":{"title":"X"}}"#).is_none());
        assert!(parse_song_json(r#"{"track":{"subtitle":"Y"}}"#).is_none());
    }

    #[test]
    fn non_string_fields_are_none() {
        assert!(parse_song_json(r#"{"track":{"title":1,"subtitle":2}}"#).is_none());
    }

    #[test]
    fn parses_full_info_line_with_marks_and_inner_parens() {
        let line = "[2026-06-30T05:58:02Z INFO songrec::cli_main /x/cli_main.rs:116] Available device: coreaudio:A3E21E5F (\u{200e}napochaaanのマイク (input))";
        let device = parse_device_line(line).unwrap();
        assert_eq!(device.id, "coreaudio:A3E21E5F");
        assert_eq!(device.name, "napochaaanのマイク (input)");
    }

    #[test]
    fn parses_aggregate_line() {
        let line = "[2026-06-30T05:58:02Z INFO songrec::cli_main /x/cli_main.rs:116] Available device: coreaudio:~:AMS2_Aggregate:0 (機器セット)";
        let device = parse_device_line(line).unwrap();
        assert_eq!(device.id, "coreaudio:~:AMS2_Aggregate:0");
        assert_eq!(device.name, "機器セット");
    }

    #[test]
    fn line_without_marker_is_none() {
        assert!(parse_device_line("[INFO] some unrelated log line").is_none());
    }

    #[test]
    fn device_without_parenthesized_name_has_empty_name() {
        let line = "Available device: coreaudio:BareDevice";
        let device = parse_device_line(line).unwrap();
        assert_eq!(device.id, "coreaudio:BareDevice");
        assert_eq!(device.name, "");
    }

    #[test]
    fn parse_device_list_collects_multiple_and_skips_noise() {
        let stderr = "[INFO] starting\n\
[2026 INFO songrec] Available device: coreaudio:A (\u{200e}マイク)\n\
unrelated line\n\
[2026 INFO songrec] Available device: coreaudio:B (機器セット)\n";
        let devices = parse_device_list(stderr);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].id, "coreaudio:A");
        assert_eq!(devices[0].name, "マイク");
        assert_eq!(devices[1].id, "coreaudio:B");
        assert_eq!(devices[1].name, "機器セット");
    }
}
