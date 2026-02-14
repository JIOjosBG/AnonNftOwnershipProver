use ethers::{
    types::{Address, Signature},
    utils::{hash_message, to_checksum},
};
use methods::{ANON_HOLDER_GUEST_ELF, ANON_HOLDER_GUEST_ID};
use risc0_zkvm::{default_prover, serde::to_vec, ExecutorEnv, Receipt};
use tracing_subscriber::fmt;

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
    let prove_info = prover
        .prove(env, ANON_HOLDER_GUEST_ELF)
        .map_err(|e| format!("Failed to generate proof {}", e.to_string()))?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::core::k256::ecdsa::SigningKey;
    use ethers::core::rand::thread_rng;
    use ethers::signers::{Signer, Wallet};
    use ethers::types::{Address, Signature};

    // Helper function to create a test wallet and sign a message
    async fn create_signed_message(
        nft_address: Address,
    ) -> (Wallet<SigningKey>, Signature, Address) {
        let wallet = Wallet::new(&mut thread_rng());
        let signer_address = wallet.address();
        let message = format!("Confirm ownership of {}", to_checksum(&nft_address, None));
        let signature = wallet.sign_message(message.as_bytes()).await.unwrap();
        (wallet, signature, signer_address)
    }

    fn generate_random_address() -> Address {
        Address::from(rand::random::<[u8; 20]>())
    }

    #[tokio::test]
    async fn test_generate_proof_with_valid_owner() {
        let nft_address = generate_random_address();
        let (_, signature, signer_address) = create_signed_message(nft_address).await;

        // Create owner list including the signer
        let mut nft_owners = vec![generate_random_address(), generate_random_address()];
        nft_owners.insert(0, signer_address);

        let receipt = generate_proof(nft_address, signature, nft_owners);

        assert!(
            &receipt.is_ok(),
            "Proof generation should succeed with valid owner"
        );

        let extracted_data =
            verify_and_extract_data(&receipt.unwrap()).expect("Should be valid proof");
        let (extracted_owners, extracted_address) = extracted_data;

        assert_eq!(
            extracted_address, nft_address,
            "Extracted address should match original"
        );
        assert_eq!(
            extracted_owners, extracted_owners,
            "Extracted owners should match original"
        );
    }

    #[tokio::test]
    async fn test_generate_proof_with_invalid_owner() {
        let nft_address = generate_random_address();
        let (_, signature, _) = create_signed_message(nft_address).await;

        // Create owner list WITHOUT the signer
        let nft_owners = vec![
            generate_random_address(),
            generate_random_address(),
            generate_random_address(),
        ];

        let result = generate_proof(nft_address, signature, nft_owners);

        assert!(
            result.is_err(),
            "Proof generation should fail when signer is not in owner list"
        );
    }

    #[tokio::test]
    async fn test_full_workflow_generate_and_verify() {
        // Test the complete workflow: generate proof -> verify -> extract data
        let nft_address = generate_random_address();
        let (_, signature, signer_address) = create_signed_message(nft_address).await;

        let nft_owners = vec![
            generate_random_address(),
            signer_address,
            generate_random_address(),
        ];
        let original_owners = nft_owners.clone();

        // Step 1: Generate proof
        let receipt =
            generate_proof(nft_address, signature, nft_owners).expect("Proof generation failed");

        // Step 2: Verify and extract
        let (extracted_owners, extracted_address) =
            verify_and_extract_data(&receipt).expect("Verification failed");

        // Step 3: Validate extracted data
        assert_eq!(extracted_address, nft_address);
        assert_eq!(extracted_owners, original_owners);
        assert!(extracted_owners.contains(&signer_address));
    }
}
