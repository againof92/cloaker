use crate::models::{GeoCacheEntry, GeoInfo};
use chrono::{Duration, Utc};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::warn;

pub struct GeoCache {
    cache: RwLock<HashMap<String, GeoCacheEntry>>,
    client: reqwest::Client,
}

impl GeoCache {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            client,
        }
    }

    pub async fn get_geo_info(&self, ip: &str) -> GeoInfo {
        // Ignora IPs locais
        if ip == "127.0.0.1" || ip == "::1" || ip.starts_with("192.168.") || ip.starts_with("10.") {
            return GeoInfo {
                status: "success".into(),
                country: "Local".into(),
                country_code: "LO".into(),
                city: "Localhost".into(),
                isp: "Local Network".into(),
                ..Default::default()
            };
        }

        // Verifica cache
        if let Some(cached) = self.get_cached(ip).await {
            return cached;
        }

        // Tenta APIs em cascata
        if let Some(geo) = self.fetch_ipwho(ip).await {
            self.set_cached(ip, &geo).await;
            return geo;
        }
        if let Some(geo) = self.fetch_ipapi_co(ip).await {
            self.set_cached(ip, &geo).await;
            return geo;
        }
        if let Some(geo) = self.fetch_ip_api(ip).await {
            self.set_cached(ip, &geo).await;
            return geo;
        }

        warn!("GeoIP: Todas as APIs falharam para IP {}", ip);
        GeoInfo::default()
    }

    async fn get_cached(&self, ip: &str) -> Option<GeoInfo> {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(ip) {
            if Utc::now() < entry.expires {
                return Some(entry.info.clone());
            }
        }
        None
    }

    async fn set_cached(&self, ip: &str, info: &GeoInfo) {
        let mut cache = self.cache.write().await;
        cache.insert(
            ip.to_string(),
            GeoCacheEntry {
                info: info.clone(),
                expires: Utc::now() + Duration::minutes(10),
            },
        );
    }

    pub async fn cleanup(&self) {
        let now = Utc::now();
        let mut cache = self.cache.write().await;
        cache.retain(|_, v| v.expires > now);
    }

    /// ipwho.is
    async fn fetch_ipwho(&self, ip: &str) -> Option<GeoInfo> {
        #[derive(serde::Deserialize)]
        struct Connection {
            isp: Option<String>,
            org: Option<String>,
            asn: Option<String>,
        }
        #[derive(serde::Deserialize)]
        struct Security {
            proxy: Option<bool>,
            hosting: Option<bool>,
            tor: Option<bool>,
            anonymous: Option<bool>,
        }
        #[derive(serde::Deserialize)]
        struct Resp {
            success: Option<bool>,
            country: Option<String>,
            country_code: Option<String>,
            region: Option<String>,
            region_code: Option<String>,
            city: Option<String>,
            connection: Option<Connection>,
            security: Option<Security>,
        }

        let url = format!("https://ipwho.is/{}", ip);
        let resp: Resp = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;

        if resp.success != Some(true) || resp.country_code.as_deref().unwrap_or("").is_empty() {
            return None;
        }

        let conn = resp.connection.as_ref();
        let sec = resp.security.as_ref();
        Some(GeoInfo {
            status: "success".into(),
            country: resp.country.unwrap_or_default(),
            country_code: resp.country_code.unwrap_or_default(),
            region: resp.region_code.unwrap_or_default(),
            region_name: resp.region.unwrap_or_default(),
            city: resp.city.unwrap_or_default(),
            isp: conn.and_then(|c| c.isp.clone()).unwrap_or_default(),
            org: conn.and_then(|c| c.org.clone()).unwrap_or_default(),
            as_info: conn.and_then(|c| c.asn.clone()).unwrap_or_default(),
            proxy: sec
                .map(|s| {
                    s.proxy.unwrap_or(false)
                        || s.tor.unwrap_or(false)
                        || s.anonymous.unwrap_or(false)
                })
                .unwrap_or(false),
            hosting: sec.and_then(|s| s.hosting).unwrap_or(false),
        })
    }

    /// ipapi.co
    async fn fetch_ipapi_co(&self, ip: &str) -> Option<GeoInfo> {
        #[derive(serde::Deserialize)]
        struct Resp {
            error: Option<bool>,
            country_name: Option<String>,
            country: Option<String>,
            region: Option<String>,
            region_code: Option<String>,
            city: Option<String>,
            org: Option<String>,
            asn: Option<String>,
        }

        let url = format!("https://ipapi.co/{}/json/", ip);
        let resp: Resp = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;

        if resp.error == Some(true) || resp.country.as_deref().unwrap_or("").is_empty() {
            return None;
        }

        Some(GeoInfo {
            status: "success".into(),
            country: resp.country_name.unwrap_or_default(),
            country_code: resp.country.unwrap_or_default(),
            region: resp.region_code.unwrap_or_default(),
            region_name: resp.region.unwrap_or_default(),
            city: resp.city.unwrap_or_default(),
            isp: resp.org.clone().unwrap_or_default(),
            org: resp.org.unwrap_or_default(),
            as_info: resp.asn.unwrap_or_default(),
            proxy: false,
            hosting: false,
        })
    }

    /// ip-api.com
    async fn fetch_ip_api(&self, ip: &str) -> Option<GeoInfo> {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Resp {
            status: Option<String>,
            country: Option<String>,
            country_code: Option<String>,
            region: Option<String>,
            region_name: Option<String>,
            city: Option<String>,
            isp: Option<String>,
            org: Option<String>,
            #[serde(rename = "as")]
            as_info: Option<String>,
            proxy: Option<bool>,
            hosting: Option<bool>,
        }

        let url = format!("http://ip-api.com/json/{}?fields=66842623", ip);
        let resp: Resp = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;

        if resp.status.as_deref() != Some("success")
            || resp.country_code.as_deref().unwrap_or("").is_empty()
        {
            return None;
        }

        Some(GeoInfo {
            status: "success".into(),
            country: resp.country.unwrap_or_default(),
            country_code: resp.country_code.unwrap_or_default(),
            region: resp.region.unwrap_or_default(),
            region_name: resp.region_name.unwrap_or_default(),
            city: resp.city.unwrap_or_default(),
            isp: resp.isp.unwrap_or_default(),
            org: resp.org.unwrap_or_default(),
            as_info: resp.as_info.unwrap_or_default(),
            proxy: resp.proxy.unwrap_or(false),
            hosting: resp.hosting.unwrap_or(false),
        })
    }
}
