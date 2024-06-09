FROM rust:latest

RUN apt-get update && \
    apt-get -y install libudev-dev

# Pre-fetch dependencies with a dummy project. This makes the later build much faster.
WORKDIR /usr/dependencies_fetch_project
RUN cargo init
COPY ./steps/Cargo.toml .
RUN cargo fetch
