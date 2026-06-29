# ziti

[English README](README.md)

[songrec](https://github.com/marin-m/SongRec) で認識した「いま流れている曲」を、OSC メッセージとして任意の宛先へ転送する小さな Rust 製 CLI です。

```
system audio (VB-Cable 等)
        │
        ▼
   songrec recognize / listen  (-j JSON)
        │   track.subtitle = artist, track.title = title
        ▼
   ziti  ──(OSC string)──▶  udp://host:port  /cannelloni/search
```

デフォルトでは `"アーティスト - 曲名"` を OSC string 1 引数として `udp://127.0.0.1:9100` の
アドレス `/cannelloni/search` へ送ります。宛先・アドレス・文字列フォーマットはすべて引数で変更できます。

## 仕組み

外部の `songrec` バイナリを子プロセスとして起動し、その `-j`（コンパクトな 1 行 JSON）出力を
パースして曲情報を取り出します。songrec をライブラリとしてリンクする方式は、`soup3`(libsoup) など
重いネイティブ依存を巻き込むため採用していません。サブプロセス方式なので依存が軽く、songrec 本体は
GPL ですが、バイナリを呼び出すだけのこの CLI には GPL は伝播しません。

songrec 自身のログ出力（stderr に出る `INFO …` 行）は抑制しているので、表示されるのは `ziti` の
出力だけです。

## 必要なもの

- Rust ツールチェイン（edition 2021 / rustc 1.70+）
- `songrec` バイナリが PATH 上にあること

```sh
# 例: macOS で FFmpeg 機能付き・GUI なしでインストール
cargo install songrec --no-default-features --features ffmpeg
```

## ビルド

```sh
# 依存取得 + デバッグビルド（実行ファイル: target/debug/ziti）
cargo build

# 最適化したリリースビルド（実行ファイル: target/release/ziti）
cargo build --release

# ビルドせず直接実行（引数は -- の後ろに渡す）
cargo run -- --list
cargo run --release -- --watch -d "<デバイス名>"
```

ビルドした実行ファイルはそのままコピーして配置できます。

```sh
cp target/release/ziti /usr/local/bin/   # 任意の PATH へ
```

## インストール

`cargo install` で PATH 上に直接インストールすることもできます。

```sh
cargo install --path .       # ~/.cargo/bin/ziti に配置
# またはローカル実行
cargo run -- --help
```

## 使い方

```sh
# 利用可能なオーディオデバイス一覧を表示して終了
ziti --list

# 単発: 1 曲認識して OSC を送ったら終了（曲が見つからなければ非 0 終了）
ziti

# 常駐: 新しい曲を認識するたびに送信（直前と同じ曲は送らない）。Ctrl-C で停止
ziti --watch -d "<デバイス名/UID>"

# 送信せず、送信予定の内容だけ表示
ziti --dry-run

# OSC 宛先・アドレス・フォーマットを変更
ziti --osc-host 192.168.1.10 --osc-port 9000 \
     --osc-address /myapp/nowplaying \
     --format "{title} / {artist}"
```

デバイス名は `--list` の出力から選びます（macOS + VB-Cable の例:
`coreaudio:com.vbaudio.vbcable:XXXXXXXX-...`）。UID はマシンや再インストールで変わるため、
固定値を控えるより `--list` で都度確認するのが確実です。

## オプション

| 引数 | 既定値 | 説明 |
|------|--------|------|
| `-l, --list` | — | オーディオデバイス一覧を表示して終了 |
| `-d, --device <NAME>` | システム既定 | 使用する音声デバイス（songrec の `-d` に渡す） |
| `--watch` | off | 常駐監視。新しい曲ごとに送信（連続同一はスキップ） |
| `-i, --interval <SEC>` | 10 | Shazam へのリクエスト間隔（songrec の `-i`） |
| `--format <TMPL>` | `{artist} - {title}` | 送信文字列テンプレート（`{artist}` `{title}` を置換） |
| `--osc-host <HOST>` | 127.0.0.1 | OSC 送信先ホスト |
| `--osc-port <PORT>` | 9100 | OSC 送信先ポート |
| `--osc-address <ADDR>` | /cannelloni/search | OSC アドレスパターン |
| `--dry-run` | off | OSC を送信せず内容のみ表示 |

## 出力

認識と送信を 2 行のログで表示します。成功/失敗は色だけでなく記号（♪ ✓ ✗ →）でも示すため、
`NO_COLOR` 指定時やパイプ経由でも情報が失われません。

```
[12:30:34] ♪ recognized
           Mirin Sheeno - Harmony
   → ✓ OSC udp://127.0.0.1:9100 /cannelloni/search
```

`--dry-run` のときは送信行が `→ (dry-run) would send …` になります。

## 挙動メモ

- 送る OSC メッセージは **string 型の引数 1 個**（整形済みの曲名）だけです。
- `--watch` では送信に失敗しても警告を出して監視を継続します（単発モードは失敗で非 0 終了）。
- 曲の重複判定は「整形後の文字列」で行うため、`--format` を変えると判定単位も変わります。
- songrec 自身の stderr ログは抑制され、表示されるのは `ziti` の出力だけです。

## 開発

```sh
cargo test                                   # 全テスト
cargo fmt --all -- --check                   # フォーマット確認
cargo clippy --all-targets -- -D warnings    # lint
```

git hook は [rusty-hook](https://github.com/swellaby/rusty-hook) で担保しています
（`.rusty-hook.toml`）。pre-commit で `fmt --check` + `clippy -D warnings`、pre-push で
`cargo test` が自動実行されます。

設計と実装計画は `docs/superpowers/` 配下にあります。

## ライセンス

このリポジトリのコードは利用者の方針に従ってください。なお `songrec` 本体は GPL-3.0+ ですが、
本 CLI は songrec を外部バイナリとして呼び出すのみで、ソースを取り込んでいません。
