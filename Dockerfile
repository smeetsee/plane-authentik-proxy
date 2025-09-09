FROM rust:1.89-alpine AS builder
RUN apk add --no-cache pkgconfig openssl-dev
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:latest AS copylibs
RUN apk add --no-cache rsync
WORKDIR /out
COPY --from=builder /app/target/release/authentik-gitlab-proxy /out/
RUN ldd /out/authentik-gitlab-proxy | awk '{print $3}' | grep -v '^$' | xargs -I '{}' rsync -R '{}' /out/

FROM scratch
COPY --from=copylibs /out/ /
EXPOSE 8080
ENTRYPOINT ["/authentik-gitlab-proxy"]