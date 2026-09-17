use std::sync::Mutex;

use async_trait::async_trait;

use crate::domain::entity::analysis::StockAnalysis;
use crate::domain::error::DomainError;
use crate::domain::port::repository::AnalysisRepository;

pub struct InMemoryAnalysisRepo {
    data: Mutex<Vec<StockAnalysis>>,
}

impl InMemoryAnalysisRepo {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl AnalysisRepository for InMemoryAnalysisRepo {
    async fn save(&self, analysis: &StockAnalysis) -> Result<(), DomainError> {
        self.data.lock().unwrap().push(analysis.clone());
        Ok(())
    }

    async fn find_latest(
        &self,
        symbol: &str,
    ) -> Result<Option<StockAnalysis>, DomainError> {
        let data = self.data.lock().unwrap();
        let result = data
            .iter()
            .filter(|a| a.symbol == symbol)
            .max_by_key(|a| a.analyzed_at)
            .cloned();
        Ok(result)
    }

    async fn find_all_latest(
        &self,
        limit: usize,
    ) -> Result<Vec<StockAnalysis>, DomainError> {
        let data = self.data.lock().unwrap();
        let mut sorted = data.clone();
        sorted.sort_by(|a, b| b.analyzed_at.cmp(&a.analyzed_at));
        sorted.truncate(limit);
        Ok(sorted)
    }
}
