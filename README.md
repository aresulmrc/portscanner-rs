26-12-2025
# Portscanner-RS: Derinlemesine Analiz Aracı

Rust ile yazılmış, tek bir hedef (IP veya URL) üzerinde derinlemesine bilgi toplayan, hızlı ve modern bir komut satırı aracıdır.

## Temel Özellikler

- **Derinlemesine Port Taraması:**
  - Belirtilen IP'deki tüm portları (1-65535) asenkron olarak tarar.
  - Açık portlarda çalışan servisleri tespit etmek için **Banner Grabbing** yapar.
  - IP adresine ait alan adını bulmak için **Reverse DNS** sorgusu gerçekleştirir.

- **Kapsamlı URL Analizi:**
  - **Teknoloji Tespiti:** WordPress, React, jQuery gibi kullanılan web teknolojilerini belirler.
  - **Güvenlik Başlıkları:** HSTS, CSP gibi önemli güvenlik başlıklarının varlığını kontrol eder.
  - **DNS & Sayfa Detayları:** Sitenin IP adreslerini, sayfa başlığını ve meta açıklamasını çeker.
  - **`robots.txt` Kontrolü:** Sitenin arama motoru botlarına verdiği direktifleri kontrol eder.

- **Esnek Çıktı Formatları:**
  - **Metin:** Terminalde okunması kolay, renkli ve simgeli insan dostu çıktı.
  - **JSON:** Hem port tarama hem de URL analizi sonuçlarını yapılandırılmış JSON formatında dışa aktarma.

## Kurulum

```sh
git clone https://github.com/aresulmrc/portscanner-rs.git
cd portscanner-rs
cargo build --release
```
Projenin çalıştırılabilir dosyası `target/release/portscanner` altında olacaktır.

## Kullanım Örnekleri

### IP Adresi Analizi

```sh
# Bir IP adresindeki tüm portları tara ve Reverse DNS sorgusu yap
cargo run -- --ip 8.8.8.8

# Tarama sonucunu JSON formatında al
cargo run -- --ip 8.8.8.8 --output json
```

### URL Analizi

```sh
# Bir web sitesini derinlemesine analiz et
cargo run -- --url https://github.com

# Analiz sonucunu report.json dosyasına kaydet
cargo run -- --url https://github.com --output json
```

### Yardım Menüsü
Tüm komutları ve seçenekleri görmek için:
```sh
cargo run -- --help
```

## Bağımlılıklar
- `tokio`
- `reqwest`
- `clap`
- `colored`
- `serde` & `serde_json`
- `scraper`
- `dns-lookup`
- `url`
