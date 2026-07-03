# Windows usage documentation — design

Date: 2026-07-03
Scope: documentation only (no code, no CI changes). Approved via AskUserQuestion;
build-guidance approach defaulted to "MSYS2-only" (recommended option) after the
user stepped away.

## Goal

Document how to use ziti on Windows in both `README.md` and `README.ja.md`, as a
single self-contained section so a Windows reader finds everything in one place.

## Facts verified against upstream (SongRec 0.7.4)

- ziti itself is pure Rust (clap/rosc/serde/toml/chrono/anyhow/dialoguer) and
  builds on Windows with any Rust toolchain.
- songrec 0.7.4 has **non-optional** native deps: `glib`, `soup3` (libsoup3),
  `gettext-sys` (gettext-system). A CLI-only `cargo install songrec
  --no-default-features --features ffmpeg` therefore still requires those
  libraries + pkg-config. Upstream's official Windows path is MSYS2 (UCRT64).
- SongRec publishes no prebuilt Windows binaries (0.7.4 assets: flathub tarball
  only). Source build is the only option.
- Windows loopback audio: VB-CABLE is the native Windows counterpart to the
  VB-Cable setup already documented for macOS. Device names come from
  cpal/WASAPI, so they look different from the macOS `coreaudio:` form; readers
  must pick from `ziti --list` output.

## Changes

1. `README.md`
   - In `## Requirements`, add a one-line pointer: Windows users see
     "Using on Windows".
   - Add new section `## Using on Windows` directly after `## Install`:
     1. Note: ziti builds with any toolchain; songrec needs MSYS2 UCRT64
        because of glib/libsoup3/gettext.
     2. Install MSYS2, open the UCRT64 shell, `pacman -S` the CLI-only dep
        subset (rust, gcc, pkgconf, glib2, libsoup3, gettext-runtime, openssl,
        ffmpeg — UCRT64 packages, per upstream's list minus GUI packages).
     3. `cargo install songrec --no-default-features --features ffmpeg`
     4. Build/install ziti in the same shell (`cargo install --path .`).
     5. Loopback: install VB-CABLE, set Windows default playback device to
        "CABLE Input", run `ziti --list`, pick the "CABLE Output" device.
        Mention "Stereo Mix" as an alternative when available.
     6. Everything else (flags, config, interactive mode) is identical.
     7. Short caveat: these steps follow upstream's official MSYS2 instructions
        but have not been verified on a Windows machine by the maintainer.
2. `README.ja.md`
   - Mirror as `## Windows で使う` in the same position, plus the same
     Requirements pointer, in Japanese.

## Addendum (2026-07-03): Docker route

User follow-up: prefer Docker over the MSYS2 route, with OSC sent out of the
container. Approved: stack onto the same branch/PR (#3), restructure the
Windows section so Docker is the recommended route and MSYS2 the native
alternative.

### Why it works

- OSC is plain UDP; ziti already has `--osc-host`/`--osc-port`. From a
  container, `host.docker.internal` reaches the Windows host; LAN targets work
  directly. No code changes.
- Audio into the container rides WSLg's PulseAudio bridge: Windows default
  playback → VB-CABLE "CABLE Input"; "CABLE Output" set as Windows default
  *recording* device; WSLg's RDPSource exposes it at `/mnt/wslg/PulseServer`;
  the container mounts `/mnt/wslg` and sets
  `PULSE_SERVER=unix:/mnt/wslg/PulseServer` (per Microsoft's official WSLg
  container sample).
- songrec builds natively on Linux (its mainline platform), eliminating MSYS2.

### Deliverables

1. `Dockerfile` — multi-stage: `rust:bookworm` builder installs apt build deps
   (pkg-config, libssl-dev, libglib2.0-dev, libsoup-3.0-dev, gettext,
   libasound2-dev, libpipewire-0.3-dev — cpal's Linux pipewire feature is
   unconditional — plus libpulse-dev if the pulse feature is used) and runs
   `cargo install songrec --no-default-features` (+`pulse` feature if it
   builds cleanly) and `cargo build --release` for ziti. Runtime stage:
   `debian:bookworm-slim` with the runtime libs (libsoup-3.0-0, libasound2 +
   libasound2-plugins for the ALSA→Pulse bridge, libpulse0, pipewire libs,
   ca-certificates) and an `/etc/asound.conf` defaulting ALSA to pulse.
   Entrypoint `ziti`.
2. `compose.yaml` — mounts `/mnt/wslg`, sets `PULSE_SERVER`, adds
   `extra_hosts: host.docker.internal:host-gateway` (Docker-Engine-in-WSL
   case), example command `--watch --osc-host host.docker.internal`.
3. `.dockerignore` — target/, .git/, docs/.
4. README.md / README.ja.md — Windows section restructured: Docker route
   (recommended) first with the audio-chain diagram and compose usage, MSYS2
   route kept as the native alternative. Honest caveats: VB-CABLE still
   required on the host; audio path verified only against upstream docs, not
   on a Windows machine; image build smoke-tested on linux/arm64.

### Verification (local, macOS)

`docker build` completes; `docker run --rm <img> --help` shows ziti help;
`songrec --help` runs in the image; cargo fmt/clippy/test stay green.

## Out of scope

- CI cross-build / release binaries (explicitly declined).
- Publishing the image to a registry.
- Verifying the audio path on a real Windows machine (explicitly declined).
- Any changes to ziti's Rust code.

## Testing

Docs-only change; run `cargo fmt --check`, `cargo clippy`, `cargo test` to
confirm the tree stays green per project rules.
