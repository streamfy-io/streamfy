# Docker Compose + Streamfy

You can run Streamfy Clusters in a Docker Compose setup, this could be useful for
local development and POC development.

In order to run a Streamfy Cluster through Docker, you will need to run Streamfy
components separately, we can use Docker Compose `service`s to achieve this.

## Services

- `sc: Streaming Controller`
- `sc-setup: Post-Initialization Commands`
- `spu: Streaming Processing Unit`

> To learn more about Streamfy architecture, please refer to [Streamfy Documentation][1]

## Running Locally

Clone this repo using `git clone https://github.com/streamfy-io/streamfy.git` and
cd into `./streamfy/examples/docker-compose`, then run `docker compose up`.

> Optionally you can run on detached mode `docker compose up -d` so
> Streamfy runs in the background.

Then use the `streamfy` CLI to connect to the cluster running in Docker, to do
that you must set the _Streamfy Profile_ to point to Docker's container SC:

> If you dont have the Streamfy CLI installed, run the following command
> `curl -fsS https://raw.githubusercontent.com/streamfy-io/streamfy/master/install.sh | bash`.
> Refer to [Streamfy CLI Reference][2] for more details.

```bash
streamfy profile add docker 127.0.0.1:9103 docker
```

> Streamfy Streaming Controller (SC) usually runs on port `9003` but given that our
> SC is running in a Docker Container, internal port `9003` is mapped to `9103`
> in your system's network.

With the profile set, you are now able to perform Streamfy Client operations
like listing topics:

```bash
streamfy topic list
```

## Teardown

In order to shutdown the Streamfy Cluster running in Docker, you must issue the
following `compose` command:

```bash
docker compose down
```

> Remember to run this command in the same directory as the `docker-compose.yml`
> file.

[1]: https://www.streamfy.io/docs/streamfy/concepts/architecture/overview/
[2]: https://www.streamfy.io/docs/streamfy/cli/overview
