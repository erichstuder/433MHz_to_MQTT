FROM rust:latest

RUN apt-get update && \
    apt-get -y install libudev-dev

# Pre-fetch dependencies with a dummy project. This makes the later build much faster.
WORKDIR /usr/dependencies_fetch_project
RUN cargo init
COPY ./steps/Cargo.toml .
RUN cargo fetch

RUN echo "umask 0000" >> /root/.bashrc
ENV BASH_ENV=/root/.bashrc
