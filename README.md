# MediaGateway

Provides a WS/HTTPS gateway securing edge-core communication

## Docker

Both server and client can be run as Docker containers.

To build Docker image for the server

```bash
docker build --build-arg="TYPE=server" -t media-gateway-server:latest .
```

To build Docker image for the client

```bash
docker build --build-arg="TYPE=client" -t media-gateway-client:latest .
```
