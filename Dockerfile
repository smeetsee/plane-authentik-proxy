FROM rust:1.89-slim AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev libzstd-dev

WORKDIR /app
COPY . .
RUN cargo build --release

FROM scratch
COPY --from=builder /app/target/release/plane-authentik-proxy /
EXPOSE 8080
ENTRYPOINT ["/plane-authentik-proxy"]