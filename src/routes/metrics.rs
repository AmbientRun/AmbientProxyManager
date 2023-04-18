use std::sync::Arc;

use axum::{
    extract::State,
    http,
    response::{IntoResponse, Response},
};
use prometheus_client::encoding::text::encode;

use crate::app::AppState;

pub async fn get_metrics(
    State(app_state): State<Arc<AppState>>,
) -> Result<Response, http::StatusCode> {
    let mut body = String::new();
    if encode(&mut body, &app_state.registry).is_err() {
        return Err(http::StatusCode::INTERNAL_SERVER_ERROR);
    }
    let mut response = body.into_response();
    response.headers_mut().insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static(
            "application/openmetrics-text; version=1.0.0; charset=utf-8",
        ),
    );
    Ok(response)
}
