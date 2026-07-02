use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Recognize the playing song via songrec and send it as an OSC string")]
pub struct Cli {
    /// List available audio devices and exit
    #[arg(short = 'l', long)]
    pub list: bool,

    /// Audio device to capture from (passed to songrec -d)
    #[arg(short = 'd', long)]
    pub device: Option<String>,

    /// Keep listening and send on every newly recognized song
    #[arg(long)]
    pub watch: bool,

    /// Seconds between Shazam requests (passed to songrec -i)
    #[arg(short = 'i', long, default_value_t = 10)]
    pub interval: u64,

    /// Template for the sent string; supports {artist}, {title}, and {offset}
    #[arg(long, default_value = "{artist} - {title}")]
    pub format: String,

    /// OSC destination host
    #[arg(long = "osc-host", default_value = "127.0.0.1")]
    pub osc_host: String,

    /// OSC destination port
    #[arg(long = "osc-port", default_value_t = 9100)]
    pub osc_port: u16,

    /// OSC address pattern
    #[arg(long = "osc-address", default_value = "/cannelloni/search")]
    pub osc_address: String,

    /// OSC address pattern for the in-track offset float
    #[arg(long = "osc-offset-address", default_value = "/ziti/offset")]
    pub osc_offset_address: String,

    /// Print what would be sent without sending OSC
    #[arg(long)]
    pub dry_run: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let cli = Cli::parse_from(["ziti"]);
        assert_eq!(cli.interval, 10);
        assert_eq!(cli.format, "{artist} - {title}");
        assert_eq!(cli.osc_host, "127.0.0.1");
        assert_eq!(cli.osc_port, 9100);
        assert_eq!(cli.osc_address, "/cannelloni/search");
        assert_eq!(cli.osc_offset_address, "/ziti/offset");
        assert!(!cli.list && !cli.watch && !cli.dry_run);
        assert!(cli.device.is_none());
    }

    #[test]
    fn parses_overrides() {
        let cli = Cli::parse_from([
            "ziti",
            "--watch",
            "-d",
            "dev",
            "--osc-port",
            "9000",
            "--osc-offset-address",
            "/myapp/offset",
            "--dry-run",
        ]);
        assert!(cli.watch);
        assert_eq!(cli.device.as_deref(), Some("dev"));
        assert_eq!(cli.osc_port, 9000);
        assert_eq!(cli.osc_offset_address, "/myapp/offset");
        assert!(cli.dry_run);
    }

    #[test]
    fn verify_clap_config() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
