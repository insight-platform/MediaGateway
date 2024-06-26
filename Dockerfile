FROM rust:1.78 as builder

WORKDIR /usr/src/media-gateway
COPY . .

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

ENV LD_LIBRARY_PATH=/opt/libs
ENV RUST_LOG=info

CMD ["/opt/bin/media_gateway_server", "/opt/media-gateway-server/config.json"]
