FROM rust:1.89-slim AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev libzstd-dev rsync

WORKDIR /app
COPY . .
RUN cargo build --release
RUN ldd /app/target/release/plane-authentik-proxy | awk '{print $3}' | grep -v '^$' | xargs -I '{}' rsync -R '{}' /out/

FROM scratch
COPY --from=builder /out /
COPY --from=builder /app/target/release/plane-authentik-proxy /
EXPOSE 8080
ENTRYPOINT ["/plane-authentik-proxy"]