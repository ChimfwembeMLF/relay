mod config;

pub use config::{resolve_seeds_for_countries, ResolvedSeed};

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::config::WalletSeedDefaults;
use crate::db::wallet_seeds::{create_wallet_with_balance, record_wallet_seed_event};
use crate::error::AppError;
use crate::models::WalletSeedOverride;

pub async fn seed_system_wallets(
    pool: &PgPool,
    system_id: Uuid,
    enabled_countries: &[String],
    defaults: &WalletSeedDefaults,
    overrides: &[WalletSeedOverride],
) -> Result<usize, AppError> {
    validate_overrides(enabled_countries, overrides)?;
    let seeds = resolve_seeds_for_countries(enabled_countries, defaults, overrides)?;
    let mut count = 0;

    for seed in seeds {
        let wallet = create_wallet_with_balance(
            pool,
            system_id,
            &seed.country,
            &seed.currency,
            seed.amount,
        )
        .await?;

        record_wallet_seed_event(
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

pub async fn seed_system_wallets_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    system_id: Uuid,
    enabled_countries: &[String],
    defaults: &WalletSeedDefaults,
    overrides: &[WalletSeedOverride],
) -> Result<usize, AppError> {
    validate_overrides(enabled_countries, overrides)?;
    let seeds = resolve_seeds_for_countries(enabled_countries, defaults, overrides)?;
    let mut count = 0;

    for seed in seeds {
        let wallet = create_wallet_with_balance(
            &mut **tx,
            system_id,
            &seed.country,
            &seed.currency,
            seed.amount,
        )
        .await?;

        record_wallet_seed_event(
            &mut **tx,
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

fn validate_overrides(
    enabled_countries: &[String],
    overrides: &[WalletSeedOverride],
) -> Result<(), AppError> {
    for ov in overrides {
        if !enabled_countries.contains(&ov.country) {
            return Err(AppError::Validation(format!(
                "wallet seed override for {} not in enabled_countries",
                ov.country
            )));
        }
    }
    Ok(())
}
