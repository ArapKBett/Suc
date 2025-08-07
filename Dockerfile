FROM rust:1.80 AS builder
WORKDIR /usr/src/solana-usdc-indexer
COPY . .
RUN cargo build --release

FROM rust:1.80-slim
WORKDIR /usr/src/solana-usdc-indexer
COPY --from=builder /usr/src/solana-usdc-indexer/target/release/solana-usdc-indexer .
EXPOSE 8080
CMD ["./solana-usdc-indexer"]
