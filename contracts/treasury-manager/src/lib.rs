//! PulsarTrack - Treasury Manager (Soroban)
//! Single-admin treasury for platform fund management on Stellar.
//!
//! Events:
//! - ("treasury", "deposit"): [token: Address, amount: i128]
//! - ("treasury", "withdraw"): [token: Address, recipient: Address, amount: i128]

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env};

#[contracttype]
#[derive(Clone)]
pub struct TreasuryState {
    pub balance: i128,
    pub total_deposited: i128,
    pub total_withdrawn: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    State,
    TokenAddress,
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;

#[contract]
pub struct TreasuryManagerContract;

#[contractimpl]
impl TreasuryManagerContract {
    pub fn initialize(env: Env, admin: Address, token: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TokenAddress, &token);
        env.storage().instance().set(
            &DataKey::State,
            &TreasuryState {
                balance: 0,
                total_deposited: 0,
                total_withdrawn: 0,
            },
        );
    }

    pub fn deposit(env: Env, sender: Address, amount: i128) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        sender.require_auth();

        if amount <= 0 {
            panic!("amount must be positive");
        }

        let token: Address = env.storage().instance().get(&DataKey::TokenAddress).unwrap();
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&sender, &env.current_contract_address(), &amount);

        let mut state: TreasuryState =
            env.storage().instance().get(&DataKey::State).unwrap();
        state.balance = state
            .balance
            .checked_add(amount)
            .expect("balance overflow");
        state.total_deposited = state
            .total_deposited
            .checked_add(amount)
            .expect("total_deposited overflow");
        env.storage().instance().set(&DataKey::State, &state);

        env.events().publish(
            (symbol_short!("treasury"), symbol_short!("deposit")),
            (token, amount),
        );
    }

    pub fn withdraw(env: Env, admin: Address, recipient: Address, amount: i128) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        if amount <= 0 {
            panic!("amount must be positive");
        }

        let token: Address = env.storage().instance().get(&DataKey::TokenAddress).unwrap();
        let token_client = token::Client::new(&env, &token);

        // Use the actual on-chain balance as the authoritative source so that
        // any divergence between internal accounting and the real token balance
        // (e.g. from a direct transfer or a prior accounting bug) cannot cause
        // a panic inside token_client.transfer.
        let actual_balance = token_client.balance(&env.current_contract_address());
        if amount > actual_balance {
            panic!("insufficient on-chain token balance");
        }

        token_client.transfer(&env.current_contract_address(), &recipient, &amount);

        // Sync internal accounting to reflect the actual post-withdrawal state.
        let mut state: TreasuryState =
            env.storage().instance().get(&DataKey::State).unwrap();
        state.balance = actual_balance
            .checked_sub(amount)
            .expect("balance underflow");
        state.total_withdrawn = state
            .total_withdrawn
            .checked_add(amount)
            .expect("total_withdrawn overflow");
        env.storage().instance().set(&DataKey::State, &state);

        env.events().publish(
            (symbol_short!("treasury"), symbol_short!("withdraw")),
            (token, recipient, amount),
        );
    }

    pub fn get_state(env: Env) -> TreasuryState {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().instance().get(&DataKey::State).unwrap()
    }

    pub fn get_token(env: Env) -> Address {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().instance().get(&DataKey::TokenAddress).unwrap()
    }

    pub fn propose_admin(env: Env, current_admin: Address, new_admin: Address) {
        pulsar_common_admin::propose_admin(
            &env,
            &DataKey::Admin,
            &DataKey::PendingAdmin,
            current_admin,
            new_admin,
        );
    }

    pub fn accept_admin(env: Env, new_admin: Address) {
        pulsar_common_admin::accept_admin(&env, &DataKey::Admin, &DataKey::PendingAdmin, new_admin);
    }
}

mod test;
