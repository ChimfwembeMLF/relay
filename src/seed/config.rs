use std::collections::HashMap;

use crate::catalog;
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
            continue;
        }

        let amount = defaults.get(country).map(|d| d.amount).unwrap_or(0);

        if let Some(entry) = catalog::country_by_iso2(country) {
            for &currency in entry.currencies {
                resolved.push(ResolvedSeed {
                    country: country.clone(),
                    currency: currency.to_string(),
                    amount,
                    source: "default".into(),
                });
            }
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
    catalog::default_currency(country)
        .unwrap_or("USD")
        .to_string()
}
