# syntax=docker/dockerfile:1.6
FROM rustlang/rust:nightly

ARG DEBIAN_FRONTEND=noninteractive

RUN --mount=type=cache,target=/var/cache/apt \
    --mount=type=cache,target=/var/lib/apt/lists \
    apt-get update \
    && apt-get install -y --no-install-recommends \
       zsh \
       git \
       build-essential \
       pkg-config \
       libssl-dev \
       openssl \
       cmake \
       curl \
       ca-certificates \
       unzip \
    && rm -rf /var/lib/apt/lists/*

RUN chsh -s /bin/zsh root

WORKDIR /workspace

COPY data/ops/scripts/install_rust_tooling.sh /tmp/install_rust_tooling.sh
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    chmod +x /tmp/install_rust_tooling.sh \
    && /tmp/install_rust_tooling.sh \
    && rm /tmp/install_rust_tooling.sh
