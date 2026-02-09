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
    use std::collections::HashMap;

    // ============================================================
    // REGRA 4: Bloqueio de bots e automação
    // ============================================================

    #[test]
    fn test_is_bot_googlebot() {
        assert!(is_bot("Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)"));
    }

    #[test]
    fn test_is_bot_bingbot() {
        assert!(is_bot("Mozilla/5.0 (compatible; bingbot/2.0; +http://www.bing.com/bingbot.htm)"));
    }

    #[test]
    fn test_is_bot_facebookexternalhit() {
        assert!(is_bot("facebookexternalhit/1.1 (+http://www.facebook.com/externalhit_uatext.php)"));
    }

    #[test]
    fn test_is_bot_semrushbot() {
        assert!(is_bot("Mozilla/5.0 (compatible; SemrushBot/7~bl; +http://www.semrush.com/bot.html)"));
    }

    #[test]
    fn test_is_bot_curl() {
        assert!(is_bot("curl/7.88.1"));
    }

    #[test]
    fn test_is_bot_wget() {
        assert!(is_bot("Wget/1.21.4"));
    }

    #[test]
    fn test_is_bot_python_requests() {
        assert!(is_bot("python-requests/2.31.0"));
    }

    #[test]
    fn test_is_bot_postman() {
        assert!(is_bot("PostmanRuntime/7.33.0"));
    }

    #[test]
    fn test_is_bot_real_iphone_not_bot() {
        assert!(!is_bot("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1"));
    }

    #[test]
    fn test_is_bot_real_android_not_bot() {
        assert!(!is_bot("Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.6099.43 Mobile Safari/537.36"));
    }

    #[test]
    fn test_is_bot_empty_ua_not_bot() {
        assert!(!is_bot(""));
    }

    #[test]
    fn test_is_automation_selenium() {
        assert!(is_automation_tool("Mozilla/5.0 Selenium WebDriver"));
    }

    #[test]
    fn test_is_automation_puppeteer() {
        assert!(is_automation_tool("Mozilla/5.0 HeadlessChrome/120.0 Puppeteer"));
    }

    #[test]
    fn test_is_automation_playwright() {
        assert!(is_automation_tool("Mozilla/5.0 Playwright/1.40"));
    }

    #[test]
    fn test_is_automation_headless_chrome() {
        assert!(is_automation_tool("Mozilla/5.0 HeadlessChrome/120.0.6099.0"));
    }

    #[test]
    fn test_is_automation_real_user_not_automation() {
        assert!(!is_automation_tool("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1"));
    }

    // ============================================================
    // REGRA 3: Somente mobile iPhone/Android
    // ============================================================

    #[test]
    fn test_mobile_iphone_safari() {
        assert!(is_mobile_device("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1"));
    }

    #[test]
    fn test_mobile_iphone_chrome() {
        assert!(is_mobile_device("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/120.0.6099.119 Mobile/15E148 Safari/604.1"));
    }

    #[test]
    fn test_mobile_iphone_facebook_app() {
        assert!(is_mobile_device("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/21A329 [FBAN/FBIOS;FBAV/441.0.0.36.110]"));
    }

    #[test]
    fn test_mobile_android_chrome() {
        assert!(is_mobile_device("Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.6099.43 Mobile Safari/537.36"));
    }

    #[test]
    fn test_mobile_android_samsung() {
        assert!(is_mobile_device("Mozilla/5.0 (Linux; Android 13; SM-S918B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.6099.43 Mobile Safari/537.36"));
    }

    #[test]
    fn test_mobile_android_facebook_app() {
        assert!(is_mobile_device("Mozilla/5.0 (Linux; Android 14; Pixel 8 Build/UQ1A.240105.004) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/120.0.6099.43 Mobile Safari/537.36 [FB_IAB/FB4A;FBAV/441.0.0.36.110]"));
    }

    #[test]
    fn test_mobile_ipad() {
        assert!(is_mobile_device("Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1"));
    }

    #[test]
    fn test_mobile_ipod() {
        assert!(is_mobile_device("Mozilla/5.0 (iPod touch; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15"));
    }

    #[test]
    fn test_desktop_windows_blocked() {
        assert!(!is_mobile_device("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
    }

    #[test]
    fn test_desktop_mac_blocked() {
        assert!(!is_mobile_device("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
    }

    #[test]
    fn test_desktop_linux_blocked() {
        assert!(!is_mobile_device("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
    }

    #[test]
    fn test_mobile_empty_ua_allowed() {
        assert!(is_mobile_device(""));
    }

    // ============================================================
    // REGRA 2: Somente anúncios Facebook/Instagram
    // ============================================================

    #[test]
    fn test_fb_ad_fbclid_param() {
        let mut params = HashMap::new();
        params.insert("fbclid".into(), "abc123xyz".into());
        assert!(is_facebook_ad_traffic("", "Mozilla/5.0 (iPhone)", &params));
    }

    #[test]
    fn test_fb_ad_igshid_param() {
        let mut params = HashMap::new();
        params.insert("igshid".into(), "abc123".into());
        assert!(is_facebook_ad_traffic("", "Mozilla/5.0 (iPhone)", &params));
    }

    #[test]
    fn test_fb_ad_utm_source_facebook() {
        let mut params = HashMap::new();
        params.insert("utm_source".into(), "facebook".into());
        assert!(is_facebook_ad_traffic("", "", &params));
    }

    #[test]
    fn test_fb_ad_utm_source_instagram() {
        let mut params = HashMap::new();
        params.insert("utm_source".into(), "instagram".into());
        assert!(is_facebook_ad_traffic("", "", &params));
    }

    #[test]
    fn test_fb_ad_utm_source_meta() {
        let mut params = HashMap::new();
        params.insert("utm_source".into(), "meta".into());
        assert!(is_facebook_ad_traffic("", "", &params));
    }

    #[test]
    fn test_fb_ad_utm_source_fb() {
        let mut params = HashMap::new();
        params.insert("utm_source".into(), "fb".into());
        assert!(is_facebook_ad_traffic("", "", &params));
    }

    #[test]
    fn test_fb_ad_utm_source_ig() {
        let mut params = HashMap::new();
        params.insert("utm_source".into(), "ig".into());
        assert!(is_facebook_ad_traffic("", "", &params));
    }

    #[test]
    fn test_fb_ad_referrer_l_facebook() {
        let empty: HashMap<String, String> = HashMap::new();
        assert!(is_facebook_ad_traffic("https://l.facebook.com/l.php?u=https://example.com", "Mozilla/5.0 (iPhone)", &empty));
    }

    #[test]
    fn test_fb_ad_referrer_lm_facebook() {
        let empty: HashMap<String, String> = HashMap::new();
        assert!(is_facebook_ad_traffic("https://lm.facebook.com/l.php?u=https://example.com", "", &empty));
    }

    #[test]
    fn test_fb_ad_referrer_l_instagram() {
        let empty: HashMap<String, String> = HashMap::new();
        assert!(is_facebook_ad_traffic("https://l.instagram.com/", "", &empty));
    }

    #[test]
    fn test_fb_ad_app_ua_fban() {
        let empty: HashMap<String, String> = HashMap::new();
        assert!(is_facebook_ad_traffic(
            "https://facebook.com/something",
            "Mozilla/5.0 (iPhone) [FBAN/FBIOS;FBAV/441.0]",
            &empty
        ));
    }

    #[test]
    fn test_fb_ad_no_indicators_blocked() {
        let empty: HashMap<String, String> = HashMap::new();
        assert!(!is_facebook_ad_traffic("", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0)", &empty));
    }

    #[test]
    fn test_fb_ad_google_referrer_blocked() {
        let empty: HashMap<String, String> = HashMap::new();
        assert!(!is_facebook_ad_traffic("https://www.google.com/", "Mozilla/5.0 (iPhone)", &empty));
    }

    #[test]
    fn test_fb_ad_direct_access_no_params_blocked() {
        let empty: HashMap<String, String> = HashMap::new();
        assert!(!is_facebook_ad_traffic("", "", &empty));
    }

    #[test]
    fn test_fb_ad_utm_source_google_blocked() {
        let mut params = HashMap::new();
        params.insert("utm_source".into(), "google".into());
        assert!(!is_facebook_ad_traffic("", "", &params));
    }

    #[test]
    fn test_fb_ad_utm_source_tiktok_blocked() {
        let mut params = HashMap::new();
        params.insert("utm_source".into(), "tiktok".into());
        assert!(!is_facebook_ad_traffic("", "", &params));
    }
}
