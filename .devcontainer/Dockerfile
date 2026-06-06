FROM mcr.microsoft.com/devcontainers/python:3.0.8-3.13-trixie

## common setup ##
RUN apt-get update && apt-get install -y \
    libudev-dev \
    wget \
    # tig is great for viewing git history
    tig \
    # less is needed e.g. for 'git diff' to make the output scrollable
    less

USER vscode

## rust setup ##
ENV CARGO_HOME=/home/vscode/.cargo
ENV RUSTUP_HOME=/home/vscode/.rustup
ENV PATH=/home/vscode/.local/bin:/home/vscode/.cargo/bin:${PATH}
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs  | sh -s -- -y --default-toolchain 1.85.0


## software setup ##
RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/download/v0.25.0/probe-rs-tools-installer.sh | sh

RUN rustup target add thumbv6m-none-eabi

RUN cargo install --locked \
    elf2uf2-rs@2.1.1 \
    flip-link@0.1.9 \
    cargo-edit@0.13.3

# Pre-fetch dependencies. This makes the later build much faster.
WORKDIR /tmp/firmware_dependencies_fetch
RUN cargo init
COPY software/firmware/Cargo.toml .
RUN cargo fetch


## features setup ##
# Pre-fetch dependencies. This makes the later build much faster.
WORKDIR /tmp/features_dependencies_fetch
RUN cargo init
COPY features/steps/Cargo.toml .
RUN cargo fetch


## doc setup ##
USER root
RUN apt-get update && apt-get install -y \
    # libasound2 \
    plantuml \
    xvfb \
    tree
    # locales

WORKDIR /tmp/drawio
RUN wget https://github.com/jgraph/drawio-desktop/releases/download/v25.0.2/drawio-amd64-25.0.2.deb && \
    apt-get install -y ./drawio-amd64-25.0.2.deb

USER vscode

RUN cargo install --locked cargo-modules@0.23.1

RUN python3 -m pip install --verbose \
    sphinx==8.1.3 \
    sphinx-autobuild==2024.10.3 \
    sphinxcontrib-drawio==0.0.17 \
    sphinxcontrib-plantuml==0.30 \
    sphinx-toolbox==3.8.1 \
    sphinxcontrib-programoutput==0.18 \
    pyserial==3.5 \
    sphinxcontrib-rust==0.8.1 \
    sphinx-needs==4.1.0 \
    sphinx-gherkindoc==3.6.4 \
    setuptools==75.6.0 \
    pillow==11.0.0

# RUN sed -i '/en_US.UTF-8/s/^# //g' /etc/locale.gen && \
#     locale-gen
