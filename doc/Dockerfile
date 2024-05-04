FROM sphinxdoc/sphinx:latest

RUN apt update && \
    apt install -y python3-sphinxcontrib.plantuml graphviz python3-sphinx-autobuild wget

RUN pip3 install sphinxcontrib-plantuml

WORKDIR /usr/plantuml
RUN wget https://github.com/plantuml/plantuml/releases/download/v1.2024.4/plantuml-gplv2-1.2024.4.jar

WORKDIR /docs
