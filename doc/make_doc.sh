#!/bin/bash

docker build --tag my_container .
docker run --rm -v .:/docs my_container make html
