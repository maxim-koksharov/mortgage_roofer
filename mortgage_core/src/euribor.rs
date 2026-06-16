use crate::error::MortgageError;
use crate::models::EuriborTenor;
use chrono::NaiveDate;

fn provider_id(tenor: EuriborTenor) -> &'static str {
    match tenor {
        EuriborTenor::OneMonth => "EURIBOR1MD_",
        EuriborTenor::ThreeMonths => "EURIBOR3MD_",
        EuriborTenor::SixMonths => "EURIBOR6MD_",
        EuriborTenor::TwelveMonths => "EURIBOR1YD_",
    }
}

static AGENT: std::sync::LazyLock<ureq::Agent> = std::sync::LazyLock::new(|| {
    let config = ureq::Agent::config_builder()
        .timeout_connect(Some(std::time::Duration::from_secs(5)))
        .timeout_global(Some(std::time::Duration::from_secs(10)))
        .build();
    config.new_agent()
});

/// Fetches the latest Euribor rate from ECB (FM dataset) for the given tenor.
/// Returns the annual rate as a percentage (e.g., 4.5 for 4.5%).
pub fn fetch_euribor(tenor: EuriborTenor) -> Result<f64, MortgageError> {
    let pid = provider_id(tenor);
    let url = format!(
        "https://data-api.ecb.europa.eu/service/data/FM/M.U2.EUR.RT.MM.{}.?lastNObservations=1&format=jsondata",
        pid
    );

    let resp = AGENT
        .get(&url)
        .header("Accept", "application/json")
        .call()
        .map_err(|e| {
            MortgageError::EuriborFetchError(format!("Failed to connect to ECB API: {}", e))
        })?;

    let status = resp.status();
    let body_text = resp.into_body().read_to_string().map_err(|e| {
        MortgageError::EuriborFetchError(format!("Failed to read ECB response body: {}", e))
    })?;

    let json: serde_json::Value = serde_json::from_str(&body_text).map_err(|e| {
        MortgageError::EuriborFetchError(format!(
            "Failed to parse ECB JSON (HTTP {}): {}",
            status, e
        ))
    })?;

    let value = json
        .get("dataSets")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("series"))
        .and_then(|v| v.get("0:0:0:0:0:0:0"))
        .and_then(|v| v.get("observations"))
        .and_then(|v| v.get("0"))
        .and_then(|v| v.get(0))
        .and_then(|v| v.as_f64())
        .ok_or_else(|| {
            MortgageError::EuriborFetchError(
                "Could not extract Euribor value from ECB response".to_string(),
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
