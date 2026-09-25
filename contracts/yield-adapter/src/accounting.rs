//! Share/asset accounting.
//!
//! Positions are denominated in shares, not raw asset amounts, so that
//! `harvest` can grow (or, on a strategy loss, shrink) every depositor's
//! value proportionally by moving a single exchange rate instead of
//! rewriting every `Position` record.
//!
//! ## Rounding direction
//!
//! Rounding must always favor the adapter over the depositor, or repeated
//! deposit/withdraw cycles at the rounding boundary let a depositor extract
//! more value than they put in:
//!
//! - `convert_to_shares` (used by `deposit`): round **down**. A depositor
//!   who deposits an amount that doesn't divide evenly gets slightly fewer
//!   shares, never more.
//! - `convert_to_assets` (used by `request_withdraw`, `claim_withdraw`,
//!   `cancel_withdraw`'s re-mint): round **down**. A withdrawal that doesn't
//!   divide evenly pays out slightly less, never more.

use soroban_sdk::{vec, Env, IntoVal, Symbol, Val};

use crate::error::Error;
use crate::storage;
use crate::types::{DataKey, StrategyInfo};

/// `(a * b) / divisor`, rounding down (`i128` division already truncates
/// toward zero, and both operands here are always non-negative, so that is
/// equivalent to floor). Falls back to a divide-then-multiply-plus-remainder
/// path when the direct product would overflow `i128`, so a large share
/// supply and a large total_assets can still be combined without an
/// intermediate overflow, at the cost of losing at most one unit of
/// precision in that fallback path.
fn mul_div_down(a: i128, b: i128, divisor: i128) -> Result<i128, Error> {
    if divisor <= 0 {
        return Err(Error::Overflow);
    }
    if let Some(product) = a.checked_mul(b) {
        return product.checked_div(divisor).ok_or(Error::Overflow);
    }
    let q = a / divisor;
    let r = a % divisor;
    let part1 = q.checked_mul(b).ok_or(Error::Overflow)?;
    let part2 = r
        .checked_mul(b)
        .ok_or(Error::Overflow)?
        .checked_div(divisor)
        .ok_or(Error::Overflow)?;
    part1.checked_add(part2).ok_or(Error::Overflow)
}

/// The active strategy's address, if any, resolved via `DataKey::ActiveStrategy`
/// (a strategy id) -> `DataKey::Strategy(id)` (the full record).
fn active_strategy_address(env: &Env) -> Option<soroban_sdk::Address> {
    let id: u64 = env.storage().instance().get(&DataKey::ActiveStrategy)?;
    env.storage()
        .persistent()
        .get::<DataKey, StrategyInfo>(&DataKey::Strategy(id))
        .map(|info| info.address)
}

/// Total vault-token value the adapter is responsible for: its own idle
/// balance plus whatever is currently deployed in the active strategy
/// (queried live via the strategy's own `balance` entrypoint — see
/// `README.md`'s "Strategy interface"). `0` before `initialize`.
pub fn total_assets(env: &Env) -> i128 {
    let Some(token_address) = storage::get_token(env) else {
        return 0;
    };
    let idle = storage::token_client(env, &token_address).balance(&env.current_contract_address());

    let deployed = match active_strategy_address(env) {
        Some(strategy_address) => {
            let args: soroban_sdk::Vec<Val> =
                vec![env, env.current_contract_address().into_val(env)];
            env.invoke_contract::<i128>(&strategy_address, &Symbol::new(env, "balance"), args)
        }
        None => 0,
    };

    idle.saturating_add(deployed)
}

/// Total shares outstanding across all positions. Backed by the
/// `DataKey::TotalShares` running total, not a scan over `Position` entries.
pub fn total_shares(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalShares)
        .unwrap_or(0)
}

/// Convert an asset amount to shares at the current exchange rate, rounding
/// down. On the very first deposit (when `total_shares() == 0`), shares are
/// minted 1:1 with assets.
pub fn convert_to_shares(env: &Env, assets: i128) -> Result<i128, Error> {
    if assets < 0 {
        return Err(Error::InvalidAmount);
    }
    let shares = total_shares(env);
    if shares == 0 {
        return Ok(assets);
    }
    let assets_total = total_assets(env);
    if assets_total == 0 {
        // A live share supply backed by zero assets is insolvent, not a
        // bootstrap state; falling back to a 1:1 rate here would let a new
        // depositor mint shares against value that was never contributed.
        return Err(Error::Overflow);
    }
    mul_div_down(assets, shares, assets_total)
}

/// Convert a share amount to assets at the current exchange rate, rounding
/// down.
pub fn convert_to_assets(env: &Env, shares: i128) -> Result<i128, Error> {
    if shares < 0 {
        return Err(Error::InvalidAmount);
    }
    let total_shares_outstanding = total_shares(env);
    if total_shares_outstanding == 0 {
        return Ok(0);
    }
    let assets_total = total_assets(env);
    mul_div_down(shares, assets_total, total_shares_outstanding)
}

/// The current exchange rate, expressed as `(total_assets, total_shares)` so
/// callers can compute a ratio at whatever precision they need without this
/// crate picking a fixed-point scale for them.
pub fn exchange_rate(env: &Env) -> (i128, i128) {
    (total_assets(env), total_shares(env))
}
