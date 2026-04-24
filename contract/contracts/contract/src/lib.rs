#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, String, Vec, Map,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

const ADMIN_KEY: &str       = "admin";
const REPORTERS_KEY: &str   = "reporters";
const FLAGS_KEY: &str       = "flags";

// ─── Data Types ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlagRecord {
    /// The flagged wallet address
    pub address:    Address,
    /// Who reported it
    pub reporter:   Address,
    /// Short reason code (e.g. "PHISHING", "RUGPULL", "SCAM")
    pub reason:     String,
    /// Ledger sequence at which this flag was recorded
    pub timestamp:  u32,
    /// Whether an admin has removed / cleared this flag
    pub active:     bool,
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct FraudRegistry;

#[contractimpl]
impl FraudRegistry {

    // ── Initialization ───────────────────────────────────────────────────────

    /// Deploy: set the contract administrator.
    /// Must be called exactly once.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&symbol_short!("admin")) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&symbol_short!("admin"), &admin);

        // Start with an empty reporter allow-list
        let reporters: Vec<Address> = Vec::new(&env);
        env.storage().instance().set(&symbol_short!("reporters"), &reporters);

        // Empty flag map  address (as string key) → FlagRecord
        let flags: Map<Address, FlagRecord> = Map::new(&env);
        env.storage().instance().set(&symbol_short!("flags"), &flags);
    }

    // ── Admin helpers ────────────────────────────────────────────────────────

    fn require_admin(env: &Env) -> Address {
        let admin: Address = env.storage().instance().get(&symbol_short!("admin")).unwrap();
        admin.require_auth();
        admin
    }

    fn get_reporters(env: &Env) -> Vec<Address> {
        env.storage().instance().get(&symbol_short!("reporters")).unwrap()
    }

    fn get_flags(env: &Env) -> Map<Address, FlagRecord> {
        env.storage().instance().get(&symbol_short!("flags")).unwrap()
    }

    // ── Reporter management (admin-only) ─────────────────────────────────────

    /// Grant a trusted reporter the right to flag addresses.
    pub fn add_reporter(env: Env, reporter: Address) {
        Self::require_admin(&env);
        let mut reporters = Self::get_reporters(&env);
        if !reporters.contains(&reporter) {
            reporters.push_back(reporter);
            env.storage().instance().set(&symbol_short!("reporters"), &reporters);
        }
    }

    /// Revoke a reporter's rights.
    pub fn remove_reporter(env: Env, reporter: Address) {
        Self::require_admin(&env);
        let reporters = Self::get_reporters(&env);
        let mut updated: Vec<Address> = Vec::new(&env);
        for r in reporters.iter() {
            if r != reporter {
                updated.push_back(r);
            }
        }
        env.storage().instance().set(&symbol_short!("reporters"), &updated);
    }

    /// Check if an address is an authorized reporter.
    pub fn is_reporter(env: Env, addr: Address) -> bool {
        Self::get_reporters(&env).contains(&addr)
    }

    // ── Flagging ─────────────────────────────────────────────────────────────

    /// Flag a suspicious address.  Caller must be an authorized reporter.
    pub fn flag_address(
        env:      Env,
        reporter: Address,
        target:   Address,
        reason:   String,
    ) {
        reporter.require_auth();

        // Only allow-listed reporters may flag
        if !Self::get_reporters(&env).contains(&reporter) {
            panic!("unauthorized reporter");
        }

        let mut flags = Self::get_flags(&env);
        if flags.contains_key(target.clone()) {
            let existing = flags.get(target.clone()).unwrap();
            if existing.active {
                panic!("address already flagged");
            }
        }

        let record = FlagRecord {
            address:   target.clone(),
            reporter:  reporter.clone(),
            reason,
            timestamp: env.ledger().sequence(),
            active:    true,
        };

        flags.set(target, record);
        env.storage().instance().set(&symbol_short!("flags"), &flags);
    }

    // ── Querying ─────────────────────────────────────────────────────────────

    /// Returns true if the address is currently flagged as fraudulent.
    pub fn is_flagged(env: Env, addr: Address) -> bool {
        let flags = Self::get_flags(&env);
        if let Some(record) = flags.get(addr) {
            return record.active;
        }
        false
    }

    /// Returns the full flag record for an address (panics if not found).
    pub fn get_flag(env: Env, addr: Address) -> FlagRecord {
        let flags = Self::get_flags(&env);
        flags.get(addr).expect("address not flagged")
    }

    // ── Admin moderation ─────────────────────────────────────────────────────

    /// Admin can clear (deactivate) a flag — useful for false positives.
    pub fn clear_flag(env: Env, addr: Address) {
        Self::require_admin(&env);
        let mut flags = Self::get_flags(&env);
        let mut record = flags.get(addr.clone()).expect("flag not found");
        record.active = false;
        flags.set(addr, record);
        env.storage().instance().set(&symbol_short!("flags"), &flags);
    }

    /// Transfer admin rights to a new address.
    pub fn transfer_admin(env: Env, new_admin: Address) {
        Self::require_admin(&env);
        new_admin.require_auth();
        env.storage().instance().set(&symbol_short!("admin"), &new_admin);
    }

    /// Returns the current admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&symbol_short!("admin")).unwrap()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn setup() -> (Env, FraudRegistryClient<'static>, Address) {
        let env    = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, FraudRegistry);
        let client     = FraudRegistryClient::new(&env, &contract_id);
        let admin      = Address::generate(&env);
        client.initialize(&admin);
        (env, client, admin)
    }

    #[test]
    fn test_flag_and_query() {
        let (env, client, admin) = setup();
        let reporter = Address::generate(&env);
        let target   = Address::generate(&env);

        client.add_reporter(&reporter);
        assert!(client.is_reporter(&reporter));

        client.flag_address(
            &reporter,
            &target,
            &String::from_str(&env, "PHISHING"),
        );

        assert!(client.is_flagged(&target));
        let record = client.get_flag(&target);
        assert_eq!(record.reason, String::from_str(&env, "PHISHING"));
        assert!(record.active);
    }

    #[test]
    fn test_clear_flag() {
        let (env, client, _admin) = setup();
        let reporter = Address::generate(&env);
        let target   = Address::generate(&env);

        client.add_reporter(&reporter);
        client.flag_address(&reporter, &target, &String::from_str(&env, "SCAM"));
        assert!(client.is_flagged(&target));

        client.clear_flag(&target);
        assert!(!client.is_flagged(&target));
    }

    #[test]
    #[should_panic(expected = "unauthorized reporter")]
    fn test_unauthorized_reporter_panics() {
        let (env, client, _admin) = setup();
        let bad_actor = Address::generate(&env);
        let target    = Address::generate(&env);

        client.flag_address(&bad_actor, &target, &String::from_str(&env, "RUGPULL"));
    }
}