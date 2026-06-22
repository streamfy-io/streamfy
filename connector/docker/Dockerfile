FROM debian:stable-slim

RUN apt-get update && apt-get -y upgrade
RUN apt-get install -y tini
RUN apt-get install -y curl

# setup streamfy as non user
ENV USER=streamfy
RUN adduser \
    --disabled-password \
    --home "/home/$USER" \
    "$USER"
USER $USER
WORKDIR /home/streamfy


# install streamfy
RUN curl -fsS https://raw.githubusercontent.com/streamfy-io/streamfy/master/install.sh | bash
RUN ls -la /home/streamfy/.streamfy/bin
ENV PATH="/home/streamfy/.streamfy/bin:$PATH"
RUN export PATH
RUN streamfy version

# copy streamfy profile and test
COPY ./streamfy_profile.toml /home/streamfy/.streamfy/config
# RUN streamfy topic list

WORKDIR /home/streamfy/connector


# Download specific connector, you can configure this to download the connector you need
RUN cdk hub download streamfy/http-source@0.3.8
RUN tar xf streamfy-http-source-0.3.8.ipkg
RUN tar -xzf manifest.tar.gz
RUN ls -la


# Copy connector configuration
COPY ./connector.yaml /home/streamfy/connector
RUN ls -la /home/streamfy/connector

# run http-source, this will be different for each connector
ENTRYPOINT ["/home/streamfy/connector/http-source",  "--config", "/home/streamfy/connector/connector.yaml"]
