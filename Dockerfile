# Hub image: Postgres store only. Do not mount store.db.
# See docs/deploy/railway-hub.md

FROM rust:bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release --locked --features postgres --bin kurultai

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/kurultai /usr/local/bin/kurultai
ENV KURULTAI_FEATURE_HUB=1
ENV KURULTAI_HUB_BIND=all
ENV KURULTAI_HUB_AUTH=api_key
EXPOSE 8421
USER nobody
ENTRYPOINT ["kurultai"]
CMD ["daemon", "--no-poll", "--no-watch"]
