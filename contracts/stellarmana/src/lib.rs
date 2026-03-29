#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug)]
pub struct ManaPool {
    pub owner: Address,
    pub max_mana: u32,
    pub current_mana: u32,
    pub regen_rate: u32, // mana per minute
    pub last_updated: u64, // timestamp
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ManaStatus {
    pub current_mana: u32,
    pub max_mana: u32,
    pub regen_rate: u32,
    pub last_updated: u64,
}

const MANA_DATA: Symbol = symbol_short!("MANA_DATA");

#[contract]
pub struct ManaContract;

#[contractimpl]
impl ManaContract {
    /// Initialize a mana pool for a specific owner.
    pub fn initialize(env: Env, owner: Address, max_mana: u32, regen_rate: u32) {
        owner.require_auth();
        let pool = ManaPool {
            owner: owner.clone(),
            max_mana,
            current_mana: max_mana,
            regen_rate,
            last_updated: env.ledger().timestamp(),
        };
        env.storage().instance().set(&MANA_DATA, &pool);
    }

    /// Internal function to calculate regenerated mana since last update.
    fn calculate_regen(env: &Env, pool: &ManaPool) -> u32 {
        let now = env.ledger().timestamp();
        let seconds_passed = now.saturating_sub(pool.last_updated);
        
        // regen_rate is mana per minute. 
        // regen = (seconds_passed / 60) * regen_rate
        let regenerated = (seconds_passed as u32 / 60) * pool.regen_rate;
        
        let new_mana = pool.current_mana.saturating_add(regenerated);
        if new_mana > pool.max_mana {
            pool.max_mana
        } else {
            new_mana
        }
    }

    /// Get current mana status, including auto-regeneration computation.
    pub fn get_status(env: Env) -> ManaStatus {
        let pool: ManaPool = env.storage().instance().get(&MANA_DATA).expect("Mana pool not initialized");
        let current_mana = Self::calculate_regen(&env, &pool);

        ManaStatus {
            current_mana,
            max_mana: pool.max_mana,
            regen_rate: pool.regen_rate,
            last_updated: pool.last_updated,
        }
    }

    /// Spend a specific amount of mana.
    pub fn spend_mana(env: Env, amount: u32) -> u32 {
        let mut pool: ManaPool = env.storage().instance().get(&MANA_DATA).expect("Mana pool not initialized");
        pool.owner.require_auth();

        let current_mana = Self::calculate_regen(&env, &pool);
        if current_mana < amount {
            panic!("Not enough mana!");
        }

        pool.current_mana = current_mana.saturating_sub(amount);
        pool.last_updated = env.ledger().timestamp();
        
        env.storage().instance().set(&MANA_DATA, &pool);
        pool.current_mana
    }

    /// Instantly recharge mana.
    pub fn recharge(env: Env, amount: u32) -> u32 {
        let mut pool: ManaPool = env.storage().instance().get(&MANA_DATA).expect("Mana pool not initialized");
        pool.owner.require_auth();

        let current_mana = Self::calculate_regen(&env, &pool);
        pool.current_mana = current_mana.saturating_add(amount);
        
        if pool.current_mana > pool.max_mana {
            pool.current_mana = pool.max_mana;
        }
        
        pool.last_updated = env.ledger().timestamp();
        env.storage().instance().set(&MANA_DATA, &pool);
        pool.current_mana
    }
}