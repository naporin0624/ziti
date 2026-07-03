# Builder: compile songrec (CLI only) and ziti.
# trixie, not bookworm: songrec's soup3-sys needs libsoup-3.0 >= 3.4.
FROM rust:trixie AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config \
        clang \
        libclang-dev \
        libssl-dev \
        libglib2.0-dev \
        libsoup-3.0-dev \
        gettext \
        libasound2-dev \
        libpipewire-0.3-dev \
        libpulse-dev \
    && rm -rf /var/lib/apt/lists/*

# songrec first so ziti source changes don't invalidate this slow layer.
# "pulse" lets songrec follow the default PulseAudio source (WSLg's RDPSource).
RUN cargo install songrec --no-default-features --features pulse

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Runtime: slim image with only the shared libraries the two binaries load.
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3t64 \
        libglib2.0-0t64 \
        libsoup-3.0-0 \
        libasound2t64 \
        libasound2-plugins \
        libpulse0 \
        libpipewire-0.3-0t64 \
    && rm -rf /var/lib/apt/lists/*

# Route ALSA (cpal's default host on Linux) to PulseAudio so audio reaches
# the socket named by PULSE_SERVER (e.g. WSLg's /mnt/wslg/PulseServer).
RUN printf 'pcm.!default {\n    type pulse\n}\nctl.!default {\n    type pulse\n}\n' > /etc/asound.conf

COPY --from=builder /usr/local/cargo/bin/songrec /usr/local/bin/songrec
COPY --from=builder /build/target/release/ziti /usr/local/bin/ziti

ENTRYPOINT ["ziti"]
CMD ["--help"]
