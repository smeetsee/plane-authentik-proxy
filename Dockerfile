FROM rust:1.89-slim AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev libzstd-dev rsync

WORKDIR /app
COPY . .
RUN cargo build --release
RUN ldd /app/target/release/plane-authentik-proxy | grep -o '/[^ ]*' | xargs -r -I{} rsync -aL --ignore-missing-args {} /out/

FROM scratch
COPY --from=builder /out /lib
COPY --from=builder /app/target/release/plane-authentik-proxy /
EXPOSE 8080
ENTRYPOINT ["/plane-authentik-proxy"]