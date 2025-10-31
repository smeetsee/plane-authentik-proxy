FROM rust:1.91-slim AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev libzstd-dev rsync

WORKDIR /app
COPY . .
RUN cargo build --release
RUN ldd /app/target/release/plane-authentik-proxy | grep -o '/[^ ]*' | xargs -r -I{} sh -c 'mkdir -p /out$(dirname {}) && rsync -aL --ignore-missing-args {} /out{}'

FROM scratch
COPY --from=rust:1.90-slim /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /out/ /
COPY --from=builder /app/target/release/plane-authentik-proxy /
EXPOSE 8080
ENTRYPOINT ["/plane-authentik-proxy"]
