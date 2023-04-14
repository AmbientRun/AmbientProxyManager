use std::{sync::Arc, net::SocketAddr};

use axum::{extract::{State, ConnectInfo, TypedHeader}, headers::UserAgent};

use crate::app::AppState;


const EU_PROXY: &str = "proxy-eu.ambient.run:7000";
const US_PROXY: &str = "proxy-us.ambient.run:7000";
const AMBIENT_USER_AGENT_PREFIX: &str = "ambient_network/";

pub async fn get_proxy(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    State(app_state): State<Arc<AppState>>,
) -> &'static str {
    // FIXME: convert this into a middleware maybe
    // resolve continent and country
    let (continent, country) = app_state.ip_reader.as_ref().map(|ip_reader| {
        ip_reader
            .lookup_continent_and_country_code(addr.ip())
            .unwrap_or_default()
    }).unwrap_or_default();
    let continent = continent.unwrap_or("ZZ");
    let country = country.unwrap_or("ZZ");

    // parse ambient version from user agent
    let ambient_version = user_agent
        .as_str()
        .strip_prefix(AMBIENT_USER_AGENT_PREFIX)
        .unwrap_or_default()
        .trim();

    tracing::info!(
        "Proxy request from {} ({}/{}) with user agent {}",
        addr.ip(),
        continent,
        country,
        user_agent,
    );

    // increment metrics
    app_state.metrics.inc_proxy_requests(country.to_string(), ambient_version.to_string());

    // choose proxy based on country
    let proxy = match continent {
        "NA" | "SA" => US_PROXY,
        _ => EU_PROXY,
    };

    proxy
}
