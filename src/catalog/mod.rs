//! PawaPay-aligned country / MNO catalog (ISO-2 storage, ISO-3 + correspondents for gateway).

#[derive(Debug, Clone, Copy)]
pub struct Mno {
    pub label: &'static str,
    pub correspondent: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct CountryEntry {
    pub iso2: &'static str,
    pub iso3: &'static str,
    pub name: &'static str,
    pub flag: &'static str,
    pub dialing_prefix: &'static str,
    pub currencies: &'static [&'static str],
    pub default_currency: &'static str,
    pub mnos: &'static [Mno],
}

pub const DEFAULT_REGISTER_COUNTRY: &str = "ZM";

pub fn all_countries() -> &'static [CountryEntry] {
    COUNTRIES
}

pub fn country_by_iso2(iso2: &str) -> Option<&'static CountryEntry> {
    let key = iso2.trim().to_uppercase();
    COUNTRIES.iter().find(|c| c.iso2 == key)
}

pub fn providers_for_country(iso2: &str) -> &'static [Mno] {
    country_by_iso2(iso2).map(|c| c.mnos).unwrap_or(&[])
}

pub fn default_currency(iso2: &str) -> Option<&'static str> {
    country_by_iso2(iso2).map(|c| c.default_currency)
}

/// Returns Ok(()) if country/currency/provider are consistent with the catalog.
pub fn validate_country_currency_provider(
    country: &str,
    currency: &str,
    provider: &str,
) -> Result<(), String> {
    let entry = country_by_iso2(country).ok_or_else(|| {
        format!("unsupported country '{}'", country.trim().to_uppercase())
    })?;
    let cur = currency.trim().to_uppercase();
    if !entry.currencies.iter().any(|c| *c == cur) {
        return Err(format!(
            "currency '{}' is not valid for {}",
            cur, entry.name
        ));
    }
    let prov = provider.trim();
    if !entry.mnos.iter().any(|m| m.correspondent == prov) {
        return Err(format!(
            "provider '{}' is not valid for {}",
            prov, entry.name
        ));
    }
    Ok(())
}

pub fn validate_country_currency(country: &str, currency: &str) -> Result<(), String> {
    let entry = country_by_iso2(country).ok_or_else(|| {
        format!("unsupported country '{}'", country.trim().to_uppercase())
    })?;
    let cur = currency.trim().to_uppercase();
    if !entry.currencies.iter().any(|c| *c == cur) {
        return Err(format!(
            "currency '{}' is not valid for {}",
            cur, entry.name
        ));
    }
    Ok(())
}

static COUNTRIES: &[CountryEntry] = &[
    CountryEntry {
        iso2: "BJ",
        iso3: "BEN",
        name: "Benin",
        flag: "🇧🇯",
        dialing_prefix: "229",
        currencies: &["XOF"],
        default_currency: "XOF",
        mnos: &[
            Mno {
                label: "Moov",
                correspondent: "MOOV_BEN",
            },
            Mno {
                label: "MTN",
                correspondent: "MTN_MOMO_BEN",
            },
        ],
    },
    CountryEntry {
        iso2: "CM",
        iso3: "CMR",
        name: "Cameroon",
        flag: "🇨🇲",
        dialing_prefix: "237",
        currencies: &["XAF"],
        default_currency: "XAF",
        mnos: &[
            Mno {
                label: "MTN",
                correspondent: "MTN_MOMO_CMR",
            },
            Mno {
                label: "Orange",
                correspondent: "ORANGE_CMR",
            },
        ],
    },
    CountryEntry {
        iso2: "CI",
        iso3: "CIV",
        name: "Côte d'Ivoire",
        flag: "🇨🇮",
        dialing_prefix: "225",
        currencies: &["XOF"],
        default_currency: "XOF",
        mnos: &[
            Mno {
                label: "MTN",
                correspondent: "MTN_MOMO_CIV",
            },
            Mno {
                label: "Orange",
                correspondent: "ORANGE_CIV",
            },
        ],
    },
    CountryEntry {
        iso2: "CD",
        iso3: "COD",
        name: "Democratic Republic of the Congo",
        flag: "🇨🇩",
        dialing_prefix: "243",
        currencies: &["CDF", "USD"],
        default_currency: "CDF",
        mnos: &[
            Mno {
                label: "Vodacom",
                correspondent: "VODACOM_MPESA_COD",
            },
            Mno {
                label: "Airtel",
                correspondent: "AIRTEL_COD",
            },
            Mno {
                label: "Orange",
                correspondent: "ORANGE_COD",
            },
        ],
    },
    CountryEntry {
        iso2: "GA",
        iso3: "GAB",
        name: "Gabon",
        flag: "🇬🇦",
        dialing_prefix: "241",
        currencies: &["XAF"],
        default_currency: "XAF",
        mnos: &[Mno {
            label: "Airtel",
            correspondent: "AIRTEL_GAB",
        }],
    },
    CountryEntry {
        iso2: "KE",
        iso3: "KEN",
        name: "Kenya",
        flag: "🇰🇪",
        dialing_prefix: "254",
        currencies: &["KES"],
        default_currency: "KES",
        mnos: &[Mno {
            label: "Safaricom",
            correspondent: "MPESA_KEN",
        }],
    },
    CountryEntry {
        iso2: "CG",
        iso3: "COG",
        name: "Republic of the Congo",
        flag: "🇨🇬",
        dialing_prefix: "242",
        currencies: &["XAF"],
        default_currency: "XAF",
        mnos: &[
            Mno {
                label: "Airtel",
                correspondent: "AIRTEL_COG",
            },
            Mno {
                label: "MTN",
                correspondent: "MTN_MOMO_COG",
            },
        ],
    },
    CountryEntry {
        iso2: "RW",
        iso3: "RWA",
        name: "Rwanda",
        flag: "🇷🇼",
        dialing_prefix: "250",
        currencies: &["RWF"],
        default_currency: "RWF",
        mnos: &[
            Mno {
                label: "Airtel",
                correspondent: "AIRTEL_RWA",
            },
            Mno {
                label: "MTN",
                correspondent: "MTN_MOMO_RWA",
            },
        ],
    },
    CountryEntry {
        iso2: "SN",
        iso3: "SEN",
        name: "Senegal",
        flag: "🇸🇳",
        dialing_prefix: "221",
        currencies: &["XOF"],
        default_currency: "XOF",
        mnos: &[
            Mno {
                label: "Free",
                correspondent: "FREE_SEN",
            },
            Mno {
                label: "Orange",
                correspondent: "ORANGE_SEN",
            },
        ],
    },
    CountryEntry {
        iso2: "SL",
        iso3: "SLE",
        name: "Sierra Leone",
        flag: "🇸🇱",
        dialing_prefix: "232",
        currencies: &["SLE"],
        default_currency: "SLE",
        mnos: &[Mno {
            label: "Orange",
            correspondent: "ORANGE_SLE",
        }],
    },
    CountryEntry {
        iso2: "UG",
        iso3: "UGA",
        name: "Uganda",
        flag: "🇺🇬",
        dialing_prefix: "256",
        currencies: &["UGX"],
        default_currency: "UGX",
        mnos: &[
            Mno {
                label: "Airtel",
                correspondent: "AIRTEL_OAPI_UGA",
            },
            Mno {
                label: "MTN",
                correspondent: "MTN_MOMO_UGA",
            },
        ],
    },
    CountryEntry {
        iso2: "ZM",
        iso3: "ZMB",
        name: "Zambia",
        flag: "🇿🇲",
        dialing_prefix: "260",
        currencies: &["ZMW"],
        default_currency: "ZMW",
        mnos: &[
            Mno {
                label: "Airtel",
                correspondent: "AIRTEL_OAPI_ZMB",
            },
            Mno {
                label: "MTN",
                correspondent: "MTN_MOMO_ZMB",
            },
            Mno {
                label: "Zamtel",
                correspondent: "ZAMTEL_ZMB",
            },
        ],
    },
];
