# syntax=docker/dockerfile:1.6

FROM quay.io/pypa/manylinux2014_x86_64 AS builder

ARG RUST_VERSION=1.93.0

ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    PATH=/opt/cargo/bin:${PATH}

RUN curl -sSf https://sh.rustup.rs | sh -s -- \
    -y \
    --profile minimal \
    --default-toolchain ${RUST_VERSION} \
    --default-host x86_64-unknown-linux-gnu

WORKDIR /work

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY bindings ./bindings

RUN cargo build -p morph-da-codec-ffi --release --locked

RUN mkdir -p /out && \
    cp target/release/libmorph_da_zstd.a \
       /out/libmorph_da_zstd_linux_amd64.a

FROM scratch AS artifact

COPY --from=builder /out/ /
