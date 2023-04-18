use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
};

use axum::{http::Method, routing::get, Router};
use prometheus_client::registry::Registry;
use tower_http::cors::CorsLayer;

use crate::{
    routes::{metrics::get_metrics, proxy::get_proxy},
    telemetry::Metrics,
    utils::IPReader,
};

pub struct AppState {
    pub registry: Registry,
    pub metrics: Metrics,
    pub ip_reader: Option<IPReader>,
}

impl Default for AppState {
    fn default() -> Self {
        let mut registry = Registry::default();
        let metrics = Metrics::new_registery(&mut registry);
        Self {
            registry,
            metrics,
            ip_reader: IPReader::discover(),
        }
    }
}

pub async fn run(listener: TcpListener) -> anyhow::Result<()> {
    tracing::debug!("Starting HTTP interface on: {:?}", listener.local_addr());

    let state = Arc::new(AppState::default());

    let app = Router::new()
        .route("/_ah/health", get(|| async move { "ok" }))
        .route("/health", get(|| async move { "ok" }))
        .route("/proxy", get(get_proxy))
        .route("/metrics", get(get_metrics))
        .with_state(state.clone())
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(vec![Method::GET])
                .allow_headers(tower_http::cors::Any),
        );

    axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .map_err(Into::into)
}
