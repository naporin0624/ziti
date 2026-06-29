use crate::song::Song;
use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

pub fn parse_song_json(line: &str) -> Option<Song> {
    let value: serde_json::Value = serde_json::from_str(line).ok()?;
    let title = value["track"]["title"].as_str()?;
    let artist = value["track"]["subtitle"].as_str()?;
    Some(Song {
        artist: artist.to_string(),
        title: title.to_string(),
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

pub fn list_devices() -> Result<()> {
    let status = Command::new("songrec")
        .args(["recognize", "-l"])
        .status()
        .context("failed to run `songrec` — is it installed and on PATH?")?;
    anyhow::ensure!(
        status.success(),
        "songrec exited with failure while listing devices"
    );
    Ok(())
}

pub fn recognize_once(device: Option<&str>, interval: u64) -> Result<Option<Song>> {
    let mut child = Command::new("songrec")
        .args(recognize_args("recognize", device, interval))
        .stdout(Stdio::piped())
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

    #[test]
    fn parses_artist_and_title() {
        let song = parse_song_json(MATCH).unwrap();
        assert_eq!(song.artist, "Mirin Sheeno");
        assert_eq!(song.title, "Harmony");
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
}
