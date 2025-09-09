FROM rust:1.89-slim AS builder
RUN apt-get update && apt-get install -y libssl-dev libzstd-dev

WORKDIR /app
COPY . .
RUN cargo build --release

FROM scratch
COPY --from=builder /app/target/release/authentik-gitlab-proxy /
EXPOSE 8080
ENTRYPOINT ["/authentik-gitlab-proxy"]