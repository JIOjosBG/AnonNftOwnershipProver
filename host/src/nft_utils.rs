use ethers::types::Address;

use ethers::prelude::*;
use std::str::FromStr;
use std::sync::Arc;

abigen!(
    IERC721Enumerable,
    r#"[
        function totalSupply() external view returns (uint256)
        function tokenByIndex(uint256 index) external view returns (uint256)
        function balanceOf(address owner) external view returns (uint256)
        function ownerOf(uint256 tokenId) external view returns (address)
    ]"#
);

pub async fn fetch_nft_owners(nft_address: Address) -> Result<Vec<Address>, String> {
    // Setup provider
    let provider = Provider::<Http>::try_from("https://invictus.ambire.com/optimism")
        .map_err(|e| format!("Failed to create provider: {}", e))?;
    let client = Arc::new(provider);

    // Create contract instance
    let contract = IERC721Enumerable::new(nft_address, client);

    // Get total supply
    let total_supply: U256 = contract
        .total_supply()
        .call()
        .await
        .map_err(|e| format!("Failed to get total supply: {}", e))?;

    println!("Total supply: {}", total_supply);

    // Fetch all owners
    let mut owners = Vec::new();

    for i in 0..total_supply.as_u64() {
        // Get token ID at index
        let token_id = contract
            .token_by_index(U256::from(i))
            .call()
            .await
            .map_err(|e| format!("Failed to get token at index {}: {}", i, e))?;

        // Get owner of token
        let owner = contract
            .owner_of(token_id)
            .call()
            .await
            .map_err(|e| format!("Failed to get owner of token {}: {}", token_id, e))?;

        owners.push(owner);

        if (i + 1) % 100 == 0 {
            println!("Fetched {} owners...", i + 1);
        }
    }

    Ok(owners)
}

pub async fn check_are_all_owners_legit(
    nft_owners: &Vec<Address>,
    nft_address: Address,
) -> Result<bool, String> {
    // Setup provider
    let provider = Provider::<Http>::try_from("https://invictus.ambire.com/optimism")
        .map_err(|e| format!("Failed to create provider: {}", e))?;
    let client = Arc::new(provider);

    // Create contract instance
    let contract = IERC721Enumerable::new(nft_address, client);

    for &owner in nft_owners {
        // Get token ID at index
        let balance: U256 = contract
            .balance_of(owner)
            .call()
            .await
            .map_err(|e| format!("Failed to get if {} has balance: {}", owner, e))?;
        if balance == U256::from(0) {
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use ethers::abi::AbiDecode;

    use super::*;
    #[tokio::test]
    async fn test_fetch_nft_owners() {
        let nft_address = Address::from_str("0x3Bd57Bf93dE179d2e47e86319F144d7482503C7d").unwrap();

        let fetched_nft_owners = fetch_nft_owners(nft_address).await.unwrap();

        assert_eq!(fetched_nft_owners.len(), 26);
        assert_eq!(
            fetched_nft_owners[0],
            Address::from_str("0x6969174fd72466430a46e18234d0b530c9fd5f49").unwrap(),
        );
    }
    #[tokio::test]
    async fn test_check_if_owners_are_legit() {
        let nft_address = Address::from_str("0x3Bd57Bf93dE179d2e47e86319F144d7482503C7d").unwrap();
        let mut fetched_nft_owners = fetch_nft_owners(nft_address).await.unwrap();

        assert_eq!(
            check_are_all_owners_legit(&fetched_nft_owners, nft_address)
                .await
                .unwrap(),
            true
        );

        let random_addr = Address::from(::rand::random::<[u8; 20]>());
        fetched_nft_owners.insert(0, random_addr);
        assert_eq!(
            check_are_all_owners_legit(&fetched_nft_owners, nft_address)
                .await
                .unwrap(),
            false
        );
    }
}
