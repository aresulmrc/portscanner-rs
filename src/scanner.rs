// src/scanner.rs

use crate::utils;
use colored::*;
use dns_lookup::lookup_addr;
use serde::Serialize;
use serde_json;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

#[derive(Serialize)]
struct PortScanResult {
    hostname: String,
    ip_address: String,
    open_ports: Vec<PortInfo>,
}

#[derive(Serialize)]
struct PortInfo {
    port: u16,
    is_open: bool,
    service_banner: String,
    response_time_ms: u128,
}

pub async fn run_port_scan(ip_str: &str, port_range: &str, output: &str) {
    utils::print_info(&format!("{} IP adresi için tarama başlatılıyor...", ip_str));

    let hostname = if let Ok(ip) = ip_str.parse::<IpAddr>() {
        lookup_addr(&ip).unwrap_or_else(|_| "Hostname bulunamadı".to_string())
    } else {
        utils::print_error("Geçersiz IP adresi formatı.");
        return;
    };

    if output == "text" {
        utils::print_success(&format!("Hostname: {}", hostname));
    }

    let (start_port, end_port) = parse_port_range(port_range);
    let mut handles = Vec::new();

    for port in start_port..=end_port {
        let ip = ip_str.to_string();
        handles.push(tokio::spawn(async move { scan_port(&ip, port).await }));
    }

    let mut open_ports = Vec::new();
    for handle in handles {
        if let Ok(Some(port_info)) = handle.await {
            if output == "text" {
                print_port_info_text(&port_info);
            }
            open_ports.push(port_info);
        }
    }

    if output == "json" {
        let result = PortScanResult {
            hostname,
            ip_address: ip_str.to_string(),
            open_ports,
        };
        if let Ok(json_str) = serde_json::to_string_pretty(&result) {
            // Dosyaya yazmak
            let file_path = format!("port_scan_{}.json", ip_str);
            if let Err(e) = std::fs::write(&file_path, json_str) {
                utils::print_error(&format!("JSON dosyası yazılırken hata oluştu: {}", e));
            } else {
                utils::print_success(&format!("Sonuçlar başarıyla kaydedildi: {}", file_path));
            }
        }
    }
}

fn print_port_info_text(info: &PortInfo) {
    println!(
        "[{}] Port {} açık (yanıt: {}) - Servis: {}",
        "✓".green(),
        info.port.to_string().cyan(),
        utils::format_duration(info.response_time_ms).yellow(),
        info.service_banner.magenta()
    );
}

fn parse_port_range(range: &str) -> (u16, u16) {
    let parts: Vec<&str> = range.split('-').collect();
    let start = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(1);
    let end = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(65535);
    (start, end)
}

async fn scan_port(ip: &str, port: u16) -> Option<PortInfo> {
    let addr_str = format!("{}:{}", ip, port);
    if let Ok(addr) = addr_str.parse::<SocketAddr>() {
        let start_time = Instant::now();
        let duration = Duration::from_millis(1000);

        if let Ok(Ok(mut stream)) = timeout(duration, TcpStream::connect(&addr)).await {
            let elapsed = start_time.elapsed();
            let mut buffer = [0; 512];
            let _ = stream.write_all(b"GET / HTTP/1.0\r\n\r\n").await;
            let banner = match timeout(duration, stream.read(&mut buffer)).await {
                Ok(Ok(size)) if size > 0 => String::from_utf8_lossy(&buffer[..size])
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string(),
                _ => "Servis bilgisi alınamadı".to_string(),
            };
            return Some(PortInfo {
                port,
                is_open: true,
                service_banner: banner,
                response_time_ms: elapsed.as_millis(),
            });
        }
    }
    None
}
