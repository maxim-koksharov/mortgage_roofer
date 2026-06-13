use crate::error::MortgageError;
use crate::models::EuriborTenor;
use chrono::NaiveDate;

/// Fetches the latest Euribor rate from ECB/Frankfurt for the given tenor.
/// Returns the annual rate as a percentage (e.g., 4.5 for 4.5%).
pub fn fetch_euribor(tenor: EuriborTenor) -> Result<f64, MortgageError> {
    let series = format!("M.EXT_EURIBOR_{}", tenor.as_str().to_uppercase());
    let url = format!(
        "https://sdw-wsrest.ecb.europa.eu/service/data/BSI/{}?lastNObservations=1&format=json",
        series
    );

    let resp = ureq::get(&url)
        .header("Accept", "application/json")
        .call()
        .map_err(|e| MortgageError::EuriborFetchError(format!("HTTP error: {}", e)))?;

    let body_text = resp
        .into_body()
        .read_to_string()
        .map_err(|e| MortgageError::EuriborFetchError(format!("Read body error: {}", e)))?;
    let json: serde_json::Value = serde_json::from_str(&body_text)
        .map_err(|e| MortgageError::EuriborFetchError(format!("JSON parse error: {}", e)))?;

    let value = json
        .get("dataSets")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("series"))
        .and_then(|v| v.get("0:0:0:0:0"))
        .and_then(|v| v.get("observations"))
        .and_then(|v| v.get("0"))
        .and_then(|v| v.get(0))
        .and_then(|v| v.as_f64())
        .ok_or_else(|| {
            MortgageError::EuriborFetchError(
                "Could not extract Euribor value from response".to_string(),
            )
        })?;

    Ok(value)
}

/// Cached Euribor store.
#[derive(Debug, Clone, Default)]
pub struct EuriborCache {
    fetched_at: Option<NaiveDate>,
    tenor: Option<EuriborTenor>,
    rate: Option<f64>,
}

impl EuriborCache {
    /// Get rate, fetching if necessary. Caches for the current calendar day.
    pub fn get_or_fetch(&mut self, tenor: EuriborTenor) -> Result<f64, MortgageError> {
        let today = chrono::Local::now().date_naive();
        if let (Some(cached_tenor), Some(cached_date), Some(cached_rate)) =
            (self.tenor, self.fetched_at, self.rate)
            && cached_tenor == tenor
            && cached_date == today
        {
            return Ok(cached_rate);
        }
        let rate = fetch_euribor(tenor)?;
        self.tenor = Some(tenor);
        self.fetched_at = Some(today);
        self.rate = Some(rate);
        Ok(rate)
    }
}
