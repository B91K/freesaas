# Build from the repo root:
#   docker build -f deploy/docker-compose/Dockerfile -t freesaas .
FROM rust:1.97 AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY server server
RUN cargo build --release --bin freesaas

FROM debian:bookworm-slim
RUN useradd --system --create-home freesaas
COPY --from=build /src/target/release/freesaas /usr/local/bin/freesaas
COPY web /srv/web
ENV FREESAAS_WEB_ROOT=/srv/web FREESAAS_CONFIG=/etc/freesaas/freesaas.yaml
USER freesaas
EXPOSE 8080
ENTRYPOINT ["freesaas"]
