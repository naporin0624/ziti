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
        curl \
        patch \
    && rm -rf /var/lib/apt/lists/*

# songrec first so ziti source changes don't invalidate this slow layer.
# "pulse" lets songrec follow the default PulseAudio source (WSLg's RDPSource).
# The patch averages channels on downmix; upstream keeps only the left one.
ARG SONGREC_VERSION=0.7.4
COPY patches/songrec-${SONGREC_VERSION}-downmix.patch /tmp/songrec-downmix.patch
RUN curl -fsSL "https://static.crates.io/crates/songrec/songrec-${SONGREC_VERSION}.crate" \
        | tar xz -C /tmp \
    && cd "/tmp/songrec-${SONGREC_VERSION}" \
    && patch -p1 < /tmp/songrec-downmix.patch \
    && cargo install --locked --path . --no-default-features --features pulse \
    && rm -rf "/tmp/songrec-${SONGREC_VERSION}" /tmp/songrec-downmix.patch

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
