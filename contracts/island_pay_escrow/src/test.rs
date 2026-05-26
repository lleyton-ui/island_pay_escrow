#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_happy_path_escrow_flow() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, IslandPayContract);
    let client = IslandPayContractClient::new(&env, &contract_id);
    
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = env.register_stellar_asset_contract(token_admin);
    let token = token::StellarAssetClient::new(&env, &token_addr);
    
    token.mint(&buyer, &1000);
    client.create_escrow(&buyer, &seller, &token_addr, &500);
    client.release_funds(&buyer);
    
    let seller_balance = token::Client::new(&env, &token_addr).balance(&seller);
    assert_eq!(seller_balance, 500);
}

#[test]
#[should_panic(expected = "Funds already released")]
fn test_fail_double_release() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, IslandPayContract);
    let client = IslandPayContractClient::new(&env, &contract_id);
    
    let (buyer, seller) = (Address::generate(&env), Address::generate(&env));
    let token_addr = env.register_stellar_asset_contract(Address::generate(&env));
    token::StellarAssetClient::new(&env, &token_addr).mint(&buyer, &1000);

    client.create_escrow(&buyer, &seller, &token_addr, &500);
    client.release_funds(&buyer);
    client.release_funds(&buyer); // Should panic
}

#[test]
fn test_state_verification() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, IslandPayContract);
    let client = IslandPayContractClient::new(&env, &contract_id);
    
    let (buyer, seller) = (Address::generate(&env), Address::generate(&env));
    let token_addr = env.register_stellar_asset_contract(Address::generate(&env));
    token::StellarAssetClient::new(&env, &token_addr).mint(&buyer, &1000);

    client.create_escrow(&buyer, &seller, &token_addr, &500);
    
    let info: EscrowInfo = env.storage().instance().get(&DataKey::Escrow(buyer)).unwrap();
    assert_eq!(info.amount, 500);
    assert_eq!(info.released, false);
}

#[test]
#[should_panic]
fn test_insufficient_funds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, IslandPayContract);
    let client = IslandPayContractClient::new(&env, &contract_id);
    
    let (buyer, seller) = (Address::generate(&env), Address::generate(&env));
    let token_addr = env.register_stellar_asset_contract(Address::generate(&env));
    // No minting here
    client.create_escrow(&buyer, &seller, &token_addr, &500);
}

#[test]
fn test_unauthorized_release_fails() {
    let env = Env::default();
    // No mock_all_auths() here to simulate real auth failure
    let contract_id = env.register_contract(None, IslandPayContract);
    let client = IslandPayContractClient::new(&env, &contract_id);
    let buyer = Address::generate(&env);
    
    let result = env.as_contract(&contract_id, || {
        client.try_release_funds(&buyer)
    });
    assert!(result.is_err());
}