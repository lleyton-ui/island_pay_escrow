#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, token};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Escrow(Address), // Maps buyer address to escrow info
}

#[contracttype]
#[derive(Clone)]
pub struct EscrowInfo {
    pub buyer: Address,
    pub seller: Address,
    pub amount: i128,
    pub token: Address,
    pub released: bool,
}

#[contract]
pub struct IslandPayContract;

#[contractimpl]
impl IslandPayContract {
    /// Creates an escrow by transferring USDC from buyer to contract
    pub fn create_escrow(env: Env, buyer: Address, seller: Address, token_addr: Address, amount: i128) {
        buyer.require_auth();
        
        let client = token::Client::new(&env, &token_addr);
        client.transfer(&buyer, &env.current_contract_address(), &amount);

        let info = EscrowInfo {
            buyer: buyer.clone(),
            seller,
            amount,
            token: token_addr,
            released: false,
        };
        
        env.storage().instance().set(&DataKey::Escrow(buyer), &info);
    }

    /// Releases funds to the seller after shipping confirmation (Buyer Auth)
    pub fn release_funds(env: Env, buyer: Address) {
        buyer.require_auth();
        
        let mut info: EscrowInfo = env.storage().instance().get(&DataKey::Escrow(buyer.clone())).unwrap();
        if info.released { panic!("Funds already released"); }

        let client = token::Client::new(&env, &info.token);
        client.transfer(&env.current_contract_address(), &info.seller, &info.amount);

        info.released = true;
        env.storage().instance().set(&DataKey::Escrow(buyer), &info);
    }
}