use reqwest;
use serde_json::Value;
use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.gemini.com/v1/symbols";
    let client = reqwest::Client::new();
    let response = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .send()
        .await?;

    if !response.status().is_success() {
        println!(
            "API isteği başarısız oldu. Durum kodu: {}",
            response.status()
        );
        return Ok(());
    }

    let text = response.text().await?;
    let symbols: Vec<String> = serde_json::from_str(&text)?;

    let mut coins: Vec<String> = symbols
        .into_iter()
        .filter(|s| s.ends_with("usd"))
        .map(|s| {
            let coin = s.trim_end_matches("usd").to_uppercase();
            format!("GEMINI:{}USD", coin)
        })
        .collect();

    if coins.is_empty() {
        println!("USD çiftleri bulunamadı. API yanıtını kontrol edin.");
        return Ok(());
    }

    coins.sort_by(|a, b| {
        let a_name = a.split(':').nth(1).unwrap_or("").trim_end_matches("USD");
        let b_name = b.split(':').nth(1).unwrap_or("").trim_end_matches("USD");

        let a_numeric = a_name
            .chars()
            .take_while(|c| c.is_numeric())
            .collect::<String>();
        let b_numeric = b_name
            .chars()
            .take_while(|c| c.is_numeric())
            .collect::<String>();

        if !a_numeric.is_empty() && !b_numeric.is_empty() {
            let a_num: u32 = a_numeric.parse().unwrap_or(0);
            let b_num: u32 = b_numeric.parse().unwrap_or(0);
            if a_num != b_num {
                return b_num.cmp(&a_num);
            }
        }

        if a_numeric.is_empty() != b_numeric.is_empty() {
            return if a_numeric.is_empty() {
                Ordering::Greater
            } else {
                Ordering::Less
            };
        }

        a_name.cmp(b_name)
    });

    let mut file = File::create("gemini_usd_markets.txt")?;
    for coin in &coins {
        writeln!(file, "{}", coin)?;
    }

    println!("Gemini USD spot piyasası coin listesi başarıyla oluşturuldu ve kaydedildi.");
    println!("Toplam {} adet USD çifti bulundu.", coins.len());

    Ok(())
}
