FROM rust:1.80 AS builder
WORKDIR /usr/src/solana-usdc-indexer
COPY . .
# Clear Cargo cache and update lockfile
RUN rm -rf /usr/local/cargo/registry
RUN cargo update
RUN cargo build --release

FROM rust:1.80-slim
WORKDIR /usr/src/solana-usdc-indexer
COPY --from=builder /usr/src/solana-usdc-indexer/target/release/solana-usdc-indexer .
EXPOSE 8080
CMD ["./solana-usdc-indexer"]
