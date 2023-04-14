FROM rust:1.67-bullseye AS builder
ADD . /build
WORKDIR /build
RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /app
COPY --from=builder /build/target/release/ambient_proxy_manager ./
CMD [ "./ambient_proxy_manager" ]
