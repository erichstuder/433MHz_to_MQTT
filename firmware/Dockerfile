FROM rust:latest

ARG USER
ARG USER_ID
ARG GROUP_ID

RUN addgroup --gid ${GROUP_ID} ${USER} && \
    adduser --disabled-password --gecos '' --uid ${USER_ID} --gid ${GROUP_ID} ${USER}

# WORKDIR /usr/src/firmware
