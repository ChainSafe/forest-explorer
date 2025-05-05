use anyhow::{ensure, Context as _};
use thirtyfour::prelude::*;
use url::Url;

pub static WEBPAGE_URL: &str = "http://127.0.0.1:8787";
pub static CHROME_DRIVER_URL: &str = "http://127.0.0.1:9515";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut cap = DesiredCapabilities::chrome();
    cap.set_headless()?;
    let chrome = WebDriver::new(CHROME_DRIVER_URL, cap).await?;
    chrome.goto(WEBPAGE_URL).await?;
    assert_faucet_reachable().await?;

    Ok(())
}

/// Checks if various pages of the faucet are reachable. It uses the `reqwest` library underneath,
/// given the `WebDriver` is not able to check the status of the page (the page would return 500 and
/// `WebDriver` would not report an issue).
async fn assert_faucet_reachable() -> anyhow::Result<()> {
    let faucet_pages = ["/faucet", "/faucet/mainnet", "/faucet/calibnet"];
    let mut unreachable = Vec::new();
    for page in faucet_pages {
        let url = Url::parse(WEBPAGE_URL)?.join(page)?;
        let response = reqwest::get(url)
            .await
            .context("Failed to reach {page} endpoint")?;
        if !response.status().is_success() {
            unreachable.push(page);
        }
    }

    ensure!(
        unreachable.is_empty(),
        "The following faucet pages are unreachable: {:?}",
        unreachable,
    );
    Ok(())
}
