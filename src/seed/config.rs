use std::collections::HashMap;

use crate::config::WalletSeedDefaults;
use crate::error::AppError;
use crate::models::WalletSeedOverride;

#[derive(Debug, Clone)]
pub struct ResolvedSeed {
    pub country: String,
    pub currency: String,
    pub amount: i64,
    pub source: String,
}

pub fn resolve_seeds_for_countries(
    enabled_countries: &[String],
    defaults: &WalletSeedDefaults,
    overrides: &[WalletSeedOverride],
) -> Result<Vec<ResolvedSeed>, AppError> {
    let override_map: HashMap<String, &WalletSeedOverride> = overrides
        .iter()
        .map(|o| (o.country.clone(), o))
        .collect();

    let mut resolved = Vec::new();

    for country in enabled_countries {
        if let Some(ov) = override_map.get(country) {
            resolved.push(ResolvedSeed {
                country: country.clone(),
                currency: ov.currency.clone(),
                amount: ov.amount,
                source: "override".into(),
            });
        } else if let Some(def) = defaults.get(country) {
            resolved.push(ResolvedSeed {
                country: country.clone(),
                currency: def.currency.clone(),
                amount: def.amount,
                source: "default".into(),
            });
        } else {
            tracing::warn!(country = %country, "no wallet seed default; creating zero balance wallet");
            resolved.push(ResolvedSeed {
                country: country.clone(),
                currency: guess_currency(country),
                amount: 0,
                source: "default".into(),
            });
        }
    }

    Ok(resolved)
}

fn guess_currency(country: &str) -> String {
    match country {
        "ZM" => "ZMW".into(),
        "US" => "USD".into(),
        "GB" => "GBP".into(),
        "CA" => "CAD".into(),
        _ => "USD".into(),
    }
}
