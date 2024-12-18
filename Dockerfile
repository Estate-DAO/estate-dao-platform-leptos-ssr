

# FROM debian:bullseye-slim as runner
FROM scratch

WORKDIR /app

COPY ./target/x86_64-unknown-linux-musl/release/estate-fe .
COPY ./city.json .
COPY ./ssr/target/x86_64-unknown-linux-musl/release/hash.txt ./release/hash.txt
COPY ./ssr/target/x86_64-unknown-linux-musl/release/hash.txt ./hash.txt
COPY ./target/site ./site

ENV LEPTOS_SITE_ROOT=site

ENV LEPTOS_ENV="production"
ENV LEPTOS_HASH_FILES="true"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV RUST_LOG="info"
# these will be passed by fly vm
# ENV PROVAB_HEADERS
# ENV LOCAL
EXPOSE 3000

CMD ["./estate-fe"]


# # Get started with a build env with Rust nightly
# FROM rustlang/rust:nightly-bullseye-slim as builder

# # Install cargo-binstall, which makes it easier to install other
# # cargo extensions like cargo-leptos
# RUN wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz
# RUN tar -xvf cargo-binstall-x86_64-unknown-linux-musl.tgz
# RUN cp cargo-binstall /usr/local/cargo/bin

# # Install cargo-leptos
# RUN cargo binstall cargo-leptos -y

# # Add the WASM target
# RUN rustup target add wasm32-unknown-unknown

# # Make an /app dir, which everything will eventually live in
# RUN mkdir -p /app
# WORKDIR /app
# COPY . .

# # Build the app
# RUN cargo leptos build --release -vv

# FROM debian:bullseye-slim as runner

# # -------------- NB: update binary name to match app name in Cargo.toml ------------------
# # Copy the server binary to the /app directory
# COPY --from=builder /app/target/release/fly-io-ssr-test-deploy /app/

# # /target/site contains our JS/WASM/CSS, etc.
# COPY --from=builder /app/target/site /app/site

# # Copy Cargo.toml if it’s needed at runtime
# COPY --from=builder /app/Cargo.toml /app/
# WORKDIR /app

# # Set any required env variables
# ENV RUST_LOG="info"
# ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
# ENV LEPTOS_SITE_ROOT="site"
# EXPOSE 8080

# # -------------- NB: update binary name to match app name in Cargo.toml ------------------
# # Run the server
# CMD ["/app/fly-io-ssr-test-deploy"]