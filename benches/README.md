# Benchmarking

Benchmark tests use Docker images built locally from sources.

## Server

To build Docker image for the server

```bash
docker build --build-arg="TYPE=server" -t media-gateway-server:latest .
```

## Client

To build Docker image for the client

```bash
docker build --build-arg="TYPE=client" -t media-gateway-client:latest .
```

