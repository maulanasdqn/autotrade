mod prompt;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::domain::entity::analysis::{StockAnalysis, StockSuggestion};
use crate::domain::entity::stock::{StockFundamental, StockPrice};
use crate::domain::error::DomainError;
use crate::domain::port::ai::AiAnalysisPort;

pub struct AiService {
    api_url: String,
    api_key: String,
    model: String,
    http: Client,
}

impl AiService {
    pub fn new(api_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            http: Client::new(),
        }
    }

    async fn chat(&self, system: &str, user: &str) -> Result<String, DomainError> {
        let body = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                Message { role: "system".into(), content: system.into() },
                Message { role: "user".into(), content: user.into() },
            ],
            temperature: 0.3,
        };

        let resp = self
            .http
            .post(format!("{}/v1/chat/completions", self.api_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| DomainError::ExternalService(e.to_string()))?;

        let status = resp.status();
        let text = resp.text().await
            .map_err(|e| DomainError::ExternalService(e.to_string()))?;

        if !status.is_success() {
            return Err(DomainError::ExternalService(
                format!("DeepSeek API {status}: {text}")
            ));
        }

        let chat_resp: ChatResponse = serde_json::from_str(&text)
            .map_err(|e| DomainError::AnalysisFailed(e.to_string()))?;

        chat_resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| DomainError::AnalysisFailed("empty response".into()))
    }
}

#[async_trait]
impl AiAnalysisPort for AiService {
    async fn analyze_stock(
        &self,
        symbol: &str,
        prices: &[StockPrice],
        fundamental: &StockFundamental,
    ) -> Result<StockAnalysis, DomainError> {
        let user_msg = prompt::build_analysis_prompt(symbol, prices, fundamental);
        let raw = self.chat(prompt::ANALYSIS_SYSTEM, &user_msg).await?;

        tracing::debug!(symbol, response_len = raw.len(), "DeepSeek responded");
        prompt::parse_analysis(symbol, &raw)
    }

    async fn suggest_stocks(
        &self,
        limit: usize,
    ) -> Result<Vec<StockSuggestion>, DomainError> {
        let user_msg = prompt::build_suggest_prompt(limit);
        let raw = self.chat(prompt::SUGGEST_SYSTEM, &user_msg).await?;

        tracing::debug!(response_len = raw.len(), "DeepSeek suggestions received");
        prompt::parse_suggestions(&raw)
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}
