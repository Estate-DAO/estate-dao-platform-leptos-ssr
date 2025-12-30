use maxminddb::{geoip2, Reader};
use once_cell::sync::OnceCell;
use std::net::IpAddr;
use std::path::Path;
use tracing::{info, warn};

static GEOIP_READER: OnceCell<Reader<Vec<u8>>> = OnceCell::new();

/// Location information from IP lookup
#[derive(Debug, Clone, Default)]
pub struct GeoLocation {
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
}

impl std::fmt::Display for GeoLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Build location string: City, State, Country (omit any that are None)
        let parts: Vec<&str> = [
            self.city.as_deref(),
            self.state.as_deref(),
            self.country.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect();

        if parts.is_empty() {
            write!(f, "Unknown")
        } else {
            write!(f, "{}", parts.join(", "))
        }
    }
}

/// Initialize the GeoIP reader from the database file
pub fn init_geoip(db_path: &str) {
    let path = Path::new(db_path);

    if !path.exists() {
        warn!("GeoIP database not found at: {}", db_path);
        return;
    }

    match Reader::open_readfile(path) {
        Ok(reader) => {
            if GEOIP_READER.set(reader).is_ok() {
                info!("GeoIP database loaded successfully from: {}", db_path);
            }
        }
        Err(e) => {
            warn!("Failed to load GeoIP database: {}", e);
        }
    }
}

/// Look up location information for an IP address
pub fn lookup_ip(ip_str: &str) -> Option<GeoLocation> {
    let reader = GEOIP_READER.get()?;

    // Parse IP address
    let ip: IpAddr = ip_str.parse().ok()?;

    // Look up in database
    let city: geoip2::City = reader.lookup(ip).ok()??;

    let city_name = city
        .city
        .and_then(|c| c.names)
        .and_then(|names| names.get("en").cloned())
        .map(|s| s.to_string());

    // Get state/subdivision (first one, usually the most specific)
    let state_name = city
        .subdivisions
        .and_then(|subs| subs.into_iter().next())
        .and_then(|sub| sub.names)
        .and_then(|names| names.get("en").cloned())
        .map(|s| s.to_string());

    let (country_name, country_code) = city
        .country
        .map(|c| {
            let name = c
                .names
                .and_then(|n| n.get("en").cloned())
                .map(|s| s.to_string());
            let code = c.iso_code.map(|s| s.to_string());
            (name, code)
        })
        .unwrap_or((None, None));

    Some(GeoLocation {
        city: city_name,
        state: state_name,
        country: country_name,
        country_code,
    })
}

/// Extract client IP from request headers (handles proxies)
pub fn extract_client_ip(headers: &axum::http::HeaderMap) -> Option<String> {
    // Try X-Forwarded-For first (may contain multiple IPs)
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(xff_str) = xff.to_str() {
            // Take the first IP (original client)
            if let Some(ip) = xff_str.split(',').next() {
                let ip = ip.trim();
                if !ip.is_empty() {
                    return Some(ip.to_string());
                }
            }
        }
    }

    // Try X-Real-IP
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip) = real_ip.to_str() {
            let ip = ip.trim();
            if !ip.is_empty() {
                return Some(ip.to_string());
            }
        }
    }

    // Try CF-Connecting-IP (Cloudflare)
    if let Some(cf_ip) = headers.get("cf-connecting-ip") {
        if let Ok(ip) = cf_ip.to_str() {
            let ip = ip.trim();
            if !ip.is_empty() {
                return Some(ip.to_string());
            }
        }
    }

    // Try Fly-Client-IP (Fly.io)
    if let Some(fly_ip) = headers.get("fly-client-ip") {
        if let Ok(ip) = fly_ip.to_str() {
            let ip = ip.trim();
            if !ip.is_empty() {
                return Some(ip.to_string());
            }
        }
    }

    None
}

/// Extract User-Agent from request headers
pub fn extract_user_agent(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}
