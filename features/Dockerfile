FROM rust:1.83.0

RUN apt-get update && apt-get install -y \
    libudev-dev=252.31-1~deb12u1

# Pre-fetch dependencies with a dummy project. This makes the later build much faster.
WORKDIR /usr/dependencies_fetch_project
RUN cargo init
COPY ./steps/Cargo.toml .
RUN cargo fetch

RUN echo "umask 0000" >> /root/.bashrc
ENV BASH_ENV=/root/.bashrc
