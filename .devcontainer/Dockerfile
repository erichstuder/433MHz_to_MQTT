FROM mcr.microsoft.com/devcontainers/python:3.0.8-3.14-trixie

## common ##
RUN apt-get update && apt-get install -y \
    libudev-dev \
    wget \
    # tig is great for viewing git history
    tig \
    # less is needed e.g. for 'git diff' to make the output scrollable
    less


## openocd ##
RUN apt-get update && apt-get install -y \
    libusb-1.0-0-dev \
    libhidapi-dev \
    libjim-dev \
    libjaylink-dev

RUN git clone --branch sdk-2.3.0 --depth 1 https://github.com/raspberrypi/openocd.git && \
    cd openocd && \
    git submodule update --init --recursive && \
    ./bootstrap && \
    ./configure --enable-jlink --enable-cmsis-dap --enable-hidapi --enable-libusb && \
    make -j$(nproc) && \
    make install

RUN apt-get update && apt-get install -y \
    gdb-multiarch \
    netcat-openbsd


USER vscode


## rust ##
ENV CARGO_HOME=/home/vscode/.cargo
ENV RUSTUP_HOME=/home/vscode/.rustup
ENV PATH=/home/vscode/.local/bin:/home/vscode/.cargo/bin:${PATH}
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs  | sh -s -- -y --default-toolchain 1.96.0


## software ##
RUN rustup target add thumbv6m-none-eabi

RUN cargo install --locked \
    elf2uf2-rs@2.2.0 \
    flip-link@0.1.12 \
    cargo-edit@0.13.11 \
    probe-rs-tools@0.31.0 \
    # defmt-print is for printing defmt logs when debugging with openocd and gdb
    defmt-print@1.1.0 \
    embedded-test-std-runner@0.1.1

# Pre-fetch dependencies. This makes the later build much faster.
WORKDIR /tmp/firmware_dependencies_fetch
RUN cargo init
COPY software/firmware/Cargo.toml .
RUN cargo fetch


## features ##
# Pre-fetch dependencies. This makes the later build much faster.
WORKDIR /tmp/features_dependencies_fetch
RUN cargo init
COPY features/steps/Cargo.toml .
RUN cargo fetch


## doc ##
USER root

RUN apt-get update && apt-get install -y \
    # libasound2 \
    plantuml \
    xvfb \
    tree
    # locales

WORKDIR /tmp/drawio
RUN wget https://github.com/jgraph/drawio-desktop/releases/download/v30.0.4/drawio-amd64-30.0.4.deb && \
    apt-get install -y ./drawio-amd64-30.0.4.deb

USER vscode

RUN cargo install --locked cargo-modules@0.26.0

RUN python3 -m pip install --verbose \
    # An older version of sphinx is needed to solve a dependency conflicts, see below.
    sphinx==8.2.3 \
    \
    sphinx-autobuild==2025.8.25 \
    sphinxcontrib-drawio==0.0.17 \
    sphinxcontrib-plantuml==0.31 \
    sphinx-toolbox==4.2.0 \
    sphinxcontrib-programoutput==0.19 \
    pyserial==3.5 \
    \
    # depends on sphinx < 9
    sphinxcontrib-rust==1.1.0 \
    \
    sphinx-needs==8.1.1 \
    sphinx-gherkindoc==3.6.4 \
    setuptools==82.0.1 \
    pillow==12.2.0 \
    ansi2html==1.9.2

# RUN sed -i '/en_US.UTF-8/s/^# //g' /etc/locale.gen && \
#     locale-gen
