use std::sync::Arc;

use crate::domain::entity::analysis::StockSuggestion;
use crate::domain::error::DomainError;
use crate::domain::port::ai::AiAnalysisPort;

pub struct SuggestStocksUseCase {
    ai: Arc<dyn AiAnalysisPort>,
}

impl SuggestStocksUseCase {
    pub fn new(ai: Arc<dyn AiAnalysisPort>) -> Self {
        Self { ai }
    }

    pub async fn execute(
        &self,
        limit: usize,
    ) -> Result<Vec<StockSuggestion>, DomainError> {
        tracing::info!(limit, "fetching stock suggestions");
        self.ai.suggest_stocks(limit).await
    }
}
