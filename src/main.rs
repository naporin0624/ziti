use anyhow::Result;
use clap::Parser;
use songrec_osc::cli::Cli;
use songrec_osc::song::{Deduplicator, Song};
use songrec_osc::{format, osc, output, songrec};

fn handle_song(cli: &Cli, song: &Song) -> Result<()> {
    let text = format::render(&cli.format, song);
    output::print_recognized(song);
    if !cli.dry_run {
        osc::send(&cli.osc_host, cli.osc_port, &cli.osc_address, &text)?;
    }
    output::print_sent(&cli.osc_host, cli.osc_port, &cli.osc_address, cli.dry_run);
    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.list {
        return songrec::list_devices();
    }

    let device = cli.device.as_deref();

    if cli.watch {
        let mut dedup = Deduplicator::new();
        let mut on_song = |song: Song| -> Result<()> {
            // render once for the dedup key; handle_song renders again for sending —
            // intentional duplicate: keeps handle_song self-contained for the one-shot path
            let text = format::render(&cli.format, &song);
            if dedup.is_new(&text) {
                handle_song(&cli, &song)?;
            }
            Ok(())
        };
        return songrec::stream_listen(device, cli.interval, &mut on_song);
    }

    match songrec::recognize_once(device, cli.interval)? {
        Some(song) => handle_song(&cli, &song),
        None => {
            anyhow::bail!("no song recognized");
        }
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("\u{1b}[31m\u{2717}\u{1b}[0m {err:#}");
        std::process::exit(1);
    }
}
