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
//!
//! TODO(issue): implement both directions and add the property test in
//! `test.rs` that asserts no sequence of deposit/withdraw calls can increase
//! total assets extracted beyond what was deposited plus harvested yield.

use soroban_sdk::Env;

use crate::error::Error;
use crate::types::DataKey;

/// Total vault-token value the adapter is responsible for: its own idle
/// balance plus whatever is currently deployed in the active strategy
/// (queried via the strategy's own balance-reporting entrypoint — see
/// `README.md`'s "Strategy interface").
///
/// TODO(issue): implement.
pub fn total_assets(_env: &Env) -> i128 {
    unimplemented!("accounting: total_assets")
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
///
/// TODO(issue): implement.
pub fn convert_to_shares(_env: &Env, _assets: i128) -> Result<i128, Error> {
    unimplemented!("accounting: convert_to_shares")
}

/// Convert a share amount to assets at the current exchange rate, rounding
/// down.
///
/// TODO(issue): implement.
pub fn convert_to_assets(_env: &Env, _shares: i128) -> Result<i128, Error> {
    unimplemented!("accounting: convert_to_assets")
}

/// The current exchange rate, expressed as `(total_assets, total_shares)` so
/// callers can compute a ratio at whatever precision they need without this
/// crate picking a fixed-point scale for them.
///
/// TODO(issue): implement.
pub fn exchange_rate(_env: &Env) -> (i128, i128) {
    unimplemented!("accounting: exchange_rate")
}
