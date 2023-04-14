use std::sync::Arc;

use axum::{extract::State, response::{Response, IntoResponse}, http};
use prometheus_client::encoding::text::encode;

use crate::app::AppState;

const OPENMETRIC_MIME_HEADER_VALUE: http::HeaderValue = http::HeaderValue::from_static("application/openmetrics-text; version=1.0.0; charset=utf-8");

pub async fn get_metrics(State(app_state): State<Arc<AppState>>) -> Result<Response, http::StatusCode> {
    let mut body = String::new();
    if encode(&mut body, &app_state.registry).is_err() {
        return Err(http::StatusCode::INTERNAL_SERVER_ERROR);
    }
    let mut response = body.into_response();
    response.headers_mut().insert(http::header::CONTENT_TYPE, OPENMETRIC_MIME_HEADER_VALUE);
    Ok(response)
}
