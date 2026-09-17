use std::process::Command;
use tokio::sync::Semaphore;

use crate::domain::error::DomainError;

const BASE_URL: &str = "https://www.idx.co.id";
const REFERER: &str =
    "https://www.idx.co.id/id/data-pasar/data-saham/aktivitas-perdagangan/";
const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
    AppleWebKit/537.36 (KHTML, like Gecko) \
    Chrome/120.0.0.0 Safari/537.36";

#[derive(Clone)]
pub struct IdxClient {
    semaphore: std::sync::Arc<Semaphore>,
}

impl IdxClient {
    pub fn new() -> Self {
        Self {
            semaphore: std::sync::Arc::new(Semaphore::new(1)),
        }
    }

    pub async fn get(&self, path: &str) -> Result<String, DomainError> {
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            DomainError::ExternalService(format!("semaphore: {e}"))
        })?;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let url = format!("{}{}", BASE_URL, path);

        for attempt in 0..3 {
            if attempt > 0 {
                let wait = 5 * attempt;
                tracing::warn!(attempt, wait, "IDX retry after Cloudflare challenge");
                tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
            }
            match Self::fetch(&url).await {
                Ok(body) => return Ok(body),
                Err(e) if attempt < 2 => {
                    tracing::warn!(%e, "IDX request failed");
                }
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }

    async fn fetch(url: &str) -> Result<String, DomainError> {
        let url = url.to_string();
        let output = tokio::task::spawn_blocking(move || {
            Command::new("curl")
                .args([
                    "-s", "--max-time", "15",
                    "-H", &format!("User-Agent: {UA}"),
                    "-H", "Accept: application/json",
                    "-H", "Accept-Language: id-ID,id;q=0.9,en-US;q=0.8,en;q=0.7",
                    "-H", "X-Requested-With: XMLHttpRequest",
                    "-H", &format!("Referer: {REFERER}"),
                    &url,
                ])
                .output()
        })
        .await
        .map_err(|e| DomainError::ExternalService(format!("spawn: {e}")))?
        .map_err(|e| DomainError::ExternalService(format!("curl: {e}")))?;

        let body = String::from_utf8_lossy(&output.stdout);
        let trimmed = body.trim();

        if trimmed.is_empty() {
            return Err(DomainError::ExternalService(
                "IDX returned empty response".into(),
            ));
        }
        if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
            return Err(DomainError::ExternalService(
                "IDX returned Cloudflare challenge".into(),
            ));
        }

        Ok(trimmed.to_string())
    }
}
