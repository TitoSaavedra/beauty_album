use std::time::Duration;

use playwright_rs::{
    Browser, BrowserContext, BrowserContextOptions, GotoOptions, LaunchOptions, Playwright,
    WaitUntil, install_browsers,
};

use crate::errors::AppError;

// On Windows release builds, allocate a hidden console before launching Playwright.
// node.exe (the Playwright driver) is a console-subsystem process; when spawned by a
// GUI-subsystem parent it would normally create a new visible console window. By giving
// our process its own console first (and immediately hiding it), node.exe inherits that
// console instead of opening a new one — works with both ConHost and Windows Terminal.
#[cfg(all(target_os = "windows", not(debug_assertions)))]
mod hidden_console {
    use std::sync::OnceLock;
    static DONE: OnceLock<()> = OnceLock::new();

    #[link(name = "kernel32")]
    extern "system" {
        fn AllocConsole() -> i32;
        fn GetConsoleWindow() -> isize;
    }
    #[link(name = "user32")]
    extern "system" {
        fn ShowWindow(hwnd: isize, nCmdShow: i32) -> i32;
    }

    pub fn ensure_hidden() {
        DONE.get_or_init(|| unsafe {
            if AllocConsole() != 0 {
                let hwnd = GetConsoleWindow();
                if hwnd != 0 {
                    ShowWindow(hwnd, 0); // SW_HIDE
                }
            }
        });
    }
}

pub struct BrowserSession {
    _playwright: Playwright,
    _browser: Browser,
    context: BrowserContext,
}

impl BrowserSession {
    pub async fn new() -> Result<Self, AppError> {
        #[cfg(all(target_os = "windows", not(debug_assertions)))]
        hidden_console::ensure_hidden();

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
