# Demo instance image — solo SQLite store, fixture corpus only.
# NOT the root Dockerfile (that one builds the Postgres hub variant).

FROM rust:bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release --locked --bin kurultai

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/kurultai /usr/local/bin/kurultai
COPY demo/ /app/demo/
RUN chmod +x /app/demo/entrypoint.sh /app/demo/seed.sh
WORKDIR /app
ENV HOME=/tmp
EXPOSE 8421
USER nobody
ENTRYPOINT ["/app/demo/entrypoint.sh"]
