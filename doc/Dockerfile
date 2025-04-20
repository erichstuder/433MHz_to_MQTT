FROM python:3.13.1
# Note: Can't use official sphinx image because drawio can't be installed from apt which causes problems.

RUN apt-get update && apt-get install -y \
    wget \
    libasound2 \
    plantuml \
    xvfb \
    tree \
    locales

WORKDIR /usr/drawio
RUN wget https://github.com/jgraph/drawio-desktop/releases/download/v25.0.2/drawio-amd64-25.0.2.deb && \
    apt-get install -y ./drawio-amd64-25.0.2.deb

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs  | sh -s -- -y && \
    . $HOME/.cargo/env && \
    rustup install 1.85.0 && \
    rustup default 1.85.0 && \
    cargo install cargo-modules@0.23.1

RUN . $HOME/.cargo/env  && python3 -m pip install --verbose \
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

RUN sed -i '/en_US.UTF-8/s/^# //g' /etc/locale.gen && \
    locale-gen
