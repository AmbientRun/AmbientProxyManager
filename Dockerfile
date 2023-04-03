FROM rust:1.67

ADD . /app
WORKDIR /app

RUN cargo build --release

ENTRYPOINT ["target/release/ambient_proxy_manager"]
