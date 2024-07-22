# Media Gateway

The media gateway provides a functionality to forward messages from one [ZeroMQ](https://zeromq.org/) instance to
another. The media gateway consists of two applications - a server and client. The client reads messages from the source
ZeroMQ instance and sends them to the server via HTTP/HTTPS. The server writes received messages to the target ZeroMQ
instance.

Following optional features are supported:

* basic authentication
* HTTPS (including a self-signed PEM encoded certificate)
* client certificate authentication (for the
  server [X509_LOOKUP_hash_dir method](https://www.openssl.org/docs/man1.1.1/man3/X509_LOOKUP_hash_dir.html) is used to
  load certificates and CRLs)
* FPS statistics logging (by frame or timestamp period)

To read from and to write to ZeroMQ [savant_core](https://github.com/insight-platform/savant-rs) crate is used.

Both server and client applications are configured via JSON files. `in_stream` in the client configuration corresponds
to [ReaderConfig](https://github.com/insight-platform/savant-rs/blob/main/savant_core/src/transport/zeromq/reader_config.rs)
and `out_stream` in the server configuration corresponds
to [WriterConfig](https://github.com/insight-platform/savant-rs/blob/main/savant_core/src/transport/zeromq/writer_config.rs).
Examples of configuration files can be found in [samples](samples) directory.

The server has a health endpoint allowing monitoring its status.

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

### Running Server

To run the server with [the default configuration](samples/configuration/server/default_config.json) and to mount `/tmp`
directory and publish the port from the default configuration:

```bash
docker run \
  -v /tmp:/tmp \
  -p 8080:8080 \
  ghcr.io/insight-platform/media-gateway-server:latest
```

To run the server with another configuration (`/home/user/server_config.json`):

```bash
docker run \
  -v /home/user/server_config.json:/opt/etc/custom_config.json \
  -p HOST_PORT:CONFIG_PORT \
  ghcr.io/insight-platform/media-gateway-server:latest \
  /opt/etc/custom_config.json
```

### Running Client

To run the client with [the default configuration](samples/configuration/client/default_config.json) and to mount `/tmp`
directory:

```bash
docker run \
  -v /tmp:/tmp \
  -e "GATEWAY_URL=<GATEWAY_URL>" \
  ghcr.io/insight-platform/media-gateway-client:latest
```

where `<GATEWAY_URL>` is the server URL, e.g. `http://192.168.0.100:8080`

To run the server with a custom configuration file (`/home/user/client_config.json`):

```bash
docker run \
  -v /home/user/client_config.json:/opt/etc/custom_config.json \
  ghcr.io/insight-platform/media-gateway-client:latest \
  /opt/etc/custom_config.json
```
