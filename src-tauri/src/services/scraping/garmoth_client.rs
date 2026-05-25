use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::core::errors::AppError;

const API_BASE: &str = "https://api.garmoth.com";

#[derive(Deserialize, Serialize)]
pub struct GarmothPreset {
    pub id: u64,
    pub class: u32,
    pub title: Option<String>,
    pub created_at: i64,
    pub image_1: Option<String>,
    pub image_2: Option<String>,
    pub user_nickname: Option<String>,
    pub character_name: Option<String>,
    pub downloads: u32,
    pub views: u32,
    pub likes: u32,
}

pub struct GarmothClient {
    api_client: reqwest::Client,
}

impl GarmothClient {
    pub fn new(cf_clearance: &str) -> Self {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:151.0) Gecko/20100101 Firefox/151.0";

        let mut api_headers = HeaderMap::new();
        api_headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
        api_headers.insert(header::REFERER, HeaderValue::from_static("https://garmoth.com/"));
        api_headers.insert(header::ORIGIN, HeaderValue::from_static("https://garmoth.com"));
        api_headers.insert("lang", HeaderValue::from_static("us"));
        api_headers.insert("region", HeaderValue::from_static("sa"));
        if !cf_clearance.is_empty() {
            let cookie = format!("cf_clearance={}", cf_clearance);
            if let Ok(val) = HeaderValue::from_str(&cookie) {
                api_headers.insert(header::COOKIE, val);
            }
        }

        let api_client = reqwest::Client::builder()
            .user_agent(ua)
            .default_headers(api_headers)
            .build()
            .expect("failed to build API client");

        Self { api_client }
    }

    pub async fn fetch_preset(&self, id: u64) -> Result<(GarmothPreset, serde_json::Value), AppError> {
        let url = format!("{}/api/beauty-album/preset/{}", API_BASE, id);
        let bytes = self.api_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Scrape(format!("API request failed: {}", e)))?
            .error_for_status()
            .map_err(|e| AppError::Scrape(format!("API error status: {}", e)))?
            .bytes()
            .await
            .map_err(|e| AppError::Scrape(format!("API read failed: {}", e)))?;
        let raw: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| AppError::Scrape(format!("API parse failed: {}", e)))?;
        let typed: GarmothPreset = serde_json::from_value(raw.clone())
            .map_err(|e| AppError::Scrape(format!("API parse failed: {}", e)))?;
        Ok((typed, raw))
    }

    pub async fn fetch_popular(&self, class_id: Option<u32>, days: &str, region: &str) -> Result<Vec<serde_json::Value>, AppError> {
        let class_param = class_id.map_or_else(|| "all".to_string(), |id| id.to_string());
        let url = format!(
            "{}/api/beauty-album/search-advanced?class={}&past={}&region={}&sort=popular&limit=100",
            API_BASE, class_param, days, region
        );
        let bytes = self.api_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Scrape(format!("Popular API request failed: {}", e)))?
            .error_for_status()
            .map_err(|e| AppError::Scrape(format!("Popular API error status: {}", e)))?
            .bytes()
            .await
            .map_err(|e| AppError::Scrape(format!("Popular API read failed: {}", e)))?;
        let val: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| AppError::Scrape(format!("Popular API parse failed: {}", e)))?;
        let arr = val["presets"]["data"].as_array().cloned().unwrap_or_default();
        Ok(arr)
    }
}
