FROM rust:latest

# Pre-fetch dependencies with a dummy project. This makes the later build much faster.
WORKDIR /usr/dependencies_fetch_project
RUN cargo init
COPY ./Cargo.toml .
RUN cargo fetch
