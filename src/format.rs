use crate::song::Song;

pub fn render(template: &str, song: &Song) -> String {
    template
        .replace("{artist}", &song.artist)
        .replace("{title}", &song.title)
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
}
