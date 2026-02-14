use ethers_core::types::{Address, Signature, H256};
use risc0_zkvm::guest::env;

fn main() {
    let message_hash: H256 = env::read();
    let signature: Signature = env::read();
    let owners: Vec<Address> = env::read();
    let nft_address: Address = env::read();

    let recovered = signature.recover(message_hash).unwrap();

    assert!(owners.contains(&recovered));

    env::commit(&owners);
    env::commit(&nft_address);
}
