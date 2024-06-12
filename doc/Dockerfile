FROM python:latest
# Note: Can't use official sphinx image because drawio can't be installed from apt which causes problems.

WORKDIR /usr/drawio
RUN apt update && apt install -y wget libasound2 plantuml && \
    wget https://github.com/jgraph/drawio-desktop/releases/download/v24.2.5/drawio-amd64-24.2.5.deb && \
    apt install -y ./drawio-amd64-24.2.5.deb

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    . $HOME/.cargo/env && \
    cargo install sphinx-rustdocgen

RUN apt update && apt install -y xvfb tree
RUN python3 -m pip install sphinx sphinx-autobuild sphinxcontrib-drawio sphinxcontrib-plantuml \
                           sphinx-toolbox sphinxcontrib-programoutput pyserial

# install pre-relase of sphinxcontrib-rust
RUN . $HOME/.cargo/env && \
    python3 -m pip install git+https://git@gitlab.com/munir0b0t/sphinxcontrib-rust@main
