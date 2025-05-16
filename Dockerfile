FROM docker.io/library/rust:1.87-alpine as builder

WORKDIR /usr/src/searched
COPY . .

RUN apk add --no-cache -U musl-dev openssl-dev
RUN cargo build --release --no-default-features --features prod

FROM docker.io/library/alpine:latest

COPY --from=builder /usr/src/searched/target/release/searched /usr/local/bin/searched/searched
COPY --from=builder /usr/src/searched/views /usr/local/bin/searched/views

WORKDIR /usr/local/bin/searched
CMD ["./searched"]
