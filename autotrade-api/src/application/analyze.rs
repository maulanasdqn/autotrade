use std::sync::Arc;

use crate::domain::entity::analysis::StockAnalysis;
use crate::domain::error::DomainError;
use crate::domain::port::ai::AiAnalysisPort;
use crate::domain::port::market::MarketDataPort;
use crate::domain::port::repository::AnalysisRepository;

pub struct AnalyzeStockUseCase {
    market: Arc<dyn MarketDataPort>,
    ai: Arc<dyn AiAnalysisPort>,
    repo: Arc<dyn AnalysisRepository>,
}

impl AnalyzeStockUseCase {
    pub fn new(
        market: Arc<dyn MarketDataPort>,
        ai: Arc<dyn AiAnalysisPort>,
        repo: Arc<dyn AnalysisRepository>,
    ) -> Self {
        Self { market, ai, repo }
    }

    pub async fn execute(
        &self,
        symbol: &str,
    ) -> Result<StockAnalysis, DomainError> {
        let prices = self.market.get_price_history(symbol, 90).await?;
        let fundamental = self.market.get_fundamentals(symbol).await?;

        tracing::info!(symbol, "running AI analysis");
        let analysis =
            self.ai.analyze_stock(symbol, &prices, &fundamental).await?;

        self.repo.save(&analysis).await?;
        Ok(analysis)
    }
}
