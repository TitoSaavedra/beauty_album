use std::time::Duration;

use playwright_rs::{
    Browser, BrowserContext, BrowserContextOptions, GotoOptions, LaunchOptions, Playwright,
    WaitUntil, install_browsers,
};

use crate::errors::AppError;

pub struct BrowserSession {
    _playwright: Playwright,
    _browser: Browser,
    context: BrowserContext,
}

impl BrowserSession {
    pub async fn new() -> Result<Self, AppError> {
        install_browsers(Some(&["chromium"]))
            .await
            .map_err(|e| AppError::Scrape(format!("browser install: {:?}", e)))?;

        let playwright = Playwright::launch()
            .await
            .map_err(|e| AppError::Scrape(format!("playwright: {:?}", e)))?;

        let browser = playwright
            .chromium()
            .launch_with_options(
                LaunchOptions::default()
                    .headless(false)
                    .args(vec![
                        "--headless=new".to_string(),
                        "--disable-blink-features=AutomationControlled".to_string(),
                    ]),
            )
            .await
            .map_err(|e| AppError::Scrape(format!("launch: {:?}", e)))?;

        let context = browser
            .new_context_with_options(BrowserContextOptions {
                user_agent: Some(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                     (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
                        .to_string(),
                ),
                ..Default::default()
            })
            .await
            .map_err(|e| AppError::Scrape(format!("context: {:?}", e)))?;

        let page = context
            .new_page()
            .await
            .map_err(|e| AppError::Scrape(format!("page: {:?}", e)))?;

        page.goto(
            "https://garmoth.com/",
            Some(GotoOptions {
                wait_until: Some(WaitUntil::DomContentLoaded),
                timeout: Some(Duration::from_secs(90)),
                ..Default::default()
            }),
        )
        .await
        .map_err(|e| AppError::Scrape(format!("navigate: {:?}", e)))?;

        Ok(BrowserSession { _playwright: playwright, _browser: browser, context })
    }

    pub async fn download(&self, url: &str) -> Result<Vec<u8>, AppError> {
        let page = self
            .context
            .new_page()
            .await
            .map_err(|e| AppError::Scrape(format!("dl page: {:?}", e)))?;

        let response = page
            .goto(url, None)
            .await
            .map_err(|e| AppError::Scrape(format!("cdn goto: {:?}", e)))?;

        match response {
            Some(resp) => {
                if resp.status() == 403 {
                    return Err(AppError::CfBlocked);
                }
                resp.body()
                    .await
                    .map_err(|e| AppError::Scrape(format!("dl body: {:?}", e)))
            }
            None => Err(AppError::Scrape(format!("no response for {}", url))),
        }
    }
}
