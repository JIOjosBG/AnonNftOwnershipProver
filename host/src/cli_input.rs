use ethers::types::{Address, Signature};
use std::io::{self, BufRead};
use std::str::FromStr;

pub fn prover_inputs() -> Result<(Address, Signature), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    println!("Enter the address of the NFT contract:");
    let address_str = lines.next().ok_or("No input provided for address")??;

    let address = Address::from_str(address_str.trim())
        .map_err(|e| format!("Invalid Ethereum address: {}", e))?;

    println!("Enter a signature of the NFT owner:");
    let signature_str = lines.next().ok_or("No input provided for signature")??;

    let signature = Signature::from_str(signature_str.trim())
        .map_err(|e| format!("Invalid signature: {}", e))?;

    Ok((address, signature))
}

pub fn get_termbin_url() -> Result<String, Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    println!("Enter termbin url with proof:");
    let termbin_url = lines.next().ok_or("No input provided for address")??;

    Ok(termbin_url.trim().to_string())
}
