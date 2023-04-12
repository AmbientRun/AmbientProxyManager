use std::net::{SocketAddr, TcpListener};

use actix_web::{
    dev::Server, get, http, web::Header, App, HttpMessage, HttpResponse, HttpServer, Responder,
};
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_actix_web::TracingLogger;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

#[get("/_ah/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("ok")
}

struct GAECountry([u8; 2]);

impl ToString for GAECountry {
    fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.0).to_string()
    }
}

impl http::header::TryIntoHeaderValue for GAECountry {
    type Error = http::header::InvalidHeaderValue;

    fn try_into_value(self) -> Result<http::header::HeaderValue, Self::Error> {
        self.0.try_into_value()
    }
}

impl http::header::Header for GAECountry {
    fn name() -> http::header::HeaderName {
        http::header::HeaderName::from_static("x-appengine-country")
    }

    fn parse<M: HttpMessage>(msg: &M) -> Result<Self, actix_web::error::ParseError> {
        let value = msg
            .headers()
            .get(Self::name())
            .ok_or(actix_web::error::ParseError::Header)?
            .as_bytes();
        if value.len() != 2 {
            return Err(actix_web::error::ParseError::Header);
        }
        Ok(GAECountry([value[0], value[1]]))
    }
}

const EU_PROXY: &str = "proxy-eu.ambient.run:7000";
const US_PROXY: &str = "proxy-us.ambient.run:7000";

#[get("/proxy")]
async fn get_proxy(country: Option<Header<GAECountry>>) -> impl Responder {
    // handle missing country header
    let country = country
        .map(|Header(country)| country.0)
        .unwrap_or([b'Z', b'Z']);

    // choose proxy based on country
    let proxy = match &country {
        b"US" | b"CA" | b"MX" | b"GL" | b"BM" | b"AG" | b"AI" | b"AW" | b"BS" | b"BB" | b"BZ"
        | b"VG" | b"KY" | b"CR" | b"CU" | b"CW" | b"DM" | b"DO" | b"SV" | b"GD" | b"GT" | b"HT"
        | b"HN" | b"JM" | b"MQ" | b"MS" | b"NI" | b"PA" | b"PR" | b"BL" | b"KN" | b"LC" | b"MF"
        | b"PM" | b"VC" | b"SX" | b"TT" | b"TC" | b"VI" | b"UM" => US_PROXY,

        b"AR" | b"BO" | b"BR" | b"CL" | b"CO" | b"EC" | b"FK" | b"GF" | b"GY" | b"PY" | b"PE"
        | b"SR" | b"UY" | b"VE" => US_PROXY,

        b"AL" | b"AD" | b"AM" | b"AT" | b"AZ" | b"BY" | b"BE" | b"BA" | b"BG" | b"HR" | b"CY"
        | b"CZ" | b"DK" | b"EE" | b"FI" | b"FR" | b"GE" | b"DE" | b"GR" | b"HU" | b"IS" | b"IE"
        | b"IT" | b"KZ" | b"XK" | b"LV" | b"LI" | b"LT" | b"LU" | b"MK" | b"MT" | b"MD" | b"MC"
        | b"ME" | b"NL" | b"NO" | b"PL" | b"PT" | b"RO" | b"RU" | b"SM" | b"RS" | b"SK" | b"SI"
        | b"ES" | b"SE" | b"CH" | b"UA" | b"GB" | b"VA" => EU_PROXY,

        _ => EU_PROXY,
    };

    HttpResponse::Ok().body(proxy)
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
    let subscriber = get_subscriber(
        "ambient_proxy_manager".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr)?;

    run(listener)?.await
}
