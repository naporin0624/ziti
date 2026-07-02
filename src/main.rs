use anyhow::Result;
use clap::Parser;
use std::io::IsTerminal;
use ziti::cli::Cli;
use ziti::settings::{Mode, Settings};
use ziti::song::{Deduplicator, Song};
use ziti::{config, format, interactive, osc, output, songrec};

fn handle_song(settings: &Settings, song: &Song) -> Result<()> {
    let text = format::render(&settings.format, song);
    output::print_recognized(song);
    if !settings.dry_run {
        osc::send(
            &settings.osc_host,
            settings.osc_port,
            &settings.osc_address,
            &text,
        )?;
    }
    output::print_sent(
        &settings.osc_host,
        settings.osc_port,
        &settings.osc_address,
        settings.dry_run,
    );
    if let Some(offset) = song.offset {
        if !settings.dry_run {
            osc::send_float(
                &settings.osc_host,
                settings.osc_port,
                &settings.osc_offset_address,
                offset as f32,
            )?;
        }
        output::print_sent_float(
            &settings.osc_host,
            settings.osc_port,
            &settings.osc_offset_address,
            offset,
            settings.dry_run,
        );
    }
    Ok(())
}

fn dispatch(settings: &Settings) -> Result<()> {
    let device = settings.device.as_deref();
    match settings.mode {
        Mode::Watch => {
            let mut dedup = Deduplicator::new();
            let mut on_song = |song: Song| -> Result<()> {
                // render once for the dedup key; handle_song renders again for sending —
                // intentional duplicate: keeps handle_song self-contained for the one-shot path
                let text = format::render(&settings.format, &song);
                if dedup.is_new(&text) {
                    if let Err(err) = handle_song(settings, &song) {
                        output::print_error(&format!("{err:#}"));
                    }
                }
                Ok(())
            };
            songrec::stream_listen(device, settings.interval, &mut on_song)
        }
        Mode::Once => match songrec::recognize_once(device, settings.interval)? {
            Some(song) => handle_song(settings, &song),
            None => anyhow::bail!("no song recognized"),
        },
    }
}

fn should_use_interactive(arg_count: usize, stdin_is_tty: bool) -> bool {
    arg_count <= 1 && stdin_is_tty
}

fn run() -> Result<()> {
    if should_use_interactive(std::env::args().count(), std::io::stdin().is_terminal()) {
        let initial = config::load();
        return match interactive::run(initial)? {
            Some(settings) => {
                if let Err(err) = config::save(&settings) {
                    output::print_error(&format!("Could not save config: {err:#}"));
                }
                dispatch(&settings)
            }
            None => Ok(()),
        };
    }

    let cli = Cli::parse();
    if cli.list {
        return songrec::list_devices();
    }
    dispatch(&Settings::from(&cli))
}

fn main() {
    if let Err(err) = run() {
        output::print_error(&format!("{err:#}"));
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interactive_only_with_no_args_and_tty() {
        assert!(should_use_interactive(1, true));
        assert!(!should_use_interactive(1, false));
        assert!(!should_use_interactive(2, true));
        assert!(!should_use_interactive(3, false));
    }
}
