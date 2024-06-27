FROM rust:1.78 as builder

WORKDIR /usr/src/media-gateway
COPY . .
COPY samples/server/default_config.json /opt/etc/config.json

RUN build/install-deps.sh
RUN cargo build --release
RUN build/copy-deps.sh
RUN cargo clean

FROM debian:bookworm-slim
RUN \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    openssl
COPY --from=builder /opt/libs /opt/libs
COPY --from=builder /opt/bin/ /opt/bin/
COPY --from=builder /opt/etc/ /opt/etc/

ENV LD_LIBRARY_PATH=/opt/libs
ENV RUST_LOG=info

ENTRYPOINT ["/opt/bin/media_gateway_server"]
CMD ["/opt/etc/config.json"]
