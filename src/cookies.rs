//! Retrieve cookies from web browsers
//


use std::collections::HashMap;
use anyhow::{anyhow, Result, Context};
use decrypt_cookies::{
    prelude::*,
    prelude::cookies::CookiesInfo,
};


async fn read_browser_cookies_chromium() -> Result<HashMap<String, String>> {
    use decrypt_cookies::chromium::GetCookies;

    let chromium = ChromiumBuilder::<Chromium>::new()
        .build().await
        .context("building Chromium cookie extractor")?;
    let mut cookies = HashMap::new();
    for cookie in chromium.cookies_all().await
        .context("retrieving Chromium cookies")?
    {
        cookies.insert(cookie.set_cookie_header(), cookie.url());
    }
    Ok(cookies)
}


async fn read_browser_cookies_chrome() -> Result<HashMap<String, String>> {
    use decrypt_cookies::chromium::GetCookies;

    let chrome = ChromiumBuilder::<Chrome>::new()
        .build().await
        .context("building Chrome cookie extractor")?;
    let mut cookies = HashMap::new();
    for cookie in chrome.cookies_all().await
        .context("retrieving Chrome cookies")?
    {
        cookies.insert(cookie.set_cookie_header(), cookie.url());
    }
    Ok(cookies)
}

async fn read_browser_cookies_edge() -> Result<HashMap<String, String>> {
    use decrypt_cookies::chromium::GetCookies;

    let edge = ChromiumBuilder::<Edge>::new()
        .build().await
        .context("building Edge cookie extractor")?;
    let mut cookies = HashMap::new();
    for cookie in edge.cookies_all().await
        .context("retrieving Edge cookies")?
    {
        cookies.insert(cookie.set_cookie_header(), cookie.url());
    }
    Ok(cookies)
}

async fn read_browser_cookies_brave() -> Result<HashMap<String, String>> {
    use decrypt_cookies::chromium::GetCookies;

    let brave = ChromiumBuilder::<Brave>::new()
        .build().await
        .context("building Brave cookie extractor")?;
    let mut cookies = HashMap::new();
    for cookie in brave.cookies_all().await
        .context("retrieving Brave cookies")?
    {
        cookies.insert(cookie.set_cookie_header(), cookie.url());
    }
    Ok(cookies)
}

async fn read_browser_cookies_opera() -> Result<HashMap<String, String>> {
    use decrypt_cookies::chromium::GetCookies;

    let opera = ChromiumBuilder::<Opera>::new()
        .build().await
        .context("building Opera cookie extractor")?;
    let mut cookies = HashMap::new();
    for cookie in opera.cookies_all().await
        .context("retrieving Opera cookies")?
    {
        cookies.insert(cookie.set_cookie_header(), cookie.url());
    }
    Ok(cookies)
}

async fn read_browser_cookies_firefox() -> Result<HashMap<String, String>> {
    use decrypt_cookies::firefox::GetCookies;

    let firefox = FirefoxBuilder::<Firefox>::new()
        .build().await
        .context("building Firefox cookie extractor")?;
    let mut cookies = HashMap::new();
    for cookie in firefox.cookies_all().await
        .context("retrieving Firefox cookies")?
    {
        cookies.insert(cookie.set_cookie_header(), cookie.url());
    }
    Ok(cookies)
}

#[cfg(target_os="macos")]
async fn read_browser_cookies_safari() -> Result<HashMap<String, String>> {
    use decrypt_cookies_rs::prelude::{SafariBuilder, SafariCookie, SafariGetter};

    let safari = SafariBuilder::new().build().await
        .context("building Safari cookie extractor")?;
    let mut cookies = HashMap::new();
    for cookie in safari.cookies_all().await
        .context("retrieving Safari cookies")?
    {
        cookies.insert(cookie.set_cookie_header(), cookie.url());
    }
    Ok(cookies)
}


pub(crate) async fn read_browser_cookies(browser_name: &str) -> Result<HashMap<String, String>> {
    match browser_name.to_lowercase().as_str() {
        "chromium" => read_browser_cookies_chromium().await,
        "chrome" => read_browser_cookies_chrome().await,
        "edge" => read_browser_cookies_edge().await,
        "brave" => read_browser_cookies_brave().await,
        "opera" => read_browser_cookies_opera().await,
        "firefox" => read_browser_cookies_firefox().await,
        #[cfg(target_os="macos")]
        "safari" => read_browser_cookies_safari().await,
        _ => Err(anyhow!("unknown browser {browser_name} in --cookies-from-browser")),
    }
}


pub(crate) async fn list_cookie_sources() -> HashMap<String, u32> {
    let mut sources = HashMap::new();

    if let Ok(cookies) = read_browser_cookies_chromium().await {
        if let Ok(count) = cookies.len().try_into() {
            sources.insert(String::from("Chromium"), count);
        }
    }
    if let Ok(cookies) = read_browser_cookies_chrome().await {
        if let Ok(count) = cookies.len().try_into() {
            sources.insert(String::from("Chrome"), count);
        }
    }
    if let Ok(cookies) = read_browser_cookies_edge().await {
        if let Ok(count) = cookies.len().try_into() {
            sources.insert(String::from("Edge"), count);
        }
    }
    if let Ok(cookies) = read_browser_cookies_brave().await {
        if let Ok(count) = cookies.len().try_into() {
            sources.insert(String::from("Brave"), count);
        }
    }
    if let Ok(cookies) = read_browser_cookies_opera().await {
        if let Ok(count) = cookies.len().try_into() {
            sources.insert(String::from("Opera"), count);
        }
    }
    if let Ok(cookies) = read_browser_cookies_firefox().await {
        if let Ok(count) = cookies.len().try_into() {
            sources.insert(String::from("Firefox"), count);
        }
    }
    #[cfg(target_os="macos")]
    if let Ok(cookies) = read_browser_cookies_safari().await {
        if let Ok(count) = cookies.len().try_into() {
            sources.insert(String::from("Safari"), count);
        }
    }
    sources
}
