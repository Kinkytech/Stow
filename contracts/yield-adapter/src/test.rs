#![cfg(test)]
//! Test skeleton. Each `#[ignore]`d test is a placeholder for a contributor.
//!
//! Pattern: register the contract, register a SEP-41 mock token
//! (`StellarAssetClient` from `soroban_sdk::testutils`), initialize, then
//! exercise the entrypoint. Mirrors `savings-vault::test`'s harness shape —
//! see that module if a helper here needs a fuller reference example.
//!
//! Tests that exercise a strategy (`harvest`, `migrate_strategy`, ...) will
//! additionally need a minimal mock strategy contract implementing the
//! interface documented in `README.md` under "Strategy interface"; building
//! that mock is issue-worthy on its own (see the module doc below) and does
//! not exist yet, so those tests cannot be un-ignored until it does.

use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, testutils::Events as _, Address, Env, IntoVal,
    Symbol,
};

use crate::error::Error;
use crate::{YieldAdapter, YieldAdapterClient};

// ---------------------------------------------------------------------------
// Mock strategy — a minimal real contract implementing the "Strategy
// interface" documented in README.md (`deposit`, `withdraw`, `balance`), so
// #245/#246's event-publisher tests can exercise register/activate/migrate
// and harvest/fee flows end to end. A fuller-featured mock (configurable
// simulated yield curves, failure injection, etc.) is tracked separately as
// issue #251; this is deliberately the minimum needed to make THIS PR's own
// new tests real.
#[contract]
pub struct MockStrategy;

#[contractimpl]
impl MockStrategy {
    pub fn deposit(env: Env, from: Address, amount: i128) {
        let key = (Symbol::new(&env, "bal"), from);
        let current: i128 = env.storage().instance().get(&key).unwrap_or(0);
        env.storage().instance().set(&key, &(current + amount));
    }

    pub fn withdraw(env: Env, to: Address, amount: i128) {
        let key = (Symbol::new(&env, "bal"), to);
        let current: i128 = env.storage().instance().get(&key).unwrap_or(0);
        env.storage().instance().set(&key, &(current - amount));
    }

    pub fn balance(env: Env, of: Address) -> i128 {
        let key = (Symbol::new(&env, "bal"), of);
        env.storage().instance().get(&key).unwrap_or(0)
    }

    /// Test-only: simulate yield/loss by directly setting the reported
    /// balance, independent of actual deposit/withdraw calls.
    pub fn set_reported_balance(env: Env, of: Address, amount: i128) {
        let key = (Symbol::new(&env, "bal"), of);
        env.storage().instance().set(&key, &amount);
    }
}

fn setup_mock_strategy(env: &Env) -> Address {
    env.register(MockStrategy, ())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup(env: &Env) -> YieldAdapterClient {
    let contract_id = env.register(YieldAdapter, ());
    YieldAdapterClient::new(env, &contract_id)
}

/// Full setup: adapter + SEP-41 mock token + admin + treasury.
///
/// Returns `(client, admin, treasury, token_address)`.
fn setup_with_token(env: &Env) -> (YieldAdapterClient, Address, Address, Address) {
    let client = setup(env);
    let admin = Address::generate(env);
    let treasury = Address::generate(env);
    let token_admin = Address::generate(env);

    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_id.address();

    client.initialize(&admin, &treasury, &token_address);

    (client, admin, treasury, token_address)
}

// ---------------------------------------------------------------------------
// Placeholder stubs — one per contributor issue
// ---------------------------------------------------------------------------

#[test]
fn initialize_sets_admin_treasury_and_token() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, treasury, token) = setup_with_token(&env);
    assert_eq!(client.admin(), admin);
    assert_eq!(client.treasury(), treasury);
    assert_eq!(client.token(), token);
    assert_eq!(client.total_shares(), 0);
    // Not extended to also assert `total_assets() == 0` (per this issue's
    // "if needed" wording): `accounting::total_assets` still calls out to
    // the active-strategy balance-reporting interface documented in
    // README.md's "Strategy interface", which is not implemented yet and
    // is out of scope for this issue — see #245/#246/#247 disclosure.
}

#[test]
fn admin_treasury_token_error_before_initialize() {
    let env = Env::default();
    let client = setup(&env);

    assert_eq!(client.try_admin(), Err(Ok(Error::NotInitialized)));
    assert_eq!(client.try_treasury(), Err(Ok(Error::NotInitialized)));
    assert_eq!(client.try_token(), Err(Ok(Error::NotInitialized)));
}

#[test]
#[ignore = "TODO(issue): implement deposit::deposit + accounting::convert_to_shares"]
fn deposit_mints_shares_proportional_to_exchange_rate() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _treasury, _token) = setup_with_token(&env);
    let _user = Address::generate(&env);
    // On the very first deposit, shares must be minted 1:1 with assets.
    todo!("deposit `amount`, assert `get_position(user).shares == amount`");
}

#[test]
#[ignore = "TODO(issue): implement withdraw::request_withdraw + claim_withdraw"]
fn withdraw_round_trip_returns_correct_assets() {
    todo!("deposit, request_withdraw the full position, advance past cooldown, claim_withdraw, assert payout == deposit");
}

#[test]
#[ignore = "TODO(issue): implement withdraw cooldown enforcement"]
fn claim_before_cooldown_elapsed_rejected() {
    todo!("request_withdraw, immediately try_claim_withdraw, assert Error::CooldownNotElapsed");
}

#[test]
#[ignore = "TODO(issue): implement withdraw::cancel_withdraw"]
fn cancel_withdraw_returns_shares_to_owner() {
    todo!("request_withdraw, cancel_withdraw, assert position shares restored");
}

#[test]
#[ignore = "TODO(issue): implement harvest::harvest — needs a mock strategy contract"]
fn harvest_increases_exchange_rate_for_depositors() {
    todo!("deposit, simulate strategy yield, harvest, assert exchange_rate() increased");
}

#[test]
#[ignore = "TODO(issue): implement harvest::apply_performance_fee"]
fn performance_fee_taken_only_on_positive_yield() {
    todo!("harvest a positive-yield report, assert fees_accrued() == yield * fee_bps / 10_000");
}

#[test]
#[ignore = "TODO(issue): implement harvest loss handling (no fee on loss)"]
fn loss_reduces_exchange_rate_without_charging_fee() {
    todo!("harvest a negative-yield report, assert exchange_rate() decreased and fees_accrued() unchanged");
}

#[test]
#[ignore = "TODO(issue): auth review — require_auth on all mutating entrypoints"]
fn unauthorized_access_rejected() {
    todo!("for each mutating entrypoint, call without the required signer's auth and assert rejection");
}

#[test]
#[ignore = "TODO(issue): implement withdraw::request_withdraw NotFound path"]
fn withdraw_more_shares_than_owned_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _treasury, _token) = setup_with_token(&env);
    let owner = Address::generate(&env);

    let result = client.try_request_withdraw(&owner, &1i128);
    assert_eq!(
        result,
        Err(Ok(Error::NotFound)),
        "a request_withdraw from an owner with no position must fail with NotFound, not panic",
    );
}

#[test]
#[ignore = "TODO(issue): implement strategy::migrate_strategy — needs two mock strategies"]
fn strategy_migration_preserves_total_assets() {
    todo!(
        "register two mock strategies, deposit, migrate_strategy, assert total_assets() unchanged"
    );
}

#[test]
#[ignore = "TODO(issue): property test — share/asset rounding never allows value extraction"]
fn share_rounding_never_allows_value_extraction() {
    todo!("proptest: for arbitrary sequences of deposit/request_withdraw/claim_withdraw, assert sum of payouts never exceeds sum of deposits plus harvested yield");
}

#[test]
fn register_strategy_rejects_duplicate_address() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _treasury, _token) = setup_with_token(&env);
    let strategy_address = setup_mock_strategy(&env);

    client.register_strategy(
        &admin,
        &strategy_address,
        &soroban_sdk::String::from_str(&env, "mock"),
    );
    let result = client.try_register_strategy(
        &admin,
        &strategy_address,
        &soroban_sdk::String::from_str(&env, "mock-again"),
    );
    assert_eq!(result, Err(Ok(Error::StrategyAlreadyRegistered)));
}

#[test]
#[ignore = "TODO(issue): implement admin::set_paused narrower blocklist"]
fn paused_blocks_mutations_but_not_claim_withdraw() {
    todo!("pause, assert deposit/request_withdraw/harvest all reject with Error::Paused, then assert an in-flight claim_withdraw still succeeds");
}

#[test]
fn initialize_twice_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, treasury, token) = setup_with_token(&env);

    let second_admin = Address::generate(&env);
    let result = client.try_initialize(&second_admin, &treasury, &token);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));

    // The original values must survive the rejected re-initialization.
    assert_eq!(client.admin(), admin);
}

// ---------------------------------------------------------------------------
// #245 — typed publishers for strategy events
// ---------------------------------------------------------------------------

#[test]
fn register_strategy_emits_strategy_registered_with_id_and_address() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _treasury, _token) = setup_with_token(&env);
    let strategy_address = setup_mock_strategy(&env);

    let id = client.register_strategy(
        &admin,
        &strategy_address,
        &soroban_sdk::String::from_str(&env, "mock"),
    );

    let now = env.ledger().timestamp();
    let events = env.events().all();
    let (contract_id, topics, data) = events.last().unwrap().clone();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (crate::events::TOPIC_STRATEGY_REGISTERED,).into_val(&env);
    let decoded: (u64, Address, u64) = soroban_sdk::TryFromVal::try_from_val(&env, &data).unwrap();

    assert_eq!(contract_id, client.address);
    assert_eq!(topics, expected_topics);
    assert_eq!(decoded, (id, strategy_address, now));
}

#[test]
fn set_active_strategy_emits_strategy_changed_with_from_none() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _treasury, _token) = setup_with_token(&env);
    let strategy_address = setup_mock_strategy(&env);
    let id = client.register_strategy(
        &admin,
        &strategy_address,
        &soroban_sdk::String::from_str(&env, "mock"),
    );

    client.set_active_strategy(&admin, &id);

    let now = env.ledger().timestamp();
    let events = env.events().all();
    let (contract_id, topics, data) = events.last().unwrap().clone();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (crate::events::TOPIC_STRATEGY_CHANGED,).into_val(&env);
    let decoded: (Option<u64>, u64, u64) =
        soroban_sdk::TryFromVal::try_from_val(&env, &data).unwrap();

    assert_eq!(contract_id, client.address);
    assert_eq!(topics, expected_topics);
    assert_eq!(decoded, (None, id, now));
}

#[test]
fn migrate_strategy_emits_strategy_changed_with_from_and_to() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _treasury, _token) = setup_with_token(&env);
    let strategy_a = setup_mock_strategy(&env);
    let strategy_b = setup_mock_strategy(&env);
    let id_a = client.register_strategy(
        &admin,
        &strategy_a,
        &soroban_sdk::String::from_str(&env, "a"),
    );
    let id_b = client.register_strategy(
        &admin,
        &strategy_b,
        &soroban_sdk::String::from_str(&env, "b"),
    );
    client.set_active_strategy(&admin, &id_a);

    client.migrate_strategy(&admin, &id_b);

    let now = env.ledger().timestamp();
    let events = env.events().all();
    let (contract_id, topics, data) = events.last().unwrap().clone();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (crate::events::TOPIC_STRATEGY_CHANGED,).into_val(&env);
    let decoded: (Option<u64>, u64, u64) =
        soroban_sdk::TryFromVal::try_from_val(&env, &data).unwrap();

    assert_eq!(contract_id, client.address);
    assert_eq!(topics, expected_topics);
    assert_eq!(decoded, (Some(id_a), id_b, now));
}

#[test]
fn emergency_withdraw_all_emits_strategy_changed_with_to_none() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _treasury, _token) = setup_with_token(&env);
    let strategy_address = setup_mock_strategy(&env);
    let id = client.register_strategy(
        &admin,
        &strategy_address,
        &soroban_sdk::String::from_str(&env, "mock"),
    );
    client.set_active_strategy(&admin, &id);

    client.emergency_withdraw_all(&admin);

    let now = env.ledger().timestamp();
    let events = env.events().all();
    let (contract_id, topics, data) = events.last().unwrap().clone();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (crate::events::TOPIC_STRATEGY_CHANGED,).into_val(&env);
    let decoded: (Option<u64>, Option<u64>, u64) =
        soroban_sdk::TryFromVal::try_from_val(&env, &data).unwrap();

    assert_eq!(contract_id, client.address);
    assert_eq!(topics, expected_topics);
    assert_eq!(decoded, (Some(id), None, now));
}

#[test]
fn deregister_strategy_emits_strategy_deregistered() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _treasury, _token) = setup_with_token(&env);
    let strategy_address = setup_mock_strategy(&env);
    let id = client.register_strategy(
        &admin,
        &strategy_address,
        &soroban_sdk::String::from_str(&env, "mock"),
    );

    client.deregister_strategy(&admin, &id);

    let now = env.ledger().timestamp();
    let events = env.events().all();
    let (contract_id, topics, data) = events.last().unwrap().clone();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (crate::events::TOPIC_STRATEGY_DEREGISTERED,).into_val(&env);
    let decoded: (u64, u64) = soroban_sdk::TryFromVal::try_from_val(&env, &data).unwrap();

    assert_eq!(contract_id, client.address);
    assert_eq!(topics, expected_topics);
    assert_eq!(decoded, (id, now));
}

#[test]
fn deregister_strategy_rejects_the_active_strategy() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _treasury, _token) = setup_with_token(&env);
    let strategy_address = setup_mock_strategy(&env);
    let id = client.register_strategy(
        &admin,
        &strategy_address,
        &soroban_sdk::String::from_str(&env, "mock"),
    );
    client.set_active_strategy(&admin, &id);

    let result = client.try_deregister_strategy(&admin, &id);
    assert_eq!(result, Err(Ok(Error::StrategyActive)));
}

// ---------------------------------------------------------------------------
// #246 — typed publishers for harvest/fee events
// ---------------------------------------------------------------------------

#[test]
fn harvest_emits_harvested_with_signed_delta_and_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _treasury, _token) = setup_with_token(&env);
    let strategy_address = setup_mock_strategy(&env);
    let mock = MockStrategyClient::new(&env, &strategy_address);
    let id = client.register_strategy(
        &admin,
        &strategy_address,
        &soroban_sdk::String::from_str(&env, "mock"),
    );
    client.set_active_strategy(&admin, &id);
    client.set_performance_fee_bps(&admin, &1_000); // 10%

    // Simulate 1_000_000 of yield accrued in the strategy.
    mock.set_reported_balance(&client.address, &1_000_000);

    let caller = Address::generate(&env);
    let delta = client.harvest(&caller);
    assert_eq!(delta, 1_000_000);

    let now = env.ledger().timestamp();
    let events = env.events().all();
    let (contract_id, topics, data) = events.last().unwrap().clone();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (crate::events::TOPIC_HARVESTED,).into_val(&env);
    let decoded: (Address, i128, i128, u64) =
        soroban_sdk::TryFromVal::try_from_val(&env, &data).unwrap();

    assert_eq!(contract_id, client.address);
    assert_eq!(topics, expected_topics);
    assert_eq!(decoded, (caller, 1_000_000, 100_000, now));
    assert_eq!(client.fees_accrued(), 100_000);
}

#[test]
fn harvest_on_a_loss_emits_zero_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _treasury, _token) = setup_with_token(&env);
    let strategy_address = setup_mock_strategy(&env);
    let mock = MockStrategyClient::new(&env, &strategy_address);
    let id = client.register_strategy(
        &admin,
        &strategy_address,
        &soroban_sdk::String::from_str(&env, "mock"),
    );
    client.set_active_strategy(&admin, &id);
    client.set_performance_fee_bps(&admin, &1_000);

    mock.set_reported_balance(&client.address, &1_000_000);
    client.harvest(&Address::generate(&env));
    // Now simulate a loss on the next report.
    mock.set_reported_balance(&client.address, &400_000);

    let caller = Address::generate(&env);
    let delta = client.harvest(&caller);
    assert_eq!(delta, -600_000);

    let now = env.ledger().timestamp();
    let events = env.events().all();
    let (contract_id, topics, data) = events.last().unwrap().clone();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (crate::events::TOPIC_HARVESTED,).into_val(&env);
    let decoded: (Address, i128, i128, u64) =
        soroban_sdk::TryFromVal::try_from_val(&env, &data).unwrap();

    assert_eq!(contract_id, client.address);
    assert_eq!(topics, expected_topics);
    assert_eq!(decoded, (caller, -600_000, 0, now));
    assert_eq!(
        client.fees_accrued(),
        100_000,
        "a loss must never charge a fee or touch fees already accrued from a prior positive harvest"
    );
}

#[test]
fn withdraw_fees_emits_fee_collected_and_pays_treasury() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, treasury, token) = setup_with_token(&env);
    let strategy_address = setup_mock_strategy(&env);
    let mock = MockStrategyClient::new(&env, &strategy_address);
    let id = client.register_strategy(
        &admin,
        &strategy_address,
        &soroban_sdk::String::from_str(&env, "mock"),
    );
    client.set_active_strategy(&admin, &id);
    client.set_performance_fee_bps(&admin, &1_000);
    mock.set_reported_balance(&client.address, &1_000_000);
    client.harvest(&Address::generate(&env));

    // The adapter needs real tokens on hand to actually pay the fee out —
    // harvest only moves accounting, not real balances (the strategy
    // interface's own deposit/withdraw calls are what move real funds; this
    // mock never actually holds the vault token, so fund the adapter
    // directly to isolate withdraw_fees' own behavior).
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&client.address, &100_000);

    let caller = Address::generate(&env);
    let swept = client.withdraw_fees(&caller);

    let now = env.ledger().timestamp();
    let events = env.events().all();
    let (contract_id, topics, data) = events.last().unwrap().clone();
    let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> =
        (crate::events::TOPIC_FEE_COLLECTED,).into_val(&env);
    let decoded: (Address, i128, u64) = soroban_sdk::TryFromVal::try_from_val(&env, &data).unwrap();

    assert_eq!(swept, 100_000);
    assert_eq!(client.fees_accrued(), 0);
    let treasury_balance = soroban_sdk::token::Client::new(&env, &token).balance(&treasury);
    assert_eq!(treasury_balance, 100_000);
    assert_eq!(contract_id, client.address);
    assert_eq!(topics, expected_topics);
    assert_eq!(decoded, (caller, 100_000, now));
}

#[test]
fn withdraw_fees_rejects_when_nothing_accrued() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _treasury, _token) = setup_with_token(&env);

    let result = client.try_withdraw_fees(&Address::generate(&env));
    assert_eq!(result, Err(Ok(Error::NoFeesAccrued)));
}

// ---------------------------------------------------------------------------
// #247 — error-code audit: implemented entrypoints must never panic on an
// expected failure path (only unimplemented!() stubs should panic, and only
// because they are genuinely not this PR's scope).
// ---------------------------------------------------------------------------

#[test]
fn set_active_strategy_rejects_unknown_strategy_id() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _treasury, _token) = setup_with_token(&env);
    let result = client.try_set_active_strategy(&admin, &999);
    assert_eq!(result, Err(Ok(Error::StrategyNotFound)));
}

#[test]
fn set_active_strategy_rejects_when_already_active() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _treasury, _token) = setup_with_token(&env);
    let strategy_address = setup_mock_strategy(&env);
    let id = client.register_strategy(
        &admin,
        &strategy_address,
        &soroban_sdk::String::from_str(&env, "mock"),
    );
    client.set_active_strategy(&admin, &id);

    let result = client.try_set_active_strategy(&admin, &id);
    assert_eq!(result, Err(Ok(Error::StrategyAlreadyActive)));
}

#[test]
fn register_strategy_requires_admin_auth() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _treasury, _token) = setup_with_token(&env);
    let strategy_address = setup_mock_strategy(&env);
    let stranger = Address::generate(&env);

    let result = client.try_register_strategy(
        &stranger,
        &strategy_address,
        &soroban_sdk::String::from_str(&env, "mock"),
    );
    assert_eq!(result, Err(Ok(Error::Unauthorized)));
}

#[test]
fn harvest_rejects_with_no_active_strategy() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _treasury, _token) = setup_with_token(&env);

    let result = client.try_harvest(&Address::generate(&env));
    assert_eq!(result, Err(Ok(Error::StrategyNotFound)));
}

#[test]
fn set_performance_fee_bps_rejects_above_cap() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _treasury, _token) = setup_with_token(&env);

    let result = client.try_set_performance_fee_bps(&admin, &3_001);
    assert_eq!(result, Err(Ok(Error::FeeTooHigh)));
}

#[test]
fn harvest_respects_the_configured_interval() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _treasury, _token) = setup_with_token(&env);
    let strategy_address = setup_mock_strategy(&env);
    let mock = MockStrategyClient::new(&env, &strategy_address);
    let id = client.register_strategy(
        &admin,
        &strategy_address,
        &soroban_sdk::String::from_str(&env, "mock"),
    );
    client.set_active_strategy(&admin, &id);
    env.as_contract(&client.address, || {
        env.storage()
            .instance()
            .set(&crate::types::DataKey::HarvestInterval, &3600u64);
    });

    client.harvest(&Address::generate(&env));

    mock.set_reported_balance(&client.address, &2_000_000);
    let result = client.try_harvest(&Address::generate(&env));
    assert_eq!(result, Err(Ok(Error::HarvestTooSoon)));
}
