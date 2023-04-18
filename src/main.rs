use ambient_proxy_manager::telemetry::init_subscriber;
use std::net::{SocketAddr, TcpListener};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_subscriber(
        "ambient_proxy_manager".into(),
        "info".into(),
        std::io::stdout,
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr)?;

    ambient_proxy_manager::app::run(listener).await
}
