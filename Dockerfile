FROM rust:1.78 as builder
ARG TYPE

WORKDIR /usr/src/media-gateway
COPY . .
COPY "samples/${TYPE}/default_config.json" /opt/etc/config.json

RUN build/install-deps.sh
RUN cargo build --release
RUN build/copy-deps.sh ${TYPE}
RUN cargo clean

FROM debian:bookworm-slim
ARG TYPE
RUN \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    openssl
COPY --from=builder /opt/libs /opt/libs
COPY --from=builder /opt/bin/ /opt/bin/
COPY --from=builder /opt/etc/ /opt/etc/
RUN ln -s "/opt/bin/media_gateway_${TYPE}" /opt/bin/media_gateway_app

ENV LD_LIBRARY_PATH=/opt/libs
ENV RUST_LOG=info

ENTRYPOINT ["/opt/bin/media_gateway_app"]
CMD ["/opt/etc/config.json"]
