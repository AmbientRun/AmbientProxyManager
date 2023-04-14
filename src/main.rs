use std::net::{SocketAddr, TcpListener};
use ambient_proxy_manager::telemetry::{get_subscriber, init_subscriber};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber(
        "ambient_proxy_manager".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr)?;

    ambient_proxy_manager::app::run(listener).await
}
