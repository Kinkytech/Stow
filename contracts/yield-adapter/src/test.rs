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

use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::error::Error;
use crate::{YieldAdapter, YieldAdapterClient};

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
#[ignore = "TODO(issue): implement strategy::register_strategy duplicate-address guard"]
fn register_strategy_rejects_duplicate_address() {
    todo!("register a strategy address, register the same address again, assert Error::StrategyAlreadyRegistered");
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
