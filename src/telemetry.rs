use prometheus_client::{encoding::EncodeLabelSet, metrics::{counter::Counter, family::Family}};
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ProxyRequestLabels {
    pub country: String,
    pub ambient_version: String,
}

#[derive(Debug)]
pub struct Metrics {
    proxy_requests: Family<ProxyRequestLabels, Counter>,
}

impl Metrics {
    pub fn new_registery(registry: &mut prometheus_client::registry::Registry) -> Self {
        let proxy_requests = Family::default();
        registry.register("proxy_requests", "Count of /proxy requests", proxy_requests.clone());
        Self { proxy_requests }
    }

    pub fn inc_proxy_requests(&self, country: String, ambient_version: String) {
        tracing::debug!("Incrementing proxy request counter for country: {}, ambient_version: {}", country, ambient_version);
        self.proxy_requests.get_or_create(&ProxyRequestLabels { country, ambient_version }).inc();
    }
}

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> tracing_subscriber::fmt::MakeWriter<'a> + Send + Sync + 'static,
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
