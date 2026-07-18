mod config;

pub use config::{resolve_seeds_for_countries, ResolvedSeed};

use sqlx::PgPool;
use uuid::Uuid;

use crate::config::WalletSeedDefaults;
use crate::error::AppError;
use crate::models::WalletSeedOverride;

pub async fn seed_system_wallets(
    pool: &PgPool,
    system_id: Uuid,
    enabled_countries: &[String],
    defaults: &WalletSeedDefaults,
    overrides: &[WalletSeedOverride],
) -> Result<usize, AppError> {
    let seeds = resolve_seeds_for_countries(enabled_countries, defaults, overrides)?;
    let mut count = 0;

    for seed in seeds {
        let wallet = crate::db::queries::create_wallet_with_balance(
            pool,
            system_id,
            &seed.country,
            &seed.currency,
            seed.amount,
        )
        .await?;

        crate::db::queries::record_wallet_seed_event(
            pool,
            system_id,
            wallet.id,
            &seed.country,
            &seed.currency,
            seed.amount,
            &seed.source,
        )
        .await?;

        count += 1;
    }

    Ok(count)
}
