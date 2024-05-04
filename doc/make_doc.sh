#!/bin/bash

docker build --tag my_container .

#docker run --rm -v .:/docs my_container make html
#docker run --rm -v .:/docs -p 8000:8000 -p 35729:35729 my_container sphinx-autobuild --port 8000 --host 0.0.0.0 --watch /docs source _build/html
docker run --rm -v .:/docs -p 8000:8000 -p 35729:35729 my_container sphinx-autobuild --port 8000 --host 0.0.0.0 source _build/html
