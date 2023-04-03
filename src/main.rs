use std::net::{TcpListener, SocketAddr};

use actix_web::{get, HttpResponse, Responder, HttpServer, App, dev::Server};
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_actix_web::TracingLogger;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

#[get("/_ah/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("ok")
}

#[get("/proxy")]
async fn get_proxy() -> impl Responder {
    // TODO: implement the logic
    HttpResponse::Ok().body("35.228.233.193:7000")
}

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn run(listener: TcpListener) -> std::io::Result<Server> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(health_check)
            .service(get_proxy)
    })
    .listen(listener)?
    .run();
    Ok(server)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("ambient_proxy_manager".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr)?;

    run(listener)?.await
}
