#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}

fn mint(env: &Env, token_addr: &Address, to: &Address, amount: i128) {
    let sac = StellarAssetClient::new(env, token_addr);
    sac.mint(to, &amount);
}

fn setup(env: &Env) -> (VestingScheduleContractClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_addr = deploy_token(env, &token_admin);
    let contract_id = env.register_contract(None, VestingScheduleContract);
    let client = VestingScheduleContractClient::new(env, &contract_id);
    client.initialize(&admin);
    (client, admin, token_addr)
}

#[test]
fn test_upsert_schedule_preserves_claimed_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, token_addr) = setup(&env);
    let beneficiary = Address::generate(&env);
    let token_client = TokenClient::new(&env, &token_addr);

    mint(&env, &token_addr, &client.address, 200_000);
    client.upsert_schedule(
        &admin,
        &beneficiary,
        &token_addr,
        &100_000i128,
        &0u64,
        &100u64,
        &0u64,
    );

    env.ledger().with_mut(|li| {
        li.timestamp = 40;
    });
    assert_eq!(client.claim(&beneficiary), 40_000);

    client.upsert_schedule(
        &admin,
        &beneficiary,
        &token_addr,
        &200_000i128,
        &0u64,
        &200u64,
        &0u64,
    );

    let schedule = client.get_schedule(&beneficiary).unwrap();
    assert_eq!(schedule.claimed_amount, 40_000);

    env.ledger().with_mut(|li| {
        li.timestamp = 80;
    });
    assert_eq!(client.claim(&beneficiary), 40_000);
    assert_eq!(token_client.balance(&beneficiary), 80_000);
}
