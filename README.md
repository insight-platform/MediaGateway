# Media Gateway

The media gateway provides a functionality to forward messages from one [ZeroMQ](https://zeromq.org/) instance to
another. The media gateway consists of two applications - a server and client. The client reads messages from the source
ZeroMQ instance and sends them to the server via HTTP/HTTPS. The server writes received messages to the target ZeroMQ
instance.

Following optional features are supported:

* basic authentication
* HTTPS (including a self-signed PEM encoded certificate)
* client certificate authentication

To read from and to write to ZeroMQ [savant_core](https://github.com/insight-platform/savant-rs) crate is used.

Both server and client applications are configured via JSON files. `in_stream` in the client configuration corresponds
to [ReaderConfig](https://github.com/insight-platform/savant-rs/blob/main/savant_core/src/transport/zeromq/reader_config.rs)
and `out_stream` in the server configuration corresponds to
[WriterConfig](https://github.com/insight-platform/savant-rs/blob/main/savant_core/src/transport/zeromq/writer_config.rs).
Examples of configuration files can be found in [samples](samples) directory.

The server has a health endpoint.

```
 GET /health HTTP/1.1
 Host: <host>
 ```

If the server is healthy an HTTP response with 200 OK status code and the body as below will be returned.

 ```json
 {
  "status": "healthy"
}
 ```

## Docker

Both server and client can be run as Docker containers.

### Server

To build Docker image for the server

```bash
docker build --build-arg="TYPE=server" -t media-gateway-server:latest .
```

To run the server with [the default configuration](samples/server/default_config.json) and to mount `/tmp` directory and
publish the port from the default configuration

```bash
docker run \
 -v /tmp:/tmp \
 -p 8080:8080 \
 media-gateway-server:latest
```

To run the server with another configuration (`/home/user/server_config.json`)

```bash
docker run \
 -v /home/user/server_config.json:/opt/etc/custom_config.json \
 -p HOST_PORT:CONFIG_PORT \
 media-gateway-server:latest \
 /opt/etc/custom_config.json
```

### Client

To build Docker image for the client

```bash
docker build --build-arg="TYPE=client" -t media-gateway-client:latest .
```

To run the client with [the default configuration](samples/client/default_config.json) and to mount `/tmp` directory

```bash
docker run \
 -v /tmp:/tmp \
  -e "GATEWAY_URL=<GATEWAY_URL>" \
 media-gateway-client:latest
```

where `<GATEWAY_URL>` is the server URL, e.g. `http://192.168.0.100:8080`

To run the server with another configuration (`/home/user/client_config.json`)

```bash
docker run \
 -v /home/user/client_config.json:/opt/etc/custom_config.json \
 media-gateway-client:latest \
 /opt/etc/custom_config.json
```
