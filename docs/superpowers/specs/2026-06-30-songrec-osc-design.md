# songrec-osc 設計ドキュメント

作成日: 2026-06-30

## 目的

songrec で認識した楽曲を「アーティスト - 曲名」の文字列にし、OSC メッセージとして
任意のホスト/ポート/アドレス（既定 `udp://127.0.0.1:9100` の `/cannelloni/search`）へ
送信する Rust 製 CLI。VRChat 等の OSC 受信側で「いま流れている曲」を検索トリガーに使う用途を想定する。

```
system audio (VB-Cable 等)
        │
        ▼
   songrec (subprocess, -j JSON)
        │  Song { artist, title }
        ▼
   songrec-osc  ──(OSC string)──▶  udp://host:port  /cannelloni/search
```

## 採用アプローチ: songrec サブプロセス方式

インストール済みの `songrec` バイナリを子プロセスとして起動し、`-j`（JSON）出力をパースする。
songrec をライブラリリンクする案は soup3 等の非オプショナル依存でビルドが破綻するため不採用
（過去セッションで確認済み）。サブプロセス方式は依存が軽く、songrec の更新にも追従しやすい。

## モジュール構成（単一責任）

| ファイル | 責任 |
|----------|------|
| `src/main.rs` | エントリポイント。引数パース結果を受けてモード分岐のみ |
| `src/cli.rs` | clap derive による引数定義（`Cli` struct） |
| `src/songrec.rs` | songrec を spawn し、出力 1 行を `Song { artist, title }` に変換 |
| `src/song.rs` | `Song` 型と「同一曲」判定（dedup キー） |
| `src/format.rs` | `Song` + テンプレート文字列 → 送信文字列のレンダリング |
| `src/osc.rs` | OSC メッセージ生成（rosc）と `UdpSocket` 送信 |
| `src/output.rs` | ターミナルへの 2 行リッチログ出力（色＋記号の二重符号化） |
| `src/lib.rs` | 上記モジュールの公開とライブラリ API |

各モジュールは内部実装を隠蔽し、`Song` 値型と関数シグネチャのみで連携する。
single-responsibility を守り、テスト時は songrec を呼ばずに `Song` を直接組み立てて検証できる。

## CLI 引数

| 引数 | 既定値 | 説明 |
|------|--------|------|
| `-l, --list` | — | `songrec recognize -l` を実行しデバイス一覧を表示して終了 |
| `-d, --device <NAME>` | システム既定 | songrec の `-d` に渡す音声デバイス |
| `--watch` | off | 常駐監視モード（`songrec listen -j` をストリーム読み） |
| `-i, --interval <SEC>` | 10 | songrec のリクエスト間隔（`-i`） |
| `--format <TMPL>` | `{artist} - {title}` | 送信文字列テンプレート |
| `--osc-host <HOST>` | 127.0.0.1 | OSC 送信先ホスト |
| `--osc-port <PORT>` | 9100 | OSC 送信先ポート |
| `--osc-address <ADDR>` | /cannelloni/search | OSC アドレスパターン |
| `--dry-run` | off | OSC を送信せず、送信予定の内容のみ表示 |

## 動作フロー

### 単発（既定）
1. `songrec recognize -j [-d <device>] [-i <sec>]` を起動。
2. 標準出力の JSON を 1 件読む。
3. `track.subtitle` = artist, `track.title` = title を抽出して `Song` を作る。
4. テンプレートで文字列化し、OSC で送信。
5. 終了。曲が見つからない（no-match）場合は何も送らず **非 0 終了**。

### 常駐（`--watch`）
1. `songrec listen -j [-d <device>] [-i <sec>]` をストリーム起動。
2. JSON 行を逐次読み、`Song` 化。
3. **直前に送った曲と同一なら送らない**（連続同一スキップ）。
4. 新しい曲のみ OSC 送信。
5. no-match の行は無視して継続。Ctrl-C で停止。

## dedup と no-match の方針

- dedup キー: 整形後の送信文字列（既定では `"{artist} - {title}"`）の完全一致。直前の 1 曲のみ保持。
- no-match: 単発は非 0 終了、watch は無視して次へ。OSC での失敗通知は行わない。

## OSC 仕様

- トランスポート: UDP。
- メッセージ: アドレス `--osc-address`、引数は **OSC string 型 1 個**（整形済み曲名）。
- 送信は `std::net::UdpSocket`（`0.0.0.0:0` を bind して `host:port` へ send_to）。
- エンコードは `rosc::encoder::encode(OscPacket::Message(...))`。

## ターミナル出力（2 行リッチログ）

色だけに依存せず、記号と併用（WCAG「色のみに依存しない」原則をターミナルに適用）。
成功=緑+`✓`、エラー=赤+`✗`、副次情報=dim。カラー無効時も記号で判別可能。

```
[12:30:34] ♪ recognized
           Mirin Sheeno - Harmony
   → ✓ OSC udp://127.0.0.1:9100 /cannelloni/search
```

`--dry-run` 時は送信行を `→ (dry-run) would send …` に置き換える。

## songrec JSON の想定スキーマ

`songrec recognize -j` は Shazam API レスポンスを出力する。利用フィールド:

- `track.title` … 曲名
- `track.subtitle` … アーティスト名

いずれかが欠落/`track` が無い行は no-match として扱う。パースは serde_json で防御的に行い、
未知フィールドは無視する。

## エラーハンドリング

- songrec バイナリが見つからない → 明確なメッセージで非 0 終了。
- JSON パース失敗（1 行）→ watch では警告ログのみ出して継続、単発では非 0 終了。
- OSC 送信失敗（UDP）→ 非 0 終了（watch では警告して継続）。

## テスト方針

- `song.rs`: dedup 判定の単体テスト。
- `format.rs`: テンプレート展開（`{artist}` `{title}` 置換、未知トークンの扱い）の単体テスト。
- `songrec.rs`: 実際のサンプル JSON 文字列 → `Song` 変換のパーステスト（songrec は起動しない）。
- `osc.rs`: ローカル `UdpSocket` を受信側に立て、エンコード/送信のラウンドトリップを検証。

## 依存クレート

- `clap`（v4, derive） … 引数
- `rosc` … OSC エンコード
- `serde_json` … songrec JSON パース
- `anyhow` … エラー
- dev: `rusty-hook`（v0.11） … git hook

## ツールチェイン / 品質ゲート（bucatini 準拠）

CLAUDE.md の「husky で lint/test 担保」は、Rust では bucatini と同じく **rusty-hook** で実現する。

`.rusty-hook.toml`:
```toml
[hooks]
pre-commit = "cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings"
pre-push = "cargo test"

[logging]
verbose = true
```

CI（`.github/workflows/ci.yml`）は安価な Linux ランナーで `cargo fmt --all -- --check` のみ。

## Cargo 構成（bucatini 準拠）

- edition 2021、`[[bin]]` name = `songrec-osc` (path `src/main.rs`)、`[lib]` name = `songrec_osc`。

## スコープ外（YAGNI）

- songrec のライブラリ直接利用、MPRIS 連携、GUI。
- OSC bundle / 複数引数。string 1 個のみ。
- 履歴 CSV 出力（既存 `songrec-vbcable.sh` の役割）。
