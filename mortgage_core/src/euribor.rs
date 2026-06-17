use crate::error::MortgageError;
use crate::models::{EuriborPoint, EuriborTenor};
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
    /// Fetches historical Euribor rates for a given date range.
    /// Returns a Vec<EuriborPoint> with dates and rates for the period.
    pub fn fetch_historical(
        &mut self,
        tenor: EuriborTenor,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<EuriborPoint>, MortgageError> {
        let pid = provider_id(tenor);
        let start = start_date.format("%Y-%m-%d");
        let end = end_date.format("%Y-%m-%d");
        let url = format!(
            "https://data-api.ecb.europa.eu/service/data/FM/M.U2.EUR.RT.MM.{}.?startPeriod={}&endPeriod={}&format=jsondata",
            pid, start, end
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

        let observations = json["dataSets"][0]["series"]["0:0:0:0:0:0:0"]["observations"]
            .as_object()
            .ok_or_else(|| {
                MortgageError::EuriborFetchError(
                    "Could not find observations in ECB response".to_string(),
                )
            })?;

        let time_values = json["structure"]["dimensions"]["observation"][0]["values"]
            .as_array()
            .ok_or_else(|| {
                MortgageError::EuriborFetchError(
                    "Could not find time values in ECB response".to_string(),
                )
            })?;

        let mut points = Vec::new();
        for (key, obs) in observations {
            let idx: usize = key.parse().unwrap_or(usize::MAX);
            if idx >= time_values.len() {
                continue;
            }
            let rate = obs[0].as_f64().ok_or_else(|| {
                MortgageError::EuriborFetchError(format!(
                    "Could not parse rate for observation {}",
                    key
                ))
            })?;
            let date_str = time_values[idx]["start"].as_str().unwrap_or("");
            if date_str.len() < 10 {
                continue;
            }
            if let Ok(date) = NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d") {
                points.push(EuriborPoint {
                    date_from: date,
                    rate,
                });
            }
        }

        points.sort_by_key(|p| p.date_from);
        if points.is_empty() {
            return Err(MortgageError::EuriborFetchError(
                "No Euribor data found for the specified period".to_string(),
            ));
        }
        Ok(points)
    }

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
