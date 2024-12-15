FROM rust:1.83.0

RUN apt update && apt install -y \
    libudev-dev=252.31-1~deb12u1

RUN rustup target add thumbv6m-none-eabi

RUN cargo install \
    elf2uf2-rs@2.1.1 \
    flip-link@0.1.9

# picotool (see: https://github.com/raspberrypi/picotool)
WORKDIR /usr
RUN apt update && apt install -y \
    build-essential=12.9 \
    pkg-config=1.8.1-1 \
    libusb-1.0-0-dev=2:1.0.26-1 \
    cmake=3.25.1-1
RUN git clone --depth 1 --branch 2.1.0 https://github.com/raspberrypi/pico-sdk.git && \
    git clone --depth 1 --branch 2.1.0 https://github.com/raspberrypi/picotool.git && \
    cd picotool && \
    mkdir build && \
    cmake -E chdir build cmake -DPICO_SDK_PATH="../../pico-sdk" .. && \
    cmake --build build && \
    cmake --install build && \
    cp udev/99-picotool.rules /lib/udev/rules.d/

# Pre-fetch dependencies. This makes the later build much faster.
# - Add a temporary lib to fulfill dependency in Cargo.toml.
WORKDIR /usr/dependencies_fetch_project
RUN cargo init --lib app
# - Use a dummy project to fetch dependencies.
WORKDIR /usr/dependencies_fetch_project/dummy
RUN cargo init
COPY ./firmware/Cargo.toml .
RUN cargo fetch
COPY ./app/Cargo.toml .
RUN cargo fetch

RUN echo "umask 0000" >> /root/.bashrc
ENV BASH_ENV=/root/.bashrc
