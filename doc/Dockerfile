FROM python:3.13.1
# Note: Can't use official sphinx image because drawio can't be installed from apt which causes problems.

WORKDIR /usr/drawio
RUN apt update && apt install -y \
    wget=1.21.3-1+b2 \
    libasound2=1.2.8-1+b1 \
    plantuml=1:1.2020.2+ds-3

RUN wget https://github.com/jgraph/drawio-desktop/releases/download/v25.0.2/drawio-amd64-25.0.2.deb && \
    apt install -y ./drawio-amd64-25.0.2.deb

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs  | sh -s -- -y && \
    . $HOME/.cargo/env && \
    rustup install 1.83.0 && \
    rustup default 1.83.0 && \
    cargo install sphinx-rustdocgen --version 0.3.0

RUN apt update && apt install -y \
    xvfb=2:21.1.7-3+deb12u8 \
    tree=2.1.0-1

# Note: sphinxcontrib-rust==0.6.1 works with maximum sphinx==7.4.7
RUN python3 -m pip install --verbose \
    sphinx==7.4.7 \
    sphinx-autobuild==2024.10.3 \
    sphinxcontrib-drawio==0.0.17 \
    sphinxcontrib-plantuml==0.30 \
    sphinx-toolbox==3.8.1 \
    sphinxcontrib-programoutput==0.18 \
    pyserial==3.5 \
    sphinxcontrib-rust==0.3.0 \
    sphinx-needs==4.1.0 \
    sphinx-gherkindoc==3.6.4 \
    setuptools==75.6.0 \
    pillow==11.0.0

RUN echo "umask 0000" >> /root/.bashrc
ENV BASH_ENV=/root/.bashrc
