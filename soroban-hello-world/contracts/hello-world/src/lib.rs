#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, IntoVal, String};

const MAX_TOKENS: u32 = 10000;
const TOKENS_RESERVED: u32 = 5;
const MAX_MINT_PER_TX: u32 = 10;

#[contracttype]
pub enum DataKey {
    IsSaleActive,
    TotalSupply,
    BaseUri,
    BaseExtension,
    Price,
    MintedPerWallet(Address),
    Owner,
    TokenOwner(u32),
}
#[contract]
pub struct NFTContract;

#[contractimpl]
impl NFTContract {
    // Initialization function to set up contract state
    pub fn init(env: Env, owner: Address) {
        owner.require_auth();

        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::IsSaleActive, &false);
        env.storage().instance().set(&DataKey::TotalSupply, &0u32);
        env.storage().instance().set(
            &DataKey::BaseUri,
            &String::from_str(
                &env,
                "https://ipfs.io/ipfs/QmUNLLsPACCz1vLxQVkXqqLX5R1X345qqfHbsf67hvA3Nn",
            ),
        );
        env.storage()
            .instance()
            .set(&DataKey::BaseExtension, &String::from_str(&env, ".json"));
        env.storage().instance().set(
            &DataKey::Price,
            &100_000_000_000_000_000u64, // 0.1 Ether in Wei
        );

        // Mint reserved tokens to the owner
        for i in 1..=TOKENS_RESERVED {
            let token_id = i as u32;
            Self::mint_token(&env, &owner, &token_id);
        }

        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &TOKENS_RESERVED);
    }

    // Internal function to mint a token to a specified address
    fn mint_token(env: &Env, to: &Address, token_id: &u32) {
        env.storage()
            .persistent()
            .set(&DataKey::TokenOwner(token_id.clone()), to);

        let key = DataKey::MintedPerWallet(to.clone());
        let count: u32 = env.storage().instance().get(&key).unwrap_or(0u32);
        env.storage().instance().set(&key, &(count + 1));
    }

    // Public function to mint new tokens
    pub fn mint(env: Env, to: Address, num_tokens: u32) {
        to.require_auth();

        // Check if the sale is active
        let is_sale_active: bool = env
            .storage()
            .instance()
            .get::<DataKey, bool>(&DataKey::IsSaleActive)
            .unwrap();
        if !is_sale_active {
            panic!("The sale is paused.");
        }

        // Validate the number of tokens requested
        if num_tokens > MAX_MINT_PER_TX {
            panic!("Cannot mint that many tokens in one transaction.");
        }

        let minted_key = DataKey::MintedPerWallet(to.clone());
        let minted: u32 = env
            .storage()
            .instance()
            .get::<DataKey, u32>(&minted_key)
            .unwrap_or(0u32);
        if minted + num_tokens > MAX_MINT_PER_TX {
            panic!("Cannot mint that many tokens in total.");
        }

        // Check if total supply exceeds the maximum
        let total_supply: u32 = env
            .storage()
            .instance()
            .get::<DataKey, u32>(&DataKey::TotalSupply)
            .unwrap();
        if total_supply + num_tokens > MAX_TOKENS {
            panic!("Exceeds total token supply.");
        }

        // Mint the tokens to the caller
        for i in 1..=num_tokens {
            let token_id = total_supply + i;

            Self::mint_token(&env, &to, &token_id);
        }

        // Update the minted tokens per wallet and total supply
        env.storage()
            .instance()
            .set(&minted_key, &(minted + num_tokens));
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(total_supply + num_tokens));
    }

    // Owner-only function to toggle the sale state
    pub fn flip_sale_state(env: Env, caller: Address) {
        let owner: Address = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Owner)
            .unwrap();
        caller.require_auth();

        let is_sale_active: bool = env
            .storage()
            .instance()
            .get::<DataKey, bool>(&DataKey::IsSaleActive)
            .unwrap();
        env.storage()
            .instance()
            .set(&DataKey::IsSaleActive, &!is_sale_active);
    }

    // Owner-only function to set the base URI
    pub fn set_base_uri(env: Env, base_uri: String) {
        let owner: Address = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Owner)
            .unwrap();
        owner.require_auth();

        env.storage().instance().set(&DataKey::BaseUri, &base_uri);
    }

    // Owner-only function to set the token price
    pub fn set_price(env: Env, price: i128) {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();

        env.storage().instance().set(&DataKey::Price, &price);
    }

    // Function to retrieve the token URI
    // pub fn token_uri(env: Env, token_id: u32) -> String {
    //     let owner_option: Option<Address> = env
    //         .storage().instance()
    //         .get::<DataKey, Option<Address>>(&DataKey::TokenOwner(token_id))
    //         .unwrap();
    //     if owner_option.is_none() {
    //         panic!("URI query for nonexistent token.");
    //     }

    //     let base_uri: String = env
    //         .storage().instance()
    //         .get::<DataKey, String>(&DataKey::BaseUri)
    //         .unwrap();
    //     let base_extension: String = env
    //         .storage().instance()
    //         .get::<DataKey, String>(&DataKey::BaseExtension)
    //         .unwrap();
    //     let token_id_string = token_id.to_string();

    //     base_uri + &token_id_string + &base_extension

    // }
}
