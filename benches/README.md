# Benchmarking

There are following benchmark tests:

* without TLS based
  on [video loop source adapter](https://docs.savant-ai.io/develop/savant_101/10_adapters.html#video-loop-source-adapter) (`video-loop.yaml`
  Docker Compose configuration file)
* with TLS based
  on [multistream source adapter](https://docs.savant-ai.io/develop/savant_101/10_adapters.html#multi-stream-source-adapter) (`multi-stream-source.yaml`
  Docker Compose configuration file)
* with TLS via [nginx](https://nginx.org/) based
  on [multistream source adapter](https://docs.savant-ai.io/develop/savant_101/10_adapters.html#multi-stream-source-adapter) (`multi-stream-source-nginx.yaml`
  Docker Compose configuration file)

Benchmark tests use Docker images built locally from sources and certificates signed by a private CA.

## Building Docker images

### Server

To build Docker image for the server

```bash
docker build --build-arg="TYPE=server" -t media-gateway-server:latest ..
```

### Client

To build Docker image for the client

```bash
docker build --build-arg="TYPE=client" -t media-gateway-client:latest ..
```

## Generating certificates

To generate certificates

```bash
    bash generate_certs.sh
```

## Running benchmark tests

```bash
    docker compose -f <file> up -d
```

where `<file>` is a Docker Compose configuration file for the chosen benchmark test.

## Stopping benchmark tests

```bash
    docker compose -f <file> down
```

where `<file>` is a Docker Compose configuration file for the chosen benchmark test.

## Cleaning up

```bash
    rm -rf ca
```
