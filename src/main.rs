// src/main.rs

mod scanner;
mod url_checker;
mod utils;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "portscanner-rs", version = "1.0")]
#[command(about = "Rust ile Gelişmiş IP ve URL Analiz Aracı", long_about = None)]
struct Args {
    /// Analiz edilecek IP adresi (örn: 8.8.8.8)
    #[arg(long, conflicts_with = "url")]
    ip: Option<String>,

    /// Taranacak port aralığı (örn: 1-1000). Sadece --ip ile kullanılır.
    #[arg(long, default_value = "1-65535", requires = "ip")]
    ports: String,

    /// Analiz edilecek tek URL (örn: https://example.com)
    #[arg(long)]
    url: Option<String>,

    /// Çıktı formatı (text, json)
    #[arg(long, default_value = "text")]
    output: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Some(ip) = args.ip {
        scanner::run_port_scan(&ip, &args.ports, &args.output).await;
    } else if let Some(url) = args.url {
        url_checker::analyze_url_with_output(&url, &args.output).await;
    } else {
        utils::print_error("Hata: Analiz için bir --ip veya --url parametresi girilmelidir.");
        utils::print_info("Daha fazla bilgi için --help kullanın.");
    }
}