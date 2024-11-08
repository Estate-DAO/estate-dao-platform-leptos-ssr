
FROM scratch

WORKDIR /app

COPY ./target/x86_64-unknown-linux-musl/release/estate-fe .
COPY ./city.json .
COPY ./hash.txt .
COPY ./target/site ./site

ENV LEPTOS_SITE_ROOT=site
# ENV LEPTOS_SITE_PKG_DIR=pkg

ENV LEPTOS_ENV="production"
ENV LEPTOS_HASH_FILES="true"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV RUST_LOG="info"
EXPOSE 3000

CMD ["./estate-fe"]
