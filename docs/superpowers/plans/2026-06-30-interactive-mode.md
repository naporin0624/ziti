# ziti 対話モード（設定一覧型）Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `./ziti` を引数なしで実行したとき、全オプションを「設定一覧型」UIで対話的に設定して実行できるようにする。選択した設定は次回の初期値として保存する。

**Architecture:** 解決済み設定を表す `Settings` を中心に据える。対話経路（`config::load` → `interactive::run` → `config::save`）と非対話経路（`Cli` → `Settings::from`）の両方が同じ `dispatch(&Settings)` に合流する。純粋なロジック（変換・ラベル生成・パス解決）は dialoguer/ファイル I/O から分離してユニットテストする。

**Tech Stack:** Rust 2021 / clap(derive) / dialoguer / serde(derive) + toml / anyhow

## Global Constraints

- Rust edition 2021、既存の clap 4 / anyhow 1 を維持する。
- 既存の非対話フラグ挙動を壊さない：`--watch` 未指定なら 1 回認識、`--list` は従来どおり、非 TTY パイプ実行も 1 回認識のまま。
- 対話モードの初期 `mode` デフォルトは **Watch（監視）**。
- 既存コードの「純粋関数＋I/O 関数」分離スタイル（`device_lines`/`print_devices` 参照）に従う。
- ユーザーへ向けた対話文言は日本語。
- 各タスク末尾で `cargo fmt --all`・`cargo clippy --all-targets -- -D warnings`・`cargo test` が通ること（husky pre-commit と同じ）。
- 設定ファイルパス：`$XDG_CONFIG_HOME/ziti/config.toml`、無ければ `$HOME/.config/ziti/config.toml`。`directories` 等の追加クレートは使わない。

---

### Task 1: `Settings` と `Mode`（解決済み設定の中核型）

**Files:**
- Create: `src/settings.rs`
- Modify: `src/lib.rs`（モジュール登録）
- Test: `src/settings.rs`（`#[cfg(test)]`）

**Interfaces:**
- Consumes: `crate::cli::Cli`（既存：`watch: bool`, `device: Option<String>`, `interval: u64`, `format: String`, `osc_host: String`, `osc_port: u16`, `osc_address: String`, `dry_run: bool`）
- Produces:
  - `pub enum Mode { Once, Watch }`（`Debug, Clone, Copy, PartialEq, Eq`）
  - `pub struct Settings { pub mode: Mode, pub device: Option<String>, pub interval: u64, pub format: String, pub osc_host: String, pub osc_port: u16, pub osc_address: String, pub dry_run: bool }`（`Debug, Clone, PartialEq, Eq`）
  - `impl Default for Settings`（mode=Watch, interval=10, format="{artist} - {title}", osc_host="127.0.0.1", osc_port=9100, osc_address="/cannelloni/search", device=None, dry_run=false）
  - `impl From<&Cli> for Settings`

- [ ] **Step 1: モジュールを登録**

`src/lib.rs` のモジュール一覧（先頭）に追記：

```rust
pub mod cli;
pub mod config;
pub mod format;
pub mod interactive;
pub mod osc;
pub mod settings;
pub mod song;
pub mod songrec;
```

（`config` と `interactive` は後続タスクで作成する。このタスクの `cargo test` は `settings` 追加分だけを対象に実行し、まだ存在しないモジュールを登録するとビルドが通らないため、**今は `settings` の 1 行だけ追加**する。後続タスクで各モジュールを追記すること。）

実際にこのタスクで追加する 1 行のみ：

```rust
pub mod settings;
```

- [ ] **Step 2: 失敗するテストを書く**

`src/settings.rs` を新規作成し、まずテストだけ書く：

```rust
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
```

- [ ] **Step 3: テストが失敗（コンパイルエラー）することを確認**

Run: `cargo test --lib settings`
Expected: FAIL — `Settings` / `Mode` 未定義でコンパイルエラー。

- [ ] **Step 4: 最小実装を書く**

`src/settings.rs` のテストより上に追記：

```rust
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
```

- [ ] **Step 5: テストが通ることを確認**

Run: `cargo test --lib settings`
Expected: PASS（3 テスト）。

- [ ] **Step 6: 整形・lint・コミット**

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
git add src/settings.rs src/lib.rs
git commit -m "feat: add Settings/Mode resolved-config type"
```

---

### Task 2: `Config` 永続化（TOML 読み書き・パス解決）

**Files:**
- Create: `src/config.rs`
- Modify: `src/lib.rs`（`pub mod config;` 追加）, `Cargo.toml`（依存追加）
- Test: `src/config.rs`（`#[cfg(test)]`）

**Interfaces:**
- Consumes: `crate::settings::{Mode, Settings}`
- Produces:
  - `pub struct Config { pub mode: String, pub device: Option<String>, pub interval: u64, pub format: String, pub osc_host: String, pub osc_port: u16, pub osc_address: String, pub dry_run: bool }`（serde `Serialize`/`Deserialize`, 容器に `#[serde(default)]`）
  - `impl From<&Settings> for Config` / `Config::into_settings(self) -> Settings`
  - `pub fn config_path_from(xdg: Option<&OsStr>, home: Option<&OsStr>) -> Option<PathBuf>`
  - `pub fn load() -> Settings`（読めない/壊れていればデフォルト）
  - `pub fn save(settings: &Settings) -> anyhow::Result<()>`

- [ ] **Step 1: 依存を追加**

`Cargo.toml` の `[dependencies]` に追記：

```toml
serde = { version = "1", features = ["derive"] }
toml = "0.8"
```

- [ ] **Step 2: モジュール登録**

`src/lib.rs` に `pub mod config;` を追加（アルファベット順で `cli` の次あたり）。

- [ ] **Step 3: 失敗するテストを書く**

`src/config.rs` を新規作成し、テストを書く：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{Mode, Settings};
    use std::ffi::OsStr;

    #[test]
    fn path_prefers_xdg_when_set() {
        let p = config_path_from(Some(OsStr::new("/x/cfg")), Some(OsStr::new("/home/u"))).unwrap();
        assert_eq!(p, std::path::PathBuf::from("/x/cfg/ziti/config.toml"));
    }

    #[test]
    fn path_falls_back_to_home_dot_config() {
        let p = config_path_from(Some(OsStr::new("")), Some(OsStr::new("/home/u"))).unwrap();
        assert_eq!(p, std::path::PathBuf::from("/home/u/.config/ziti/config.toml"));
    }

    #[test]
    fn path_is_none_without_home() {
        assert!(config_path_from(None, None).is_none());
    }

    #[test]
    fn default_config_round_trips_to_default_settings() {
        let settings = Config::default().into_settings();
        assert_eq!(settings, Settings::default());
    }

    #[test]
    fn settings_survive_toml_round_trip() {
        let original = Settings {
            mode: Mode::Once,
            device: Some("coreaudio:UID".to_string()),
            interval: 12,
            format: "{title} / {artist}".to_string(),
            osc_host: "10.0.0.2".to_string(),
            osc_port: 9000,
            osc_address: "/x/y".to_string(),
            dry_run: true,
        };
        let text = toml::to_string_pretty(&Config::from(&original)).unwrap();
        let parsed: Config = toml::from_str(&text).unwrap();
        assert_eq!(parsed.into_settings(), original);
    }

    #[test]
    fn missing_keys_fall_back_to_defaults() {
        let parsed: Config = toml::from_str("interval = 30").unwrap();
        let s = parsed.into_settings();
        assert_eq!(s.interval, 30);
        assert_eq!(s.mode, Mode::Watch); // default mode
        assert_eq!(s.osc_port, 9100);
    }
}
```

- [ ] **Step 4: テストが失敗することを確認**

Run: `cargo test --lib config`
Expected: FAIL — `Config` 未定義でコンパイルエラー。

- [ ] **Step 5: 最小実装を書く**

`src/config.rs` のテストより上に追記：

```rust
use crate::settings::{Mode, Settings};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub mode: String,
    pub device: Option<String>,
    pub interval: u64,
    pub format: String,
    pub osc_host: String,
    pub osc_port: u16,
    pub osc_address: String,
    pub dry_run: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config::from(&Settings::default())
    }
}

impl From<&Settings> for Config {
    fn from(s: &Settings) -> Self {
        Config {
            mode: match s.mode {
                Mode::Once => "once",
                Mode::Watch => "watch",
            }
            .to_string(),
            device: s.device.clone(),
            interval: s.interval,
            format: s.format.clone(),
            osc_host: s.osc_host.clone(),
            osc_port: s.osc_port,
            osc_address: s.osc_address.clone(),
            dry_run: s.dry_run,
        }
    }
}

impl Config {
    pub fn into_settings(self) -> Settings {
        Settings {
            mode: if self.mode == "once" {
                Mode::Once
            } else {
                Mode::Watch
            },
            device: self.device,
            interval: self.interval,
            format: self.format,
            osc_host: self.osc_host,
            osc_port: self.osc_port,
            osc_address: self.osc_address,
            dry_run: self.dry_run,
        }
    }
}

pub fn config_path_from(xdg: Option<&OsStr>, home: Option<&OsStr>) -> Option<PathBuf> {
    if let Some(x) = xdg {
        if !x.is_empty() {
            return Some(PathBuf::from(x).join("ziti").join("config.toml"));
        }
    }
    let home = home?;
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("ziti")
            .join("config.toml"),
    )
}

fn config_path() -> Option<PathBuf> {
    config_path_from(
        std::env::var_os("XDG_CONFIG_HOME").as_deref(),
        std::env::var_os("HOME").as_deref(),
    )
}

pub fn load() -> Settings {
    let Some(path) = config_path() else {
        return Settings::default();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Settings::default();
    };
    match toml::from_str::<Config>(&text) {
        Ok(config) => config.into_settings(),
        Err(_) => Settings::default(),
    }
}

pub fn save(settings: &Settings) -> Result<()> {
    let path = config_path().context("config パスを解決できません（HOME 未設定）")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("ディレクトリ作成に失敗: {}", parent.display()))?;
    }
    let text =
        toml::to_string_pretty(&Config::from(settings)).context("config のシリアライズに失敗")?;
    std::fs::write(&path, text).with_context(|| format!("書き込みに失敗: {}", path.display()))?;
    Ok(())
}
```

- [ ] **Step 6: テストが通ることを確認**

Run: `cargo test --lib config`
Expected: PASS（6 テスト）。

- [ ] **Step 7: 整形・lint・コミット**

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
git add src/config.rs src/lib.rs Cargo.toml Cargo.lock
git commit -m "feat: add config load/save with TOML persistence"
```

---

### Task 3: `fetch_devices`（対話用デバイス取得）

**Files:**
- Modify: `src/songrec.rs`
- Test: `src/songrec.rs`（`#[cfg(test)]`、既存に追記）

**Interfaces:**
- Consumes: 既存 `parse_device_line`, `crate::song::Device`, `crate::output::print_devices`
- Produces:
  - `pub fn parse_device_list(stderr: &str) -> Vec<Device>`（純粋：複数行 → デバイス列）
  - `pub fn fetch_devices() -> anyhow::Result<Vec<Device>>`（songrec 実行 → パース）
  - `list_devices()` を `parse_device_list` 経由に変更（出力挙動は不変）

- [ ] **Step 1: 失敗するテストを書く**

`src/songrec.rs` の `mod tests` 内に追記：

```rust
    #[test]
    fn parse_device_list_collects_multiple_and_skips_noise() {
        let stderr = "[INFO] starting\n\
[2026 INFO songrec] Available device: coreaudio:A (\u{200e}マイク)\n\
unrelated line\n\
[2026 INFO songrec] Available device: coreaudio:B (機器セット)\n";
        let devices = parse_device_list(stderr);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].id, "coreaudio:A");
        assert_eq!(devices[0].name, "マイク");
        assert_eq!(devices[1].id, "coreaudio:B");
        assert_eq!(devices[1].name, "機器セット");
    }
```

- [ ] **Step 2: テストが失敗することを確認**

Run: `cargo test --lib songrec::tests::parse_device_list_collects_multiple_and_skips_noise`
Expected: FAIL — `parse_device_list` 未定義。

- [ ] **Step 3: 実装を書く**

`src/songrec.rs` に `parse_device_list` を追加（`parse_device_line` の直後）：

```rust
pub fn parse_device_list(stderr: &str) -> Vec<crate::song::Device> {
    stderr.lines().filter_map(parse_device_line).collect()
}

pub fn fetch_devices() -> Result<Vec<crate::song::Device>> {
    let output = Command::new("songrec")
        .args(["recognize", "-l"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .context("failed to run `songrec` — is it installed and on PATH?")?;
    anyhow::ensure!(
        output.status.success(),
        "songrec exited with failure while listing devices"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(parse_device_list(&stderr))
}
```

そして既存の `list_devices` の本体を `parse_device_list` 経由に置き換える（重複した collect 式を削除）：

```rust
pub fn list_devices() -> Result<()> {
    let devices = fetch_devices()?;
    crate::output::print_devices(&devices);
    Ok(())
}
```

- [ ] **Step 4: テストが通ることを確認**

Run: `cargo test --lib songrec`
Expected: PASS（既存 + 新規）。

- [ ] **Step 5: 整形・lint・コミット**

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
git add src/songrec.rs
git commit -m "feat: add fetch_devices/parse_device_list for interactive picker"
```

---

### Task 4: `interactive` モジュール（設定一覧メニュー）

**Files:**
- Create: `src/interactive.rs`
- Modify: `src/lib.rs`（`pub mod interactive;`）, `Cargo.toml`（dialoguer 追加）
- Test: `src/interactive.rs`（`#[cfg(test)]`、純粋ヘルパーのみ）

**Interfaces:**
- Consumes: `crate::settings::{Mode, Settings}`, `crate::songrec::fetch_devices`
- Produces:
  - `pub fn menu_items(s: &Settings) -> Vec<String>`（純粋：Select 用ラベル列。index 0 = `▶ 実行`、1..=8 = 各項目、9 = `終了`）
  - `pub fn run(settings: Settings) -> anyhow::Result<Option<Settings>>`（`Some` = 実行、`None` = 終了）

- [ ] **Step 1: 依存を追加**

`Cargo.toml` の `[dependencies]` に追記：

```toml
dialoguer = "0.11"
```

- [ ] **Step 2: モジュール登録**

`src/lib.rs` に `pub mod interactive;` を追加。

- [ ] **Step 3: 失敗するテストを書く**

`src/interactive.rs` を新規作成し、テストを書く：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{Mode, Settings};

    #[test]
    fn menu_items_has_run_fields_and_quit_in_order() {
        let items = menu_items(&Settings::default());
        assert_eq!(items.len(), 10);
        assert_eq!(items[0], "▶ 実行");
        assert_eq!(items[1], "モード        監視 (watch)");
        assert_eq!(items[2], "デバイス      (指定なし)");
        assert_eq!(items[3], "間隔          10s");
        assert_eq!(items[9], "終了");
    }

    #[test]
    fn menu_reflects_settings_values() {
        let s = Settings {
            mode: Mode::Once,
            device: Some("coreaudio:X".to_string()),
            dry_run: true,
            ..Settings::default()
        };
        let items = menu_items(&s);
        assert_eq!(items[1], "モード        1回認識");
        assert_eq!(items[2], "デバイス      coreaudio:X");
        assert_eq!(items[8], "dry-run       on");
    }
}
```

- [ ] **Step 4: テストが失敗することを確認**

Run: `cargo test --lib interactive`
Expected: FAIL — `menu_items` 未定義。

- [ ] **Step 5: 実装を書く**

`src/interactive.rs` のテストより上に追記：

```rust
use crate::settings::{Mode, Settings};
use crate::songrec;
use anyhow::Result;
use dialoguer::{Confirm, Input, Select};

const RUN_LABEL: &str = "▶ 実行";
const QUIT_LABEL: &str = "終了";

fn mode_value(mode: Mode) -> &'static str {
    match mode {
        Mode::Once => "1回認識",
        Mode::Watch => "監視 (watch)",
    }
}

fn device_value(device: &Option<String>) -> String {
    match device {
        Some(d) if !d.is_empty() => d.clone(),
        _ => "(指定なし)".to_string(),
    }
}

pub fn menu_items(s: &Settings) -> Vec<String> {
    vec![
        RUN_LABEL.to_string(),
        format!("モード        {}", mode_value(s.mode)),
        format!("デバイス      {}", device_value(&s.device)),
        format!("間隔          {}s", s.interval),
        format!("フォーマット   {}", s.format),
        format!("OSC host      {}", s.osc_host),
        format!("OSC port      {}", s.osc_port),
        format!("OSC address   {}", s.osc_address),
        format!("dry-run       {}", if s.dry_run { "on" } else { "off" }),
        QUIT_LABEL.to_string(),
    ]
}

fn edit_mode(current: Mode) -> Result<Mode> {
    let options = [Mode::Once, Mode::Watch];
    let labels = [mode_value(Mode::Once), mode_value(Mode::Watch)];
    let default = if current == Mode::Watch { 1 } else { 0 };
    let idx = Select::new()
        .with_prompt("モード")
        .items(&labels)
        .default(default)
        .interact()?;
    Ok(options[idx])
}

fn edit_device(current: Option<String>) -> Result<Option<String>> {
    let devices = match songrec::fetch_devices() {
        Ok(d) => d,
        Err(err) => {
            eprintln!("デバイス一覧を取得できませんでした: {err:#}");
            Vec::new()
        }
    };
    let mut labels: Vec<String> = vec!["(指定なし / songrec デフォルト)".to_string()];
    for d in &devices {
        if d.name.is_empty() {
            labels.push(d.id.clone());
        } else {
            labels.push(format!("{} ({})", d.name, d.id));
        }
    }
    labels.push("手入力…".to_string());

    let idx = Select::new()
        .with_prompt("デバイス")
        .items(&labels)
        .default(0)
        .interact()?;

    if idx == 0 {
        Ok(None)
    } else if idx == labels.len() - 1 {
        let entered: String = Input::new()
            .with_prompt("デバイス ID")
            .allow_empty(true)
            .with_initial_text(current.unwrap_or_default())
            .interact_text()?;
        Ok(if entered.is_empty() {
            None
        } else {
            Some(entered)
        })
    } else {
        Ok(Some(devices[idx - 1].id.clone()))
    }
}

fn edit_interval(current: u64) -> Result<u64> {
    Ok(Input::<u64>::new()
        .with_prompt("認識間隔（秒）")
        .with_initial_text(current.to_string())
        .interact_text()?)
}

fn edit_port(current: u16) -> Result<u16> {
    Ok(Input::<u16>::new()
        .with_prompt("OSC port")
        .with_initial_text(current.to_string())
        .interact_text()?)
}

fn edit_text(prompt: &str, current: &str) -> Result<String> {
    Ok(Input::<String>::new()
        .with_prompt(prompt)
        .with_initial_text(current.to_string())
        .interact_text()?)
}

fn edit_dry_run(current: bool) -> Result<bool> {
    Ok(Confirm::new()
        .with_prompt("dry-run（送信せず表示のみ）")
        .default(current)
        .interact()?)
}

pub fn run(mut settings: Settings) -> Result<Option<Settings>> {
    loop {
        let items = menu_items(&settings);
        let choice = Select::new()
            .with_prompt("ziti — 項目を選んで編集、▶ 実行で開始")
            .items(&items)
            .default(0)
            .interact()?;
        match choice {
            0 => return Ok(Some(settings)),
            1 => settings.mode = edit_mode(settings.mode)?,
            2 => settings.device = edit_device(settings.device)?,
            3 => settings.interval = edit_interval(settings.interval)?,
            4 => settings.format = edit_text("送信フォーマット", &settings.format)?,
            5 => settings.osc_host = edit_text("OSC host", &settings.osc_host)?,
            6 => settings.osc_port = edit_port(settings.osc_port)?,
            7 => settings.osc_address = edit_text("OSC address", &settings.osc_address)?,
            8 => settings.dry_run = edit_dry_run(settings.dry_run)?,
            _ => return Ok(None),
        }
    }
}
```

- [ ] **Step 6: テストが通ることを確認**

Run: `cargo test --lib interactive`
Expected: PASS（2 テスト）。

- [ ] **Step 7: 整形・lint・コミット**

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
git add src/interactive.rs src/lib.rs Cargo.toml Cargo.lock
git commit -m "feat: add interactive 設定一覧 menu module"
```

---

### Task 5: `main.rs` 配線（起動分岐と dispatch 合流）

**Files:**
- Modify: `src/main.rs`
- Test: `src/main.rs`（`#[cfg(test)]`、`should_use_interactive` のみ）

**Interfaces:**
- Consumes: `ziti::settings::{Mode, Settings}`, `ziti::config`, `ziti::interactive`, 既存 `format`/`osc`/`output`/`songrec`/`song`
- Produces: バイナリ挙動。`should_use_interactive(arg_count: usize, stdin_is_tty: bool) -> bool`

- [ ] **Step 1: 失敗するテストを書く**

`src/main.rs` 末尾に追記：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interactive_only_with_no_args_and_tty() {
        assert!(should_use_interactive(1, true));
        assert!(!should_use_interactive(1, false)); // パイプ実行
        assert!(!should_use_interactive(2, true)); // フラグあり
        assert!(!should_use_interactive(3, false));
    }
}
```

- [ ] **Step 2: テストが失敗することを確認**

Run: `cargo test --bin ziti`
Expected: FAIL — `should_use_interactive` 未定義。

- [ ] **Step 3: `main.rs` を書き換える**

`src/main.rs` 全体を以下に置き換える：

```rust
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
                    output::print_error(&format!("config を保存できませんでした: {err:#}"));
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
```

- [ ] **Step 4: テストが通ることを確認**

Run: `cargo test --bin ziti`
Expected: PASS。

- [ ] **Step 5: 全テスト・lint を確認**

Run: `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`
Expected: 全テスト PASS、警告ゼロ。

- [ ] **Step 6: 手動動作確認**

```bash
# 対話モード（TTY）：メニューが出る → ▶ 実行 で開始、終了で抜ける
cargo run --quiet

# 非対話：従来どおり即実行・list・dry-run
cargo run --quiet -- --list
cargo run --quiet -- --dry-run -d "<any device id>"

# パイプ（非 TTY）：対話に入らず 1 回認識へフォールバック
echo "" | cargo run --quiet
```
Expected:
- 引数なし TTY → 設定一覧メニュー表示、保存後に選んだモードで動作。
- `--list` → 従来のデバイス一覧。
- パイプ → メニューを出さず 1 回認識（曲が無ければ `no song recognized`）。

- [ ] **Step 7: コミット**

```bash
git add src/main.rs
git commit -m "feat: launch interactive mode on bare ./ziti, unify dispatch"
```

---

## Self-Review

**Spec coverage**
- 起動分岐（TTY/非TTY/フラグ）→ Task 5 `should_use_interactive` + `run`。✓
- 設定一覧型 UI → Task 4 `menu_items` + `run` ループ。✓
- mode デフォルト Watch → Task 1 `Settings::default` + Task 2 `Config` default。✓
- 永続化（XDG/HOME パス・load/save・壊れ時フォールバック）→ Task 2。✓
- `fetch_devices` / list 統合 → Task 3。✓
- モジュール分離（settings/config/interactive/songrec/main）→ 各タスク。✓
- 既存フラグ挙動維持・`--list` 温存・非TTYフォールバック → Task 5。✓
- テスト方針（純粋関数を assert_eq、ループは薄く保つ）→ Task 1/2/3/4/5 のテスト。✓

**Placeholder scan**：TODO/TBD・曖昧な「適切に処理」等なし。各コードステップは完全なコードを含む。✓

**Type consistency**
- `Settings`/`Mode` のフィールド名・型は Task 1 定義と Task 2/4/5 の使用で一致。
- `config::load() -> Settings`（Config ではない）を Task 5 が前提化、Task 2 の定義と一致。✓
- `interactive::run(Settings) -> Result<Option<Settings>>` を Task 5 が使用、Task 4 定義と一致。✓
- `fetch_devices() -> Result<Vec<Device>>` を Task 4 `edit_device` が使用、Task 3 定義と一致。✓
- `menu_items` の index 割当（0=実行, 1..=8=項目, 9=終了）が Task 4 の `run` の match と一致。✓
