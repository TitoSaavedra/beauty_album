use std::time::Duration;

use playwright_rs::{
    Browser, BrowserContext, BrowserContextOptions, GotoOptions, LaunchOptions, Playwright,
    WaitUntil, install_browsers,
};

use crate::errors::AppError;

// On Windows release builds, hide any new console windows that playwright's
// Node.js driver creates (they appear because the driver is a console-subsystem
// process and the parent Tauri app is a GUI-subsystem app with no console).
#[cfg(all(target_os = "windows", not(debug_assertions)))]
mod console_hider {
    #[link(name = "user32")]
    extern "system" {
        fn EnumWindows(lpEnumFunc: unsafe extern "system" fn(isize, isize) -> i32, lParam: isize) -> i32;
        fn GetClassNameW(hwnd: isize, lpClassName: *mut u16, nMaxCount: i32) -> i32;
        fn IsWindowVisible(hwnd: isize) -> i32;
        fn ShowWindow(hwnd: isize, nCmdShow: i32) -> i32;
    }

    unsafe extern "system" fn collect_cb(hwnd: isize, lparam: isize) -> i32 {
        if IsWindowVisible(hwnd) == 0 { return 1; }
        let mut class = [0u16; 64];
        let len = GetClassNameW(hwnd, class.as_mut_ptr(), 64) as usize;
        // "ConsoleWindowClass" — 18 UTF-16 code units
        if len == 18 {
            const TARGET: [u16; 18] = [
                0x43, 0x6F, 0x6E, 0x73, 0x6F, 0x6C, 0x65, 0x57,
                0x69, 0x6E, 0x64, 0x6F, 0x77, 0x43, 0x6C, 0x61, 0x73, 0x73,
            ];
            if class[..18] == TARGET {
                let vec = &mut *(lparam as *mut Vec<isize>);
                vec.push(hwnd);
            }
        }
        1
    }

    pub fn snapshot() -> Vec<isize> {
        let mut hwnds: Vec<isize> = Vec::new();
        unsafe { EnumWindows(collect_cb, &mut hwnds as *mut _ as isize); }
        hwnds
    }

    pub fn hide_new(before: &[isize]) {
        let after = snapshot();
        for hwnd in after {
            if !before.contains(&hwnd) {
                unsafe { ShowWindow(hwnd, 0); } // SW_HIDE
            }
        }
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
        let hide_task = {
            let before = console_hider::snapshot();
            tokio::task::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(150)).await;
                    console_hider::hide_new(&before);
                }
            })
        };

        install_browsers(Some(&["chromium"]))
            .await
            .map_err(|e| AppError::Scrape(format!("browser install: {:?}", e)))?;

        let playwright = Playwright::launch()
            .await
            .map_err(|e| AppError::Scrape(format!("playwright: {:?}", e)))?;

        #[cfg(all(target_os = "windows", not(debug_assertions)))]
        hide_task.abort();

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
