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

Windows の場合は [Windows で使う](#windows-で使う) を参照してください。

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

## Windows で使う

`ziti` 自体は純粋な Rust なので任意の Windows 用 Rust ツールチェインでビルドできますが、
`songrec` は CLI のみのビルドでも GNOME 系のネイティブライブラリ（glib, libsoup3, gettext）に
依存しており、ビルド済み Windows バイナリも配布されていません。手段は 2 つあります。
Docker コンテナですべて動かす方法（推奨 — songrec の本来のプラットフォームである Linux 上で
そのままビルドできます）と、MSYS2 でネイティブビルドする方法です。

### Docker（推奨）

音声は WSLg の PulseAudio ブリッジ経由でコンテナに入り、OSC はただの UDP として
コンテナから出ていきます。コード変更も MSYS2 も不要です。

前提条件:

- Docker — WSL2 バックエンドの Docker Desktop、または WSL2 ディストリビューション内に
  インストールした Docker Engine。
- Windows ホスト側の [VB-CABLE](https://vb-audio.com/Cable/)。ループバックはホストの
  仕事のままで、コンテナは WSLg が転送してきた音を読むだけです。

Windows 側のオーディオ設定（設定 > システム > サウンド）:

- 既定の**再生**デバイス → 「CABLE Input」。再生した音がすべてケーブルに流れます。
- 既定の**録音**デバイス → 「CABLE Output」。WSLg の RDPSource は既定の録音デバイスを
  キャプチャして Linux 側に公開します。

```
Windows audio (既定の再生デバイス)
        │
        ▼
   VB-CABLE  (CABLE Input → CABLE Output, 既定の録音デバイス)
        │
        ▼
   WSLg PulseAudio  (/mnt/wslg/PulseServer)
        │   volume mount + PULSE_SERVER
        ▼
   container:  songrec ──▶ ziti
        │
        ▼  (UDP OSC)
   host.docker.internal (Windows ホスト)  /  LAN 上の任意のホスト
```

使い方（WSL2 ディストリビューション内のリポジトリチェックアウトから）:

```sh
# イメージをビルドし、コンテナから見えるオーディオデバイスを一覧
docker compose run --rm ziti --list

# 常駐監視して Windows ホスト上のプログラムへ OSC を送信
docker compose run --rm ziti --watch --osc-host host.docker.internal

# または compose.yaml の `command:` のコメントを外して
docker compose up
```

Windows ホスト上の受信側へ届けるには `--osc-host host.docker.internal` を指定します。
LAN 上の別ホストへは `--osc-host <ip>` でそのまま届きます。その他のフラグ・設定ファイル・
対話モードはすべて同じです。songrec が Pulse ソースを自動選択しない場合は
`-d alsa:pulse` を付けてください。

注意点:

- `/mnt/wslg` のマウントは Microsoft 公式の WSLg コンテナサンプルに沿ったもので、
  WSL2 ディストリビューション内で動かす Docker Engine で最も確実に動作します。
  Docker Desktop 自身の VM からはディストリビューションの `/mnt/wslg` が見えない
  場合があります。
- オーディオ経路は実際の Windows マシンではまだ検証していません。
- イメージのビルドは linux/arm64 でのみスモークテスト済みです。

### MSYS2 でネイティブビルド

MSYS2 の UCRT64 環境は SongRec 本家の公式 Windows 手順で、songrec に必要なネイティブ
ライブラリがすべて揃います。

1. [MSYS2](https://www.msys2.org/) をインストールし、**UCRT64** シェルを開きます。

2. ビルドに必要な依存をインストールします（本家 SongRec の UCRT64 リストから
   GUI 専用パッケージを除いたもの）。

   ```sh
   pacman -S mingw-w64-ucrt-x86_64-rust mingw-w64-ucrt-x86_64-gcc \
             mingw-w64-ucrt-x86_64-pkgconf mingw-w64-ucrt-x86_64-glib2 \
             mingw-w64-ucrt-x86_64-libsoup3 mingw-w64-ucrt-x86_64-gettext-runtime \
             mingw-w64-ucrt-x86_64-openssl mingw-w64-ucrt-x86_64-ffmpeg
   ```

3. songrec をインストールします（CLI のみ・GUI なし）。

   ```sh
   cargo install songrec --no-default-features --features ffmpeg
   ```

4. ziti のチェックアウトディレクトリから、同じシェルで ziti をインストールします。

   ```sh
   cargo install --path .
   ```

5. オーディオループバックを設定します。[VB-CABLE](https://vb-audio.com/Cable/) を
   インストールし、Windows の既定の再生デバイスを「CABLE Input」に設定（設定 > システム >
   サウンド）した上で、デバイス一覧から「CABLE Output」のエントリを `-d` に渡します。

   ```sh
   ziti --list
   ziti --watch -d "<--list に表示された CABLE Output デバイス>"
   ```

   サウンドドライバが「ステレオ ミキサー (Stereo Mix)」を提供している場合は、VB-CABLE の
   代わりに使えます。Windows のデバイス名は WASAPI 由来のため、macOS の `coreaudio:...`
   形式とは見た目が異なります。必ず `--list` の出力から選んでください。

フラグ・設定ファイル・対話モードはすべて macOS/Linux と同じように動作します。なお、この手順は
SongRec 本家の公式 MSYS2 手順に沿ったものですが、実際の Windows マシンではまだ検証していません。

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
