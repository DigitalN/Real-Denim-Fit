//! The default currency, from the system's region.

/// The currency for a locale such as `en-US` or `fr_CA`, if the region is
/// one we know. Euro countries map to EUR.
pub fn for_locale(locale: &str) -> Option<&'static str> {
    let region = locale
        .split(['-', '_', '.', '@'])
        .skip(1)
        .find(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_alphabetic()))?
        .to_ascii_uppercase();
    const EURO: &[&str] = &[
        "AT", "BE", "HR", "CY", "EE", "FI", "FR", "DE", "GR", "IE", "IT", "LV", "LT", "LU", "MT", "NL", "PT", "SK",
        "SI", "ES",
    ];
    if EURO.contains(&region.as_str()) {
        return Some("EUR");
    }
    Some(match region.as_str() {
        "US" => "USD",
        "GB" => "GBP",
        "CA" => "CAD",
        "AU" => "AUD",
        "NZ" => "NZD",
        "JP" => "JPY",
        "CH" | "LI" => "CHF",
        "SE" => "SEK",
        "NO" => "NOK",
        "DK" => "DKK",
        "IS" => "ISK",
        "PL" => "PLN",
        "CZ" => "CZK",
        "HU" => "HUF",
        "RO" => "RON",
        "BG" => "BGN",
        "TR" => "TRY",
        "IL" => "ILS",
        "KR" => "KRW",
        "CN" => "CNY",
        "HK" => "HKD",
        "SG" => "SGD",
        "IN" => "INR",
        "ID" => "IDR",
        "MY" => "MYR",
        "PH" => "PHP",
        "TH" => "THB",
        "MX" => "MXN",
        "BR" => "BRL",
        "ZA" => "ZAR",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::for_locale;

    #[test]
    fn regions_map_to_their_currency() {
        assert_eq!(for_locale("en-US"), Some("USD"));
        assert_eq!(for_locale("en_GB"), Some("GBP"));
        assert_eq!(for_locale("fr-CA"), Some("CAD"));
        assert_eq!(for_locale("de-DE"), Some("EUR"));
        assert_eq!(for_locale("zh-Hant-HK"), Some("HKD"));
        assert_eq!(for_locale("en"), None);
    }
}
