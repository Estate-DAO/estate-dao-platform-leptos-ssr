
FROM scratch

WORKDIR /app

COPY ./target/x86_64-unknown-linux-musl/release/estate-fe .
COPY ./hash.txt .
COPY ./city.json .

COPY ./target/site ./site
ENV LEPTOS_SITE_ROOT="site"

ENV LEPTOS_ENV="production"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_HASH_FILES="true"
EXPOSE 3000

CMD ["./estate-fe"]
