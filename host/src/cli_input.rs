use ethers::types::{Address, Signature};
use ethers::utils::to_checksum;
use std::io::{self, BufRead};
use std::str::FromStr;

pub fn prover_inputs() -> Result<(Address, Signature), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    println!("Enter the address of the NFT contract:");
    let address_str = lines.next().ok_or("No input provided for address")??;

    let nft_address = Address::from_str(address_str.trim())
        .map_err(|e| format!("Invalid Ethereum address: {}", e))?;

    println!("Enter a signature of the NFT owner.");
    let message = format!("Confirm ownership of {}", to_checksum(&nft_address, None));
    println!("Signature should be over this message: {}", message);

    let signature_str = lines.next().ok_or("No input provided for signature")??;

    let signature = Signature::from_str(signature_str.trim())
        .map_err(|e| format!("Invalid signature: {}", e))?;

    Ok((nft_address, signature))
}

pub fn get_termbin_url() -> Result<String, Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    println!("Enter termbin url with proof:");
    let termbin_url = lines.next().ok_or("No input provided for url")??;

    Ok(termbin_url.trim().to_string())
}
