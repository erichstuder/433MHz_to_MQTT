FROM rust:1.83.0

# Add target toolchain
RUN rustup self update && \
    rustup update stable && \
	rustup target add thumbv6m-none-eabi

# Install tools
# TODO: Can't these tools be moved to Cargo.toml? Why not?
RUN apt update && \
    apt install -y libudev-dev && \
    cargo install elf2uf2-rs@2.0.0 && \
    cargo install flip-link@0.1.2

# Install picotool (see: https://github.com/raspberrypi/picotool)
WORKDIR /usr
RUN git clone --depth 1 --branch 1.5.1 https://github.com/raspberrypi/pico-sdk.git && \
    git clone --depth 1 --branch 1.1.2 https://github.com/raspberrypi/picotool.git && \
    apt update && \
    apt install -y build-essential pkg-config libusb-1.0-0-dev cmake && \
    cd picotool && \
    mkdir build && \
    cmake -E chdir build cmake -DPICO_SDK_PATH="../../pico-sdk" .. && \
    cmake --build build && \
    cmake --install build && \
    cp udev/99-picotool.rules /lib/udev/rules.d/

# Pre-fetch dependencies with a dummy project. This makes the later build much faster.
WORKDIR /usr/dependencies_fetch_project
# add dummy lib to fulfill dependency in Cargo.toml
RUN cargo init --lib app
WORKDIR /usr/dependencies_fetch_project/dummy
RUN cargo init
COPY ./firmware/Cargo.toml .
RUN cargo fetch
COPY ./app/Cargo.toml .
RUN cargo fetch

RUN echo "umask 0000" >> /root/.bashrc
ENV BASH_ENV=/root/.bashrc
