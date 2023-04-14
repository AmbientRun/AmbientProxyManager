use std::{path::Path, net::IpAddr};

pub struct IPReader {
    reader: maxminddb::Reader<Vec<u8>>,
}

impl IPReader {
    pub fn discover() -> Option<Self> {
        let path = Path::new("country.mmdb");
        if path.exists() {
            match Self::new(path) {
                Ok(reader) => Some(reader),
                Err(e) => {
                    tracing::error!("Failed to open GeoIP database: {}", e);
                    None
                }
            }
        } else {
            tracing::warn!("GeoIP database not found at: {}", path.display());
            None
        }
    }

    pub fn new(path: impl AsRef<Path>) -> Result<Self, maxminddb::MaxMindDBError> {
        let reader = maxminddb::Reader::open_readfile(path)?;
        Ok(Self { reader })
    }

    pub fn lookup(&self, ip: IpAddr) -> Result<maxminddb::geoip2::Country, maxminddb::MaxMindDBError> {
        self.reader.lookup(ip)
    }

    pub fn lookup_continent_and_country_code(&self, ip: IpAddr) -> Result<(Option<&str>, Option<&str>), maxminddb::MaxMindDBError> {
        let country = self.lookup(ip)?;
        Ok((country.continent.and_then(|c| c.code), country.country.and_then(|c| c.iso_code)))
    }
}
