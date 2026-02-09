use once_cell::sync::Lazy;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::IpAddr;

use crate::models::RedirectLink;

// ========== Mapa de estados brasileiros ==========
pub static BR_STATE_NAMES: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("AC", "Acre");
    m.insert("AL", "Alagoas");
    m.insert("AP", "Amapa");
    m.insert("AM", "Amazonas");
    m.insert("BA", "Bahia");
    m.insert("CE", "Ceara");
    m.insert("DF", "Distrito Federal");
    m.insert("ES", "Espirito Santo");
    m.insert("GO", "Goias");
    m.insert("MA", "Maranhao");
    m.insert("MT", "Mato Grosso");
    m.insert("MS", "Mato Grosso do Sul");
    m.insert("MG", "Minas Gerais");
    m.insert("PA", "Para");
    m.insert("PB", "Paraiba");
    m.insert("PR", "Parana");
    m.insert("PE", "Pernambuco");
    m.insert("PI", "Piaui");
    m.insert("RJ", "Rio de Janeiro");
    m.insert("RN", "Rio Grande do Norte");
    m.insert("RS", "Rio Grande do Sul");
    m.insert("RO", "Rondonia");
    m.insert("RR", "Roraima");
    m.insert("SC", "Santa Catarina");
    m.insert("SP", "Sao Paulo");
    m.insert("SE", "Sergipe");
    m.insert("TO", "Tocantins");
    m
});

// ========== Hash SHA256 ==========
pub fn hash_param(param: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(param.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn verify_param(link: &RedirectLink, param: &str) -> bool {
    if param.is_empty() {
        return false;
    }
    if !link.param_hash.is_empty() {
        return hash_param(param) == link.param_hash;
    }
    if !link.param_code.is_empty() {
        return link.param_code == param;
    }
    false
}

pub fn set_param(link: &mut RedirectLink, param: &str) {
    link.param_code = param.to_string();
    link.param_hash = hash_param(param);
}

// ========== Geração de código aleatório ==========
pub fn generate_code(length: usize) -> String {
    if length == 0 {
        return String::new();
    }
    let bytes_needed = length.div_ceil(2);
    let mut buf = vec![0u8; bytes_needed];
    rand::thread_rng().fill(&mut buf[..]);
    let hex_str = hex::encode(&buf);
    hex_str[..length].to_string()
}

// ========== Parse CSV ==========
pub fn parse_csv(input: &str) -> Vec<String> {
    let input = input.trim();
    if input.is_empty() {
        return vec![];
    }
    input
        .split([',', '\n', '\r', ';', '\t'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// ========== Comparação case-insensitive ==========
pub fn contains_ignore_case(list: &[String], value: &str) -> bool {
    let target = value.trim().to_uppercase();
    if target.is_empty() {
        return false;
    }
    list.iter().any(|item| item.trim().to_uppercase() == target)
}

// ========== Bloqueio de IP (suporta CIDR) ==========
pub fn is_ip_blocked(ip: &str, blocked_ips: &[String]) -> bool {
    let parsed: Option<IpAddr> = ip.parse().ok();

    for blocked in blocked_ips {
        if blocked.contains('/') {
            // CIDR
            if let (Some(ip_addr), Ok(network)) = (parsed, parse_cidr(blocked)) {
                if network_contains(&network, ip_addr) {
                    return true;
                }
            }
        } else if blocked == ip {
            return true;
        }
    }
    false
}

struct CidrNetwork {
    addr: IpAddr,
    prefix_len: u8,
}

fn parse_cidr(cidr: &str) -> Result<CidrNetwork, ()> {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return Err(());
    }
    let addr: IpAddr = parts[0].parse().map_err(|_| ())?;
    let prefix_len: u8 = parts[1].parse().map_err(|_| ())?;
    Ok(CidrNetwork { addr, prefix_len })
}

fn network_contains(network: &CidrNetwork, ip: IpAddr) -> bool {
    match (network.addr, ip) {
        (IpAddr::V4(net), IpAddr::V4(target)) => {
            let net_bits = u32::from(net);
            let target_bits = u32::from(target);
            let mask = if network.prefix_len >= 32 {
                u32::MAX
            } else {
                u32::MAX << (32 - network.prefix_len)
            };
            (net_bits & mask) == (target_bits & mask)
        }
        (IpAddr::V6(net), IpAddr::V6(target)) => {
            let net_bits = u128::from(net);
            let target_bits = u128::from(target);
            let mask = if network.prefix_len >= 128 {
                u128::MAX
            } else {
                u128::MAX << (128 - network.prefix_len)
            };
            (net_bits & mask) == (target_bits & mask)
        }
        _ => false,
    }
}

// ========== Bloqueio de ISP ==========
pub fn is_isp_blocked(isp: &str, org: &str, blocked_isps: &[String]) -> bool {
    let combined = format!("{} {}", isp, org).to_lowercase();
    blocked_isps
        .iter()
        .any(|b| combined.contains(&b.to_lowercase()))
}

// ========== Verificação de horário permitido ==========
pub fn is_within_allowed_hours(allowed_hours: &str) -> bool {
    let parts: Vec<&str> = allowed_hours.split('-').collect();
    if parts.len() != 2 {
        return true;
    }
    let now = chrono::Local::now();
    let current_minutes = now.hour() as i32 * 60 + now.minute() as i32;

    let start_parts: Vec<&str> = parts[0].split(':').collect();
    let end_parts: Vec<&str> = parts[1].split(':').collect();
    if start_parts.len() != 2 || end_parts.len() != 2 {
        return true;
    }

    let start_h: i32 = start_parts[0].parse().unwrap_or(0);
    let start_m: i32 = start_parts[1].parse().unwrap_or(0);
    let end_h: i32 = end_parts[0].parse().unwrap_or(0);
    let end_m: i32 = end_parts[1].parse().unwrap_or(0);

    let start_minutes = start_h * 60 + start_m;
    let end_minutes = end_h * 60 + end_m;

    if end_minutes < start_minutes {
        // Passa da meia-noite
        current_minutes >= start_minutes || current_minutes <= end_minutes
    } else {
        current_minutes >= start_minutes && current_minutes <= end_minutes
    }
}

// ========== Fix UTF-8 mojibake ==========
pub fn fix_utf8(s: &str) -> String {
    if s.is_empty()
        || (!s.contains('\u{00C3}') && !s.contains('\u{00C2}') && !s.contains('\u{FFFD}'))
    {
        return s.to_string();
    }
    let bytes: Vec<u8> = s
        .chars()
        .map(|c| {
            if c as u32 > 0xFF {
                return None;
            }
            Some(c as u8)
        })
        .collect::<Option<Vec<u8>>>()
        .unwrap_or_default();

    if bytes.is_empty() {
        return s.to_string();
    }

    match std::str::from_utf8(&bytes) {
        Ok(fixed) if !fixed.is_empty() => {
            let orig_bad = s.matches('\u{00C3}').count()
                + s.matches('\u{00C2}').count()
                + s.matches('\u{FFFD}').count();
            let fixed_bad = fixed.matches('\u{00C3}').count()
                + fixed.matches('\u{00C2}').count()
                + fixed.matches('\u{FFFD}').count();
            if fixed_bad > orig_bad {
                s.to_string()
            } else {
                fixed.to_string()
            }
        }
        _ => s.to_string(),
    }
}

// ========== Normalização de nomes de estado ==========
pub fn normalize_state_name(s: &str) -> String {
    let s = fix_utf8(s.trim());
    if s.is_empty() {
        return String::new();
    }
    let lower = s.to_lowercase();
    lower
        .replace(['á', 'à', 'â', 'ã', 'ä'], "a")
        .replace(['é', 'ê', 'è', 'ë'], "e")
        .replace(['í', 'ì', 'î', 'ï'], "i")
        .replace(['ó', 'ô', 'ò', 'õ', 'ö'], "o")
        .replace(['ú', 'ù', 'û', 'ü'], "u")
        .replace('ç', "c")
        .replace('-', " ")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

pub fn state_code_from_name(name: &str) -> String {
    let normalized = normalize_state_name(name);
    if normalized.is_empty() {
        return String::new();
    }
    for (code, nm) in BR_STATE_NAMES.iter() {
        if normalize_state_name(nm) == normalized {
            return code.to_string();
        }
    }
    String::new()
}

// ========== Validações ==========
pub fn validate_slug(slug: &str) -> Result<(), String> {
    let slug = slug.trim();
    if slug.is_empty() {
        return Err("Slug vazio".into());
    }
    let re = regex::Regex::new(r"^[a-zA-Z0-9._-]{1,64}$").unwrap();
    if !re.is_match(slug) {
        return Err("Slug invalido. Use apenas letras, numeros, '.', '_' ou '-' (1-64)".into());
    }
    Ok(())
}

pub fn validate_param_name(name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Nome de parametro vazio".into());
    }
    let re = regex::Regex::new(r"^[a-zA-Z0-9_-]{1,32}$").unwrap();
    if !re.is_match(name) {
        return Err(
            "Nome do parametro invalido. Use apenas letras, numeros, '_' ou '-' (1-32)".into(),
        );
    }
    Ok(())
}

pub fn validate_offer_url(raw: &str) -> Result<(), String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("URL da oferta vazia".into());
    }
    match url::Url::parse(raw) {
        Ok(u) => {
            if u.scheme() != "http" && u.scheme() != "https" {
                return Err("URL da oferta deve usar http ou https".into());
            }
            if u.host().is_none() {
                return Err("URL da oferta invalida".into());
            }
            Ok(())
        }
        Err(_) => Err("URL da oferta invalida".into()),
    }
}

// ========== HTML escape ==========
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

// ========== Clean SVG ==========
pub fn clean_svg(svg: &str) -> String {
    if svg.is_empty() {
        return String::new();
    }
    let mut result = String::new();
    let mut in_comment = false;
    for line in svg.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<?xml") {
            continue;
        }
        if trimmed.starts_with("<!--") {
            in_comment = true;
        }
        if in_comment {
            if trimmed.contains("-->") {
                in_comment = false;
            }
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }
    result.trim().to_string()
}

// ========== Fetch safe page HTML ==========
pub async fn fetch_safe_page_html(url_str: &str, client: &reqwest::Client) -> Option<String> {
    let url_str = url_str.trim();
    if url_str.is_empty() {
        return None;
    }
    let resp = client
        .get(url_str)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(4))
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let body = resp.text().await.ok()?;
    if body.is_empty() {
        return None;
    }

    // Injeta <base> tag
    let base_href = base_href_for_url(url_str);
    if !base_href.is_empty() {
        Some(inject_base_tag(&body, &base_href))
    } else {
        Some(body)
    }
}

fn base_href_for_url(raw: &str) -> String {
    match url::Url::parse(raw.trim()) {
        Ok(mut u) => {
            let path = u.path().to_string();
            if path.is_empty() || path == "/" {
                u.set_path("/");
            } else if !path.ends_with('/') {
                let dir = match path.rfind('/') {
                    Some(idx) => {
                        let d = &path[..=idx];
                        if d.is_empty() {
                            "/"
                        } else {
                            d
                        }
                    }
                    None => "/",
                };
                u.set_path(dir);
            }
            u.set_query(None);
            u.set_fragment(None);
            u.to_string()
        }
        Err(_) => String::new(),
    }
}

fn inject_base_tag(html: &str, base_href: &str) -> String {
    if html.is_empty() || base_href.is_empty() {
        return html.to_string();
    }
    let lower = html.to_lowercase();
    if lower.contains("<base") {
        return html.to_string();
    }
    let base_tag = format!("<base href=\"{}\">", html_escape(base_href));
    if let Some(head_idx) = lower.find("<head") {
        if let Some(end) = lower[head_idx..].find('>') {
            let insert_pos = head_idx + end + 1;
            return format!("{}{}{}", &html[..insert_pos], base_tag, &html[insert_pos..]);
        }
    }
    format!("{}{}", base_tag, html)
}

// ========== Base64 encode JSON ==========
pub fn encode_json_base64<T: serde::Serialize>(v: &T) -> String {
    use base64::Engine;
    let data = serde_json::to_string(v).unwrap_or_else(|_| "[]".into());
    base64::engine::general_purpose::STANDARD.encode(data.as_bytes())
}

// ========== Normalize IP ==========
pub fn normalize_ip(ip: &str) -> String {
    let ip = ip.trim();
    if ip.is_empty() {
        return String::new();
    }
    // Tenta extrair host de host:port
    if let Some(bracket_end) = ip.find(']') {
        // IPv6 com porta [::1]:8080
        return ip[1..bracket_end].to_string();
    }
    if ip.matches(':').count() == 1 {
        // IPv4 com porta
        if let Some(idx) = ip.rfind(':') {
            return ip[..idx].to_string();
        }
    }
    ip.to_string()
}

pub fn first_forwarded_ip(xff: &str) -> String {
    for part in xff.split(',') {
        let ip = normalize_ip(part);
        if !ip.is_empty() {
            return ip;
        }
    }
    String::new()
}

pub fn get_client_ip(
    headers: &axum::http::HeaderMap,
    remote_addr: Option<std::net::SocketAddr>,
) -> String {
    // CF-Connecting-IP (Cloudflare)
    if let Some(cf) = headers.get("CF-Connecting-IP") {
        let ip = normalize_ip(cf.to_str().unwrap_or(""));
        if !ip.is_empty() {
            return ip;
        }
    }
    // X-Real-IP
    if let Some(xr) = headers.get("X-Real-IP") {
        let ip = normalize_ip(xr.to_str().unwrap_or(""));
        if !ip.is_empty() {
            return ip;
        }
    }
    // X-Forwarded-For
    if let Some(xff) = headers.get("X-Forwarded-For") {
        let ip = first_forwarded_ip(xff.to_str().unwrap_or(""));
        if !ip.is_empty() {
            return ip;
        }
    }
    // Remote addr
    remote_addr.map(|a| a.ip().to_string()).unwrap_or_default()
}

use chrono::Timelike;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_param() {
        let hash1 = hash_param("test123");
        let hash2 = hash_param("test123");
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash_param("other"));
    }

    #[test]
    fn test_parse_csv() {
        let result = parse_csv(" a, b\nc;d\te ");
        assert_eq!(result.len(), 5);
        assert_eq!(result[0], "a");
        assert_eq!(result[4], "e");
    }

    #[test]
    fn test_is_ip_blocked() {
        let blocked = vec!["1.2.3.4".into(), "10.0.0.0/8".into()];
        assert!(is_ip_blocked("1.2.3.4", &blocked));
        assert!(is_ip_blocked("10.1.2.3", &blocked));
        assert!(!is_ip_blocked("8.8.8.8", &blocked));
    }

    #[test]
    fn test_normalize_state_name() {
        assert_eq!(normalize_state_name("São   Paulo"), "sao paulo");
        assert_eq!(normalize_state_name("  RIO-DE-JANEIRO "), "rio de janeiro");
    }

    #[test]
    fn test_validate_offer_url() {
        assert!(validate_offer_url("https://example.com").is_ok());
        assert!(validate_offer_url("http://example.com/path").is_ok());
        assert!(validate_offer_url("ftp://example.com").is_err());
        assert!(validate_offer_url("example.com").is_err());
        assert!(validate_offer_url("").is_err());
    }

    #[test]
    fn test_validate_slug() {
        assert!(validate_slug("oferta-1").is_ok());
        assert!(validate_slug("oferta_1").is_ok());
        assert!(validate_slug("oferta.1").is_ok());
        assert!(validate_slug("oferta 1").is_err());
        assert!(validate_slug("").is_err());
    }

    // ============================================================
    // REGRA 1: Parâmetro secreto obrigatório (?apx=...)
    // ============================================================

    fn make_test_link(param_code: &str) -> RedirectLink {
        let mut link = RedirectLink {
            id: "test-id".into(),
            slug: "test-slug".into(),
            param_hash: String::new(),
            param_code: String::new(),
            offer_url: "https://example.com".into(),
            safe_page_url: String::new(),
            clicks: 0,
            blocked: 0,
            created_at: chrono::Utc::now(),
            active: true,
            cloaker_active: true,
            ad_verify_mode: false,
            max_clicks: 0,
            param_ttl: 0,
            allowed_countries: vec![],
            blocked_countries: vec![],
            blocked_ips: vec![],
            blocked_isps: vec![],
            block_vpn: false,
            mobile_only: true,
            allowed_hours: String::new(),
            require_facebook: true,
            protection_total: false,
            strict_param_required: true,
            only_facebook_ads: true,
            advanced_fingerprint: false,
            ml_bot_detection: false,
            dynamic_referrer_spoof: false,
        };
        if !param_code.is_empty() {
            set_param(&mut link, param_code);
        }
        link
    }

    #[test]
    fn test_param_correct_code_accepted() {
        let link = make_test_link("abc123");
        assert!(verify_param(&link, "abc123"));
    }

    #[test]
    fn test_param_wrong_code_rejected() {
        let link = make_test_link("abc123");
        assert!(!verify_param(&link, "wrong-code"));
    }

    #[test]
    fn test_param_empty_code_rejected() {
        let link = make_test_link("abc123");
        assert!(!verify_param(&link, ""));
    }

    #[test]
    fn test_param_no_code_set_rejects_everything() {
        let link = make_test_link("");
        assert!(!verify_param(&link, "anything"));
    }

    #[test]
    fn test_param_hash_verification_works() {
        let link = make_test_link("my-secret-code");
        // Verifica que o hash foi gerado
        assert!(!link.param_hash.is_empty());
        // Verifica que o param_code foi armazenado
        assert_eq!(link.param_code, "my-secret-code");
        // Verifica pelo hash
        assert_eq!(link.param_hash, hash_param("my-secret-code"));
        // Código correto aceito
        assert!(verify_param(&link, "my-secret-code"));
        // Código parecido rejeitado
        assert!(!verify_param(&link, "my-secret-cod"));
        assert!(!verify_param(&link, "My-Secret-Code"));
    }

    #[test]
    fn test_param_case_sensitive() {
        let link = make_test_link("AbCdEf");
        assert!(verify_param(&link, "AbCdEf"));
        assert!(!verify_param(&link, "abcdef"));
        assert!(!verify_param(&link, "ABCDEF"));
    }

    #[test]
    fn test_param_special_characters() {
        let link = make_test_link("p@r4m!#$%");
        assert!(verify_param(&link, "p@r4m!#$%"));
        assert!(!verify_param(&link, "p@r4m"));
    }

    #[test]
    fn test_set_param_updates_both_fields() {
        let mut link = make_test_link("");
        set_param(&mut link, "new-code");
        assert_eq!(link.param_code, "new-code");
        assert_eq!(link.param_hash, hash_param("new-code"));
        assert!(verify_param(&link, "new-code"));
    }

    #[test]
    fn test_generate_code_length() {
        let code = generate_code(8);
        assert_eq!(code.len(), 8);
        let code2 = generate_code(16);
        assert_eq!(code2.len(), 16);
        let code3 = generate_code(0);
        assert!(code3.is_empty());
    }

    #[test]
    fn test_generate_code_unique() {
        let code1 = generate_code(16);
        let code2 = generate_code(16);
        assert_ne!(code1, code2);
    }
}
