FROM python:latest
# Note: Can't use official sphinx image because drawio can't be installed from apt which causes problems.

WORKDIR /usr/drawio
RUN apt update && apt install -y wget libasound2 && \
	wget https://github.com/jgraph/drawio-desktop/releases/download/v24.2.5/drawio-amd64-24.2.5.deb && \
	apt install -y ./drawio-amd64-24.2.5.deb

RUN python3 -m pip install sphinx sphinx-autobuild sphinxcontrib-drawio && \
	apt update && apt install -y xvfb

WORKDIR /docs
