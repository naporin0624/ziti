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

## Out of scope

- CI cross-build / release binaries (explicitly declined).
- Verifying on a real Windows machine (explicitly declined).
- Any code changes.

## Testing

Docs-only change; run `cargo fmt --check`, `cargo clippy`, `cargo test` to
confirm the tree stays green per project rules.
