use crate::song::Song;

pub fn offset_seconds(offset: f64) -> String {
    format!("{offset:.1}")
}

pub fn render(template: &str, song: &Song) -> String {
    let offset = song.offset.map(offset_seconds).unwrap_or_default();
    template
        .replace("{artist}", &song.artist)
        .replace("{title}", &song.title)
        .replace("{offset}", &offset)
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
    fn default_template() {
        assert_eq!(
            render("{artist} - {title}", &song()),
            "Mirin Sheeno - Harmony"
        );
    }

    #[test]
    fn title_only() {
        assert_eq!(render("{title}", &song()), "Harmony");
    }

    #[test]
    fn unknown_tokens_left_verbatim() {
        assert_eq!(
            render("[{artist}] {unknown}", &song()),
            "[Mirin Sheeno] {unknown}"
        );
    }

    fn song_at(offset: f64) -> Song {
        Song {
            offset: Some(offset),
            ..song()
        }
    }

    #[test]
    fn offset_seconds_keeps_one_decimal() {
        assert_eq!(offset_seconds(88.6), "88.6");
        assert_eq!(offset_seconds(160.36), "160.4");
        assert_eq!(offset_seconds(0.0), "0.0");
    }

    #[test]
    fn offset_token_renders_one_decimal() {
        assert_eq!(render("{offset}", &song_at(88.6)), "88.6");
    }

    #[test]
    fn offset_token_empty_when_missing() {
        assert_eq!(render("{offset}", &song()), "");
    }

    #[test]
    fn offset_token_combines_with_other_tokens() {
        assert_eq!(
            render("{artist} - {title} @{offset}", &song_at(88.6)),
            "Mirin Sheeno - Harmony @88.6"
        );
    }
}
