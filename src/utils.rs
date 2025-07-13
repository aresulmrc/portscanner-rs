// src/utils.rs

use colored::*;

/// Başarılı mesajı renkli göster
pub fn print_success(msg: &str) {
    println!("{}", msg.green().bold());
}

/// Hata mesajı renkli göster
pub fn print_error(msg: &str) {
    println!("{}", msg.red().bold());
}

/// Bilgi mesajı renkli göster
pub fn print_info(msg: &str) {
    println!("{}", msg.cyan());
}

/// Süreyi ms cinsinden biçimlendir
pub fn format_duration(ms: u128) -> String {
    format!("{} ms", ms)
}
