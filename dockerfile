FROM rustlang/rust:nightly-alpine as builder

RUN apk update && apk add --no-cache bash binaryen gcc git g++ libc-dev make npm openssl-dev protobuf-dev protoc musl-dev zlib-dev

RUN rustup target add wasm32-unknown-unknown

RUN cargo install cargo-leptos

WORKDIR /work
COPY . .

RUN cargo leptos build --release -vv

FROM scratch as runner

WORKDIR /app

COPY --from=builder /work/target/release/catenary /app/
COPY --from=builder /work/target/site /app/site

ENV LEPTOS_SITE_ROOT=./site
ENV LEPTOS_SITE_PKG_DIR=pkg
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"

CMD ["/app/catenary"]
