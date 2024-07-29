ARG TYPE

FROM rust:1.78 as builder

WORKDIR /usr/src/media-gateway
COPY . .
COPY samples/configuration/server/default_config.json /opt/etc/server.json
COPY samples/configuration/client/default_config.json /opt/etc/client.json

RUN build/install-deps.sh
RUN cargo build --release
RUN build/copy-deps.sh
RUN cargo clean

FROM debian:bookworm-slim as base
RUN \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    openssl
COPY --from=builder /opt/libs /opt/libs

ENV LD_LIBRARY_PATH=/opt/libs
ENV RUST_LOG=info

FROM base as server

COPY --from=builder /opt/bin/media_gateway_server /opt/bin/mgw-server
COPY --from=builder /opt/etc/server.json /opt/etc/config.json

ENTRYPOINT ["/opt/bin/mgw-server"]
CMD ["/opt/etc/config.json"]

FROM base as client

COPY --from=builder /opt/bin/media_gateway_client /opt/bin/mgw-client
COPY --from=builder /opt/etc/client.json /opt/etc/config.json

ENTRYPOINT ["/opt/bin/mgw-client"]
CMD ["/opt/etc/config.json"]

FROM ${TYPE} as final
