// src/url_checker.rs

use crate::utils;
use colored::*;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::net::ToSocketAddrs;
use std::time::Instant;
use url::Url;

#[derive(Serialize)]
struct UrlAnalysisResult {
    url: String,
    ip_addresses: Vec<String>,
    response_time_ms: u128,
    http_status: u16,
    content_type: String,
    content_length: u64,
    server: String,
    powered_by: String,
    page_title: Option<String>,
    meta_description: Option<String>,
    robots_txt_found: bool,
    technologies: Vec<String>,
    security_headers: SecurityHeaders,
}

#[derive(Serialize)]
struct SecurityHeaders {
    hsts: bool,
    csp: bool,
    x_frame_options: bool,
}

/// Tek bir URL'yi analiz eder ve sonucu belirtilen formatta yazdırır.
pub async fn analyze_url_with_output(url_str: &str, output: &str) {
    utils::print_info(&format!("🌐 URL analizi yapılıyor: {}", url_str));
    let client = Client::new();

    if let Some(result) = analyze_url(&client, url_str).await {
        if output == "json" {
            if let Ok(json_str) = serde_json::to_string_pretty(&result) {
                if let Ok(mut file) = File::create("url_report.json") {
                    let _ = file.write_all(json_str.as_bytes());
                    utils::print_success("JSON raporu oluşturuldu: url_report.json");
                }
            }
        } else {
            print_text_report(&result);
        }
    }
}

/// URL'yi analiz eder ve sonucu bir struct olarak döndürür.
async fn analyze_url(client: &Client, url_str: &str) -> Option<UrlAnalysisResult> {
    let start = Instant::now();
    let response = client.get(url_str).send().await.ok()?;
    let elapsed = start.elapsed();

    let (ip_addresses, robots_txt_found) = get_domain_info(client, url_str).await;
    let http_status = response.status().as_u16();
    let headers = response.headers();

    let security_headers = SecurityHeaders {
        hsts: headers.contains_key("strict-transport-security"),
        csp: headers.contains_key("content-security-policy"),
        x_frame_options: headers.contains_key("x-frame-options"),
    };

    let content_type = headers.get("content-type")?.to_str().ok()?.to_string();
    let server = headers.get("server")?.to_str().ok()?.to_string();
    let powered_by = headers.get("x-powered-by").and_then(|v| v.to_str().ok()).unwrap_or("Bilinmiyor").to_string();
    let content_length = response.content_length().unwrap_or(0);

    let body = response.text().await.ok()?;
    let (page_title, meta_description) = find_meta_info(&body);
    let technologies = find_technologies(&body);

    Some(UrlAnalysisResult {
        url: url_str.to_string(),
        ip_addresses,
        response_time_ms: elapsed.as_millis(),
        http_status,
        content_type,
        content_length,
        server,
        powered_by,
        page_title,
        meta_description,
        robots_txt_found,
        technologies,
        security_headers,
    })
}

/// Alan adı ile ilgili bilgileri (IP, robots.txt) toplar.
async fn get_domain_info(client: &Client, url_str: &str) -> (Vec<String>, bool) {
    let mut ips = Vec::new();
    let mut robots_found = false;

    if let Ok(parsed_url) = Url::parse(url_str) {
        if let Some(host) = parsed_url.host_str() {
            if let Ok(addrs) = (host, parsed_url.port().unwrap_or(443)).to_socket_addrs() {
                ips = addrs.map(|a| a.ip().to_string()).collect();
            }
            if let Ok(robots_url) = parsed_url.join("/robots.txt") {
                if let Ok(resp) = client.get(robots_url).send().await {
                    robots_found = resp.status().is_success();
                }
            }
        }
    }
    (ips, robots_found)
}

/// HTML içeriğinden sayfa başlığı ve meta açıklamayı çeker.
fn find_meta_info(body: &str) -> (Option<String>, Option<String>) {
    let document = Html::parse_document(body);
    let title_selector = Selector::parse("title").ok();
    let meta_selector = Selector::parse("meta[name='description']").ok();

    let title = title_selector.and_then(|s| document.select(&s).next()).map(|el| el.inner_html().trim().to_string());
    let meta_desc = meta_selector.and_then(|s| document.select(&s).next()).and_then(|el| el.value().attr("content")).map(str::to_string);

    (title, meta_desc)
}

/// HTML içeriğinde bilinen teknolojileri arar.
fn find_technologies(body: &str) -> Vec<String> {
    let mut found = Vec::new();
    let document = Html::parse_document(body);
    let selectors = [
        ("WordPress", "link[href*='wp-content'], script[src*='wp-content'], meta[name='generator'][content*='WordPress']"),
        ("jQuery", "script[src*='jquery']"),
        ("React", "[data-reactroot], [data-reactid]"),
        ("Bootstrap", "link[href*='bootstrap'], script[src*='bootstrap']"),
    ];

    for (tech, selector_str) in selectors.iter() {
        if let Ok(selector) = Selector::parse(selector_str) {
            if document.select(&selector).next().is_some() {
                found.push(tech.to_string());
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

/// Analiz sonucunu terminale güzel bir formatta yazdırır.
fn print_text_report(r: &UrlAnalysisResult) {
    println!("\n--- 📝 Genel Bilgiler ---");
    println!("{:<20} {}", "URL:", r.url.cyan());
    println!("{:<20} {}", "HTTP Durumu:", r.http_status.to_string().green());
    println!("{:<20} {}", "Yanıt Süresi:", utils::format_duration(r.response_time_ms).yellow());
    println!("{:<20} {}", "IP Adresleri:", r.ip_addresses.join(", ").magenta());

    println!("\n--- 📦 İçerik Detayları ---");
    println!("{:<20} {}", "Sayfa Başlığı:", r.page_title.as_deref().unwrap_or("Bulunamadı").cyan());
    println!("{:<20} {}", "Meta Açıklama:", r.meta_description.as_deref().unwrap_or("Bulunamadı"));
    println!("{:<20} {}", "Content-Type:", r.content_type);
    println!("{:<20} {}", "Content-Length:", format!("{} bytes", r.content_length));

    println!("\n--- 💻 Sunucu ve Teknolojiler ---");
    println!("{:<20} {}", "Sunucu:", r.server.magenta());
    println!("{:<20} {}", "X-Powered-By:", r.powered_by.blue());
    if !r.technologies.is_empty() {
        println!("{:<20} {}", "Teknolojiler:", r.technologies.join(", ").yellow());
    }

    println!("\n--- 🛡️ Güvenlik Taraması ---");
    println!("{:<20} {}", "robots.txt:", if r.robots_txt_found { "Bulundu".green() } else { "Bulunamadı".red() });
    println!("{:<20} {}", "HSTS (Strict-Transport-Security):", if r.security_headers.hsts { "Aktif".green() } else { "Pasif".red() });
    println!("{:<20} {}", "CSP (Content-Security-Policy):", if r.security_headers.csp { "Aktif".green() } else { "Pasif".red() });
    println!("{:<20} {}", "X-Frame-Options:", if r.security_headers.x_frame_options { "Aktif".green() } else { "Pasif".red() });
}

