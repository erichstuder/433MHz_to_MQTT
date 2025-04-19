FROM rust:1.83.0

RUN apt-get update && apt-get install -y \
    libudev-dev

RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/download/v0.25.0/probe-rs-tools-installer.sh | sh

ARG USER
ARG UID
RUN useradd -m -s /bin/bash -u ${UID:-2222} $USER
USER ${USER}

RUN rustup target add thumbv6m-none-eabi

RUN cargo install \
    elf2uf2-rs@2.1.1 \
    flip-link@0.1.9

# Pre-fetch dependencies. This makes the later build much faster.
# - Add a temporary lib to fulfill dependency in Cargo.toml.
WORKDIR /home/$USER/dependencies_fetch_project
# - Use a dummy project to fetch dependencies.
WORKDIR /home/$USER/dependencies_fetch_project/dummy
RUN cargo init
COPY ./firmware/Cargo.toml .
RUN cargo fetch
# - For example when used as devcontainer, the UID is set to a default value (see above).
#   I wasn't able to pass the UID of the local user to the container in this case.
#   So when using the devcontainer, the local user is then used and can't access the fetched dependencies.
#   To solve this, the fetched dependencies are made readable and writable by anyone.
RUN chmod -R a+rw /usr/local/cargo/registry
