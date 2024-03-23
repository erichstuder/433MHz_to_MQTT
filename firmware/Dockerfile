FROM rust:latest

# Add target toolchain
RUN rustup self update && \
    rustup update stable && \
	rustup target add thumbv6m-none-eabi

# Install tools
# TODO: Can't these tools be moved to Cargo.toml? Why not?
RUN apt update && \
    apt install libudev-dev && \
    cargo install elf2uf2-rs@2.0.0 && \
    cargo install flip-link@0.1.2

# Pre-fetch dependencies with a dummy project. This makes the later build much faster.
WORKDIR /usr/dummy
RUN cargo init
COPY Cargo.toml .
RUN cargo fetch
