use base64::{engine::general_purpose, Engine as _};
use risc0_zkvm::serde::{from_slice, to_vec};
use risc0_zkvm::Receipt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
pub async fn upload_receipt_to_termbin(receipt: &Receipt) -> Result<String, String> {
    // RISC0's internal serialization (returns Vec<u32>)
    let receipt_words =
        to_vec(receipt).map_err(|e| format!("Failed to serialize receipt: {}", e))?;

    // Convert Vec<u32> to Vec<u8>
    let receipt_bytes: Vec<u8> = receipt_words
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect();

    let receipt_base64 = general_purpose::STANDARD.encode(&receipt_bytes);

    let mut stream = TcpStream::connect("termbin.com:9999")
        .await
        .map_err(|e| format!("Failed to connect to termbin.com: {}", e))?;
    stream
        .write_all(receipt_base64.as_bytes())
        .await
        .map_err(|e| format!("Failed to write to termbin: {}", e))?;
    stream
        .shutdown()
        .await
        .map_err(|e| format!("Failed to shutdown stream: {}", e))?;

    // Read the response (termbin URL)
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let url = response.trim_end_matches('\0').trim().to_string();

    if url.is_empty() {
        return Err("Received empty response from termbin".to_string());
    }

    Ok(url)
}

pub async fn get_receipt_from_termbin(termbin_url: String) -> Result<Receipt, String> {
    let response = reqwest::get(termbin_url)
        .await
        .map_err(|e| format!("Failed to fetch URL: {}", e))?;

    let base64_content = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let bytes = general_purpose::STANDARD
        .decode(base64_content.trim())
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    let words: Vec<u32> = bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    // Deserialize to Receipt
    let receipt =
        from_slice(&words).map_err(|e| format!("Failed to deserialize receipt: {}", e))?;

    Ok(receipt)
}
