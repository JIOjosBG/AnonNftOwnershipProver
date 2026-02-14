use ethers::{
    types::{Address, Signature},
    utils::{hash_message, to_checksum},
};
use methods::{ANON_HOLDER_GUEST_ELF, ANON_HOLDER_GUEST_ID};
use risc0_zkvm::{default_prover, serde::to_vec, ExecutorEnv, Receipt};

pub fn generate_proof(
    nft_address: Address,
    signature: Signature,
    nft_owners: Vec<Address>,
) -> Result<Receipt, String> {
    println!("Generating proof of of ownership");

    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    // tracing_subscriber::fmt()
    //     .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
    //     .init();

    let message = format!("Confirm ownership of {}", to_checksum(&nft_address, None));
    println!("Message: {}", message);

    let message_hash = hash_message(&message);

    let recovered = signature.recover(message_hash).unwrap();
    println!("Recovered {}", recovered);
    println!("{:?}", nft_owners);
    println!("{:?}", nft_owners.contains(&recovered));

    let env = ExecutorEnv::builder()
        .write(&message_hash)
        .unwrap()
        .write(&signature)
        .unwrap()
        .write(&nft_owners)
        .unwrap()
        .write(&nft_address)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();

    // Proof information by proving the specified ELF binary.
    // This struct contains the receipt along with statistics about execution of the guest
    let prove_info = prover.prove(env, ANON_HOLDER_GUEST_ELF).unwrap();

    // extract the receipt.
    let receipt = prove_info.receipt;

    let receipt_bytes = to_vec(&receipt).unwrap();
    println!("Receipt size: {} bytes", receipt_bytes.len());

    // let _owners: Vec<Address> = receipt.journal.decode().unwrap();
    // The receipt was verified at the end of proving, but the below code is an
    // example of how someone else could verify this receipt.
    // receipt.verify(ANON_HOLDER_GUEST_ID).unwrap();

    return Ok(receipt);
}

pub fn verify_and_extract_data(receipt: &Receipt) -> Result<(Vec<Address>, Address), String> {
    let (nft_owners, nft_address): (Vec<Address>, Address) = receipt.journal.decode().unwrap();
    receipt
        .verify(ANON_HOLDER_GUEST_ID)
        .map_err(|e| format!("Receipt verification failed: {}", e.to_string()))?;

    return Ok((nft_owners, nft_address));
}
