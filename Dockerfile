FROM rust:1-bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /build/target/release/pivx-walletd /pivx-walletd
ENV PIVX_WALLETD_BIND=0.0.0.0:8333
EXPOSE 8333
ENTRYPOINT ["/pivx-walletd"]
