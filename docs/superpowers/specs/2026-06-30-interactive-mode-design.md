# ziti 対話モード（設定一覧型）設計

- 日付: 2026-06-30
- 状態: 承認済み（実装前）

## 目的

`./ziti` を引数なしで実行したとき、フラグを覚えなくても全オプションを対話的に
設定して実行できるようにする。設定は「一覧型」UIで提示し、編集したい項目だけを
選んで直し、`▶ 実行` で確定する。選んだ設定は次回起動時の初期値として保存する。

従来のフラグ指定（`./ziti -d X --watch ...`）は非対話で即実行し、挙動を変えない。

## 起動の分岐（`main.rs`）

```
引数なし & stdin が TTY    → 対話モード
引数あり（--list/--help 等含む） → 従来通り即実行（clap パース）
引数なし & 非 TTY（パイプ）   → 従来の 1 回認識にフォールバック（対話 UI を出せないため）
```

- 「引数なし」の判定は `std::env::args().len() == 1`（プログラム名のみ）で行う。
  これにより `./ziti --list` などフラグ付きは確実に clap 側へ流れる。
- 非 TTY 判定は `std::io::stdin().is_terminal()` を用いる。

## 対話画面（設定一覧型）

`dialoguer` の `Select` をメインメニューにしたループ。各項目は現在値をインラインに表示。

```
┌─ ziti ─────────────────────────────────────┐
  モード        監視 (watch)
  デバイス      BlackHole 2ch
  間隔          10s
  フォーマット   {artist} - {title}
  OSC           udp://127.0.0.1:9100  /cannelloni/search
  dry-run       off
└─────────────────────────────────────────────┘
? › ❯ ▶ 実行
      モード / デバイス / 間隔 / フォーマット / OSC host / port / address / dry-run
      終了
```

- 項目選択 → その場で編集（モード/デバイスは `Select`、数値・文字列は `Input`、
  dry-run は `Confirm`）→ 一覧へ戻る。
- `▶ 実行` で確定。`終了` で何もせず抜ける（exit code 0）。
- デバイス編集時は `songrec::fetch_devices()` で一覧を取得し、`Select` で選ばせる。
  先頭に「(指定なし / songrec デフォルト)」、末尾に「手入力…」を置く。
- list は「モード」ではなく「デバイス編集」内の一覧表示に統合する
  （独立した `--list` フラグは非対話用として従来どおり残す）。

## モジュール構成（単一責任で分離）

| ファイル | 責務 |
|---|---|
| `settings.rs` | `Settings`（解決済み設定）+ `Mode { Once, Watch }` enum、`From<&Cli>` |
| `config.rs` | TOML の `Config` 読み書き、パス解決、`Config`↔`Settings` 変換 |
| `interactive.rs` | 設定一覧メニューのループ。純粋関数 `summary_lines(&Settings)->String` と I/O を分離 |
| `songrec.rs` | `fetch_devices() -> Result<Vec<Device>>` を追加（既存 `list_devices` はこれを使う形へ） |
| `main.rs` | 起動分岐。対話で `Settings` を組み立て → `config::save` → 既存の `recognize_once`/`stream_listen` へ |

### `Settings` / `Mode`

```rust
pub enum Mode { Once, Watch }

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
```

- `From<&Cli>`：`mode` は `cli.watch` から決定（`list` はここでは扱わない）。
- 既存の `Cli` のデフォルト値（interval=10, format, osc_* など）と一致させる。
  デフォルトは一箇所（`Settings::default()` か定数）に集約し、`Cli` と `Config` から参照する。
- **`Settings::default().mode` / `Config` のデフォルト mode は `Watch`**。
  対話モードを `./ziti`（引数なし）で起動したとき、設定ファイルが無ければ
  初期選択は「監視 (watch)」になる。
- 非対話のフラグ経路は従来どおり `--watch` を明示しない限り 1 回認識のまま
  （clap の `watch: bool` デフォルト false を維持し、既存挙動を壊さない）。
  非 TTY フォールバックも従来どおり 1 回認識。

### `config.rs`（永続化）

- パス解決: `$XDG_CONFIG_HOME/ziti/config.toml`、無ければ `$HOME/.config/ziti/config.toml`。
  （`directories` クレートは使わず `std::env` で解決し、依存を増やさない）
- `load() -> Config`: ファイルが無い/壊れている場合はデフォルトにフォールバック（エラーで落とさない）。
- `save(settings: &Settings) -> Result<()>`: 親ディレクトリを作成して TOML を書き出す。
- `Config` は serde の `Serialize`/`Deserialize`。欠損フィールドは `#[serde(default)]` でデフォルト補完。
- `mode` は文字列（`"once"` / `"watch"`）として保存。

### `interactive.rs`

- `run(initial: Settings, devices_fn) -> Result<Option<Settings>>`
  - `Some(settings)`: `▶ 実行` が選ばれた
  - `None`: `終了` が選ばれた
- 純粋関数として切り出してテストする:
  - `summary_lines(&Settings) -> String`（一覧の表示文字列）
  - `menu_label(field, &Settings) -> String`（「モード        1回認識」のような行）
- dialoguer 依存部分（ループ本体）は薄く保ち、ロジックは純粋関数へ寄せる。

## エラー処理

- 設定ファイルの読み込み失敗はサイレントにデフォルトへフォールバック（致命的でない）。
- 設定ファイルの書き込み失敗は警告を出しつつ実行は継続（`output::print_error` 相当の warn）。
- `fetch_devices()` 失敗時は対話のデバイス編集で「手入力」のみ提示し、原因を 1 行表示。
- 既存の認識失敗（`no song recognized`）等の挙動は変えない。

## テスト方針

既存の `device_lines`/`print_devices`（純粋関数＋I/O 分離）と同じ流儀で:

- `summary_lines` / `menu_label` の出力を `assert_eq!` で検証。
- `Config` ↔ `Settings` 変換と TOML ラウンドトリップ（serialize→deserialize で一致）。
- `Settings::from(&Cli)`：フラグ指定が正しく `Settings` に反映されること。
- `config::load` のパス解決（`XDG_CONFIG_HOME` を env で差し替えて検証）。
- dialoguer のループ本体（端末入力）は単体テスト対象外。薄く保つことで担保。

## 追加依存

- `dialoguer`（対話 UI: Select / Input / Confirm）
- `toml`、`serde`（`derive` feature）

## 非対象（YAGNI）

- 設定プロファイルの複数管理。
- 対話中の OSC 送信テスト/プレビュー実行。
- 色テーマのカスタマイズ（既存の dim/green 方針を踏襲）。
