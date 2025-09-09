FROM rust:1.89-slim as builder
RUN apt-get update && apt-get install -y libssl-dev

ENV OPENSSL_STATIC=1 \
    OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu \
    OPENSSL_INCLUDE_DIR=/usr/include

WORKDIR /app
COPY . .
RUN cargo build --release

FROM scratch
COPY --from=builder /app/target/release/authentik-gitlab-proxy /
EXPOSE 8080
ENTRYPOINT ["/authentik-gitlab-proxy"]