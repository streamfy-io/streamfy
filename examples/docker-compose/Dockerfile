FROM ubuntu:20.04

RUN apt-get update
RUN apt-get install -y curl unzip
RUN curl -fsS https://raw.githubusercontent.com/streamfy-io/streamfy/master/install.sh | bash

ENV PATH "$PATH:/root/.streamfy/bin"
ENV PATH "$PATH:/root/.svm/bin"
