# ziti

[日本語版 README](README.ja.md)

A small Rust CLI that recognizes the currently playing song with
[songrec](https://github.com/marin-m/SongRec) and forwards it as an OSC message
to any destination.

```
system audio (e.g. VB-Cable)
        │
        ▼
   songrec recognize / listen  (-j JSON)
        │   track.subtitle = artist, track.title = title
        ▼
   ziti  ──(OSC string)──▶  udp://host:port  /cannelloni/search
```

By default it sends `"artist - title"` as a single OSC string argument to
`udp://127.0.0.1:9100` at the address `/cannelloni/search`. The destination,
address, and string format are all configurable via flags.

## How it works

`ziti` launches the external `songrec` binary as a child process and parses its
`-j` output (compact single-line JSON) to extract the track. Linking songrec as
a library would drag in heavy native dependencies such as `soup3` (libsoup), so
the subprocess approach is used instead: dependencies stay light, and although
songrec itself is GPL, this CLI only invokes its binary, so GPL does not
propagate here.

songrec's own log output (the `INFO …` lines it writes to stderr) is suppressed,
so you only see `ziti`'s own status output.

## Requirements

- A Rust toolchain (edition 2021 / rustc 1.70+)
- The `songrec` binary on your `PATH`

```sh
# Example: install on macOS with the FFmpeg feature, no GUI
cargo install songrec --no-default-features --features ffmpeg
```

## Build

```sh
# Fetch dependencies and build (binary: target/debug/ziti)
cargo build

# Optimized release build (binary: target/release/ziti)
cargo build --release

# Run without building a standalone binary (args go after --)
cargo run -- --list
cargo run --release -- --watch -d "<device name>"
```

The built binary is self-contained and can be copied anywhere on your `PATH`:

```sh
cp target/release/ziti /usr/local/bin/
```

## Install

You can also install it straight onto your `PATH` with cargo:

```sh
cargo install --path .    # installs to ~/.cargo/bin/ziti
```

## Usage

```sh
# List available audio devices and exit
ziti --list

# One-shot: recognize a single song, send OSC, exit
# (exits non-zero if nothing is recognized)
ziti

# Watch: send on every newly recognized song; an immediate repeat is skipped.
# Stop with Ctrl-C.
ziti --watch -d "<device name / UID>"

# Print what would be sent without sending OSC
ziti --dry-run

# Change OSC destination, address, and string format
ziti --osc-host 192.168.1.10 --osc-port 9000 \
     --osc-address /myapp/nowplaying \
     --format "{title} / {artist}"
```

Pick a device name from the `--list` output (macOS + VB-Cable example:
`coreaudio:com.vbaudio.vbcable:XXXXXXXX-...`). UIDs change per machine and on
reinstall, so checking `--list` each time is more robust than hardcoding one.

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `-l, --list` | — | List audio devices and exit |
| `-d, --device <NAME>` | system default | Audio device to use (passed to songrec `-d`) |
| `--watch` | off | Keep listening; send on every new song (consecutive duplicates skipped) |
| `-i, --interval <SEC>` | 10 | Seconds between Shazam requests (songrec `-i`) |
| `--format <TMPL>` | `{artist} - {title}` | Template for the sent string (`{artist}`, `{title}`, `{offset}` are substituted) |
| `--osc-host <HOST>` | 127.0.0.1 | OSC destination host |
| `--osc-port <PORT>` | 9100 | OSC destination port |
| `--osc-address <ADDR>` | /cannelloni/search | OSC address pattern |
| `--osc-offset-address <ADDR>` | /ziti/offset | OSC address pattern for the in-track offset float |
| `--no-osc-offset` | off | Do not send the in-track offset float |
| `--dry-run` | off | Print the message instead of sending it |

## Output

Recognition and sending are shown as a two-line log. Success and failure are
conveyed with symbols (♪ ✓ ✗ →) as well as color, so no information is lost
under `NO_COLOR` or when piped.

```
[12:30:34] ♪ recognized
           Mirin Sheeno - Harmony (88.6s)
   → ✓ OSC udp://127.0.0.1:9100 /cannelloni/search
   → ✓ OSC udp://127.0.0.1:9100 /ziti/offset 88.6
```

The `(88.6s)` suffix and the `/ziti/offset` line appear only when Shazam
reports the in-track position. With `--dry-run` each send line becomes
`→ (dry-run) would send …` (the offset line includes the value that would
have been sent).

## Extra OSC signals

Alongside the song string, `ziti` sends a float message on a separate address
(default `/ziti/offset`, configurable via `--osc-offset-address`):

| Address | Type | Meaning |
|---------|------|---------|
| `/ziti/offset` (default) | float | Position in seconds within the track where the recognized snippet sits; sent right after each song string when Shazam reports it. Negative offsets (window starting before the track head) are clamped to `0.0` |

`--dry-run` prints the values instead of sending them. The float send can be
disabled entirely with `--no-osc-offset`; the `(88.6s)` display and `{offset}`
in `--format` are unaffected.

## Behavior notes

- The message at `--osc-address` carries exactly **one `string` argument** (the
  formatted text); the offset travels separately as the float message at
  `--osc-offset-address` described above.
- In `--watch` mode a send failure is logged as a warning and watching
  continues; one-shot mode exits non-zero on failure.
- Duplicate detection compares the **formatted string**, so changing `--format`
  also changes what counts as the same song.
- `{offset}` in `--format` becomes the in-track position in seconds with one
  decimal (e.g. `88.6`), or an empty string when Shazam reports none. Because
  duplicate detection keys on the rendered string, a format containing
  `{offset}` re-sends the same song on every recognition cycle with the updated
  offset — useful as a moving current-time signal, but it effectively disables
  duplicate suppression.
- songrec's own stderr logging is silenced; only `ziti`'s output is printed.

## Development

```sh
cargo test                                   # all tests
cargo fmt --all -- --check                   # formatting check
cargo clippy --all-targets -- -D warnings    # lint
```

Git hooks are enforced with [rusty-hook](https://github.com/swellaby/rusty-hook)
(`.rusty-hook.toml`): pre-commit runs `fmt --check` + `clippy -D warnings`, and
pre-push runs `cargo test`.

Design and implementation notes live under `docs/superpowers/`.

## License

Use the code in this repository per your own policy. Note that `songrec` itself
is GPL-3.0+, but this CLI only invokes songrec as an external binary and does not
incorporate its source.
