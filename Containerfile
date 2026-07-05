FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ src/

RUN cargo build --release

FROM alpine:latest

WORKDIR /app

COPY --from=builder /app/target/release/clarity-dashboard /app/clarity-dashboard

EXPOSE 3000

CMD ["/app/clarity-dashboard"]
