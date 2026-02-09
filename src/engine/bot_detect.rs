/// User agents de bots conhecidos
pub static BOT_USER_AGENTS: &[&str] = &[
    "googlebot",
    "bingbot",
    "slurp",
    "duckduckbot",
    "baiduspider",
    "yandexbot",
    "sogou",
    "exabot",
    "facebot",
    "facebookexternalhit",
    "ia_archiver",
    "mj12bot",
    "semrushbot",
    "ahrefsbot",
    "dotbot",
    "rogerbot",
    "seznambot",
    "crawler",
    "spider",
    "bot",
    "scraper",
    "curl",
    "wget",
    "python-requests",
    "httpie",
    "postman",
    "insomnia",
    "selenium",
    "phantomjs",
    "headless",
    "puppeteer",
    "playwright",
    "httrack",
    "apache-httpclient",
    "java/",
    "libwww",
    "lwp-trivial",
    "go-http-client",
    "php/",
    "ruby",
    "perl",
    "python-urllib",
    "adsbot",
    "mediapartners",
    "adreview",
    "facebookcatalog",
];

/// Assinaturas de ferramentas de automação
static AUTOMATION_SIGNATURES: &[&str] = &[
    "selenium",
    "webdriver",
    "puppeteer",
    "playwright",
    "phantomjs",
    "headless",
    "headlesschrome",
    "chromeheadless",
    "electron",
    "nightmare",
    "cypress",
    "browserless",
    "chrome-lighthouse",
    "inspect",
    "debugger",
    "lucid",
    "clarity",
];

/// Verifica se o User-Agent é de um bot conhecido
pub fn is_bot(ua: &str) -> bool {
    let lower = ua.to_lowercase();
    BOT_USER_AGENTS.iter().any(|bot| lower.contains(bot))
}

/// Verifica se é ferramenta de automação
pub fn is_automation_tool(ua: &str) -> bool {
    let lower = ua.to_lowercase();
    AUTOMATION_SIGNATURES.iter().any(|sig| lower.contains(sig))
}

/// Verifica se é dispositivo mobile (iPhone/Android)
pub fn is_mobile_device(ua: &str) -> bool {
    let lower = ua.to_lowercase();

    // UA vazio → libera para não perder cliente real
    if lower.trim().is_empty() {
        return true;
    }

    // iPhone / iPod
    if lower.contains("iphone") || lower.contains("ipod") {
        return true;
    }
    // Android
    if lower.contains("android") {
        return true;
    }
    // iPad
    if lower.contains("ipad") {
        return true;
    }

    // Desktop claramente identificado → bloqueia
    if lower.contains("windows nt")
        || lower.contains("macintosh")
        || lower.contains("x11")
        || (lower.contains("linux") && !lower.contains("android"))
    {
        return false;
    }

    // Desconhecido → liberar
    true
}

/// Verifica se é tráfego específico de anúncio Facebook/Instagram
pub fn is_facebook_ad_traffic(
    referer: &str,
    user_agent: &str,
    query_params: &std::collections::HashMap<String, String>,
) -> bool {
    let ref_lower = referer.to_lowercase();
    let ua_lower = user_agent.to_lowercase();

    // Referrers de anúncios Facebook/Instagram
    let ad_referers = [
        "l.facebook.com",
        "lm.facebook.com",
        "l.instagram.com",
        "business.facebook.com",
        "fb.com/ads",
        "facebook.com/ads",
        "instagram.com/ads",
    ];
    let referer_from_ad = ad_referers.iter().any(|r| ref_lower.contains(r));

    // User-Agent do app Facebook/Instagram
    let fb_app_signatures = [
        "fban/",
        "fbios",
        "fb_iab",
        "fbav/",
        "instagram",
        "[fban",
        "[fbss",
    ];
    let app_ua = fb_app_signatures.iter().any(|sig| ua_lower.contains(sig));

    // fbclid
    if query_params.contains_key("fbclid") {
        return true;
    }
    // igshid
    if query_params.contains_key("igshid") {
        return true;
    }
    // utm_source
    if let Some(utm) = query_params.get("utm_source") {
        let utm_lower = utm.to_lowercase();
        if ["facebook", "instagram", "fb", "ig", "meta"].contains(&utm_lower.as_str()) {
            return true;
        }
    }

    // Referrer de anúncio + UA do app
    if referer_from_ad && app_ua {
        return true;
    }
    // Referrer válido de Facebook ads
    if referer_from_ad {
        return true;
    }
    // App UA + referrer do Facebook/Instagram
    if app_ua && (ref_lower.contains("facebook.com") || ref_lower.contains("instagram.com")) {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_bot() {
        assert!(is_bot("Mozilla/5.0 (compatible; Googlebot/2.1)"));
        assert!(!is_bot("Mozilla/5.0 (iPhone; CPU iPhone OS 16_0)"));
    }

    #[test]
    fn test_is_mobile_device() {
        assert!(is_mobile_device("Mozilla/5.0 (iPhone; CPU iPhone OS 16_0)"));
        assert!(is_mobile_device("Mozilla/5.0 (Linux; Android 13)"));
        assert!(!is_mobile_device(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"
        ));
        assert!(!is_mobile_device(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"
        ));
    }

    #[test]
    fn test_is_facebook_ad_traffic() {
        let mut params = std::collections::HashMap::new();
        params.insert("fbclid".into(), "123".into());
        assert!(is_facebook_ad_traffic("", "", &params));

        let empty: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        assert!(!is_facebook_ad_traffic("", "", &empty));
    }
}
