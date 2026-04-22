FROM debian:bookworm-slim AS runner

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY target/release/estate-fe .
RUN chmod +x ./estate-fe
COPY target/release/hash.txt .
COPY target/site ./site
COPY city.json ./city.json
COPY city.parquet ./city.parquet
# ip_db.mmdb is pre-decompressed by CI before docker build
COPY ip_db.mmdb ./ip_db.mmdb

ENV LEPTOS_ENV="production"
ENV RUST_LOG="debug,hyper=info,tower=info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_HASH_FILES="true"

EXPOSE 3000

CMD ["./estate-fe"]
