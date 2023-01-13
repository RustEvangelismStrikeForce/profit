# use builder image with rust toolchain
FROM rust:1.66.1-slim as builder

COPY . .

RUN cargo build --release


# assemble final image
FROM debian:buster-slim

COPY --from=builder target/release/profit_cli /

CMD ["/profit_cli"]
