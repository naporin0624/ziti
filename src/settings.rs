use crate::cli::Cli;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Once,
    Watch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub mode: Mode,
    pub device: Option<String>,
    pub interval: u64,
    pub format: String,
    pub osc_host: String,
    pub osc_port: u16,
    pub osc_address: String,
    pub dry_run: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            mode: Mode::Watch,
            device: None,
            interval: 10,
            format: "{artist} - {title}".to_string(),
            osc_host: "127.0.0.1".to_string(),
            osc_port: 9100,
            osc_address: "/cannelloni/search".to_string(),
            dry_run: false,
        }
    }
}

impl From<&Cli> for Settings {
    fn from(cli: &Cli) -> Self {
        Settings {
            mode: if cli.watch { Mode::Watch } else { Mode::Once },
            device: cli.device.clone(),
            interval: cli.interval,
            format: cli.format.clone(),
            osc_host: cli.osc_host.clone(),
            osc_port: cli.osc_port,
            osc_address: cli.osc_address.clone(),
            dry_run: cli.dry_run,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn default_mode_is_watch_and_matches_cli_defaults() {
        let s = Settings::default();
        assert_eq!(s.mode, Mode::Watch);
        assert_eq!(s.interval, 10);
        assert_eq!(s.format, "{artist} - {title}");
        assert_eq!(s.osc_host, "127.0.0.1");
        assert_eq!(s.osc_port, 9100);
        assert_eq!(s.osc_address, "/cannelloni/search");
        assert_eq!(s.device, None);
        assert!(!s.dry_run);
    }

    #[test]
    fn from_cli_without_watch_is_once() {
        let cli = Cli::parse_from(["ziti", "-d", "dev"]);
        let s = Settings::from(&cli);
        assert_eq!(s.mode, Mode::Once);
        assert_eq!(s.device.as_deref(), Some("dev"));
    }

    #[test]
    fn from_cli_with_watch_is_watch_and_copies_fields() {
        let cli = Cli::parse_from(["ziti", "--watch", "--osc-port", "9000", "--dry-run"]);
        let s = Settings::from(&cli);
        assert_eq!(s.mode, Mode::Watch);
        assert_eq!(s.osc_port, 9000);
        assert!(s.dry_run);
    }
}
