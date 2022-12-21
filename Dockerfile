FROM rust:1.65-slim AS builder

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

#to enable cargo build before adding sources
RUN mkdir src
RUN touch src/main.rs
RUN echo "fn main() {}" > src/main.rs

COPY Cargo.toml .
RUN cargo build --target x86_64-unknown-linux-musl --release

RUN rm -r src
RUN rm ./target/x86_64-unknown-linux-musl/release/deps/scraping_api_backend*
COPY src src
RUN cargo build --target x86_64-unknown-linux-musl --release



FROM alpine

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/scraping-api-backend ./


EXPOSE 3000

ENTRYPOINT ["./scraping-api-backend"]
