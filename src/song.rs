#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Song {
    pub artist: String,
    pub title: String,
}

/// Suppresses consecutive identical OSC sends. Keyed on the rendered string so
/// that two distinct tracks collapsing to the same text under the active
/// `--format` are still treated as one.
pub struct Deduplicator {
    last: Option<String>,
}

impl Deduplicator {
    pub fn new() -> Self {
        Deduplicator { last: None }
    }

    pub fn is_new(&mut self, key: &str) -> bool {
        if self.last.as_deref() == Some(key) {
            return false;
        }
        self.last = Some(key.to_string());
        true
    }
}

impl Default for Deduplicator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_key_is_new() {
        let mut d = Deduplicator::new();
        assert!(d.is_new("A - B"));
    }

    #[test]
    fn immediate_repeat_is_not_new() {
        let mut d = Deduplicator::new();
        d.is_new("A - B");
        assert!(!d.is_new("A - B"));
    }

    #[test]
    fn changed_key_is_new_again() {
        let mut d = Deduplicator::new();
        d.is_new("A - B");
        assert!(d.is_new("C - D"));
    }

    #[test]
    fn same_key_after_change_is_new() {
        let mut d = Deduplicator::new();
        d.is_new("A - B");
        d.is_new("C - D");
        assert!(d.is_new("A - B"));
    }
}
