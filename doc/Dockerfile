FROM sphinxdoc/sphinx:latest

RUN apt update && \
    apt install -y python3-sphinx-autobuild wget

WORKDIR /usr/drawio
RUN wget https://github.com/jgraph/drawio-desktop/releases/download/v24.2.5/drawio-amd64-24.2.5.deb && \
	apt install -y ./drawio-amd64-24.2.5.deb && \
	apt update && apt install -y libasound2

RUN python3 -m pip install sphinxcontrib-drawio && \
	apt update && apt install -y xvfb

WORKDIR /docs
