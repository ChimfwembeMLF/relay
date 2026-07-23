/** PawaPay-aligned country / MNO catalog (mirrors src/catalog). */

export type Mno = {
  label: string;
  correspondent: string;
};

export type CountryEntry = {
  iso2: string;
  iso3: string;
  name: string;
  flag: string;
  dialingPrefix: string;
  currencies: string[];
  defaultCurrency: string;
  mnos: Mno[];
};

export const DEFAULT_REGISTER_COUNTRY = "ZM";

export const COUNTRIES: CountryEntry[] = [
  {
    iso2: "BJ",
    iso3: "BEN",
    name: "Benin",
    flag: "🇧🇯",
    dialingPrefix: "229",
    currencies: ["XOF"],
    defaultCurrency: "XOF",
    mnos: [
      { label: "Moov", correspondent: "MOOV_BEN" },
      { label: "MTN", correspondent: "MTN_MOMO_BEN" },
    ],
  },
  {
    iso2: "CM",
    iso3: "CMR",
    name: "Cameroon",
    flag: "🇨🇲",
    dialingPrefix: "237",
    currencies: ["XAF"],
    defaultCurrency: "XAF",
    mnos: [
      { label: "MTN", correspondent: "MTN_MOMO_CMR" },
      { label: "Orange", correspondent: "ORANGE_CMR" },
    ],
  },
  {
    iso2: "CI",
    iso3: "CIV",
    name: "Côte d'Ivoire",
    flag: "🇨🇮",
    dialingPrefix: "225",
    currencies: ["XOF"],
    defaultCurrency: "XOF",
    mnos: [
      { label: "MTN", correspondent: "MTN_MOMO_CIV" },
      { label: "Orange", correspondent: "ORANGE_CIV" },
    ],
  },
  {
    iso2: "CD",
    iso3: "COD",
    name: "Democratic Republic of the Congo",
    flag: "🇨🇩",
    dialingPrefix: "243",
    currencies: ["CDF", "USD"],
    defaultCurrency: "CDF",
    mnos: [
      { label: "Vodacom", correspondent: "VODACOM_MPESA_COD" },
      { label: "Airtel", correspondent: "AIRTEL_COD" },
      { label: "Orange", correspondent: "ORANGE_COD" },
    ],
  },
  {
    iso2: "GA",
    iso3: "GAB",
    name: "Gabon",
    flag: "🇬🇦",
    dialingPrefix: "241",
    currencies: ["XAF"],
    defaultCurrency: "XAF",
    mnos: [{ label: "Airtel", correspondent: "AIRTEL_GAB" }],
  },
  {
    iso2: "KE",
    iso3: "KEN",
    name: "Kenya",
    flag: "🇰🇪",
    dialingPrefix: "254",
    currencies: ["KES"],
    defaultCurrency: "KES",
    mnos: [{ label: "Safaricom", correspondent: "MPESA_KEN" }],
  },
  {
    iso2: "CG",
    iso3: "COG",
    name: "Republic of the Congo",
    flag: "🇨🇬",
    dialingPrefix: "242",
    currencies: ["XAF"],
    defaultCurrency: "XAF",
    mnos: [
      { label: "Airtel", correspondent: "AIRTEL_COG" },
      { label: "MTN", correspondent: "MTN_MOMO_COG" },
    ],
  },
  {
    iso2: "RW",
    iso3: "RWA",
    name: "Rwanda",
    flag: "🇷🇼",
    dialingPrefix: "250",
    currencies: ["RWF"],
    defaultCurrency: "RWF",
    mnos: [
      { label: "Airtel", correspondent: "AIRTEL_RWA" },
      { label: "MTN", correspondent: "MTN_MOMO_RWA" },
    ],
  },
  {
    iso2: "SN",
    iso3: "SEN",
    name: "Senegal",
    flag: "🇸🇳",
    dialingPrefix: "221",
    currencies: ["XOF"],
    defaultCurrency: "XOF",
    mnos: [
      { label: "Free", correspondent: "FREE_SEN" },
      { label: "Orange", correspondent: "ORANGE_SEN" },
    ],
  },
  {
    iso2: "SL",
    iso3: "SLE",
    name: "Sierra Leone",
    flag: "🇸🇱",
    dialingPrefix: "232",
    currencies: ["SLE"],
    defaultCurrency: "SLE",
    mnos: [{ label: "Orange", correspondent: "ORANGE_SLE" }],
  },
  {
    iso2: "UG",
    iso3: "UGA",
    name: "Uganda",
    flag: "🇺🇬",
    dialingPrefix: "256",
    currencies: ["UGX"],
    defaultCurrency: "UGX",
    mnos: [
      { label: "Airtel", correspondent: "AIRTEL_OAPI_UGA" },
      { label: "MTN", correspondent: "MTN_MOMO_UGA" },
    ],
  },
  {
    iso2: "ZM",
    iso3: "ZMB",
    name: "Zambia",
    flag: "🇿🇲",
    dialingPrefix: "260",
    currencies: ["ZMW"],
    defaultCurrency: "ZMW",
    mnos: [
      { label: "Airtel", correspondent: "AIRTEL_OAPI_ZMB" },
      { label: "MTN", correspondent: "MTN_MOMO_ZMB" },
      { label: "Zamtel", correspondent: "ZAMTEL_ZMB" },
    ],
  },
];

export function countryByIso2(iso2: string): CountryEntry | undefined {
  const key = iso2.trim().toUpperCase();
  return COUNTRIES.find((c) => c.iso2 === key);
}

export function countriesForEnabled(enabled: string[]): CountryEntry[] {
  const set = new Set(enabled.map((c) => c.toUpperCase()));
  return COUNTRIES.filter((c) => set.has(c.iso2));
}

export function mnoLabels(iso2: string): string[] {
  return countryByIso2(iso2)?.mnos.map((m) => m.label) ?? [];
}
