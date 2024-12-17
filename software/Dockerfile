FROM rust:1.83.0

RUN apt update && apt install -y \
    libudev-dev=252.31-1~deb12u1

ARG USER
RUN useradd -m $USER
USER $USER

RUN id -u && id -g

RUN rustup target add thumbv6m-none-eabi

RUN cargo install \
    elf2uf2-rs@2.1.1 \
    flip-link@0.1.9

# Pre-fetch dependencies. This makes the later build much faster.
# - Add a temporary lib to fulfill dependency in Cargo.toml.
WORKDIR /home/$USER/dependencies_fetch_project
RUN cargo init --lib app
# - Use a dummy project to fetch dependencies.
WORKDIR /home/$USER/dependencies_fetch_project/dummy
RUN cargo init
COPY ./firmware/Cargo.toml .
RUN cargo fetch
COPY ./app/Cargo.toml .
RUN cargo fetch
