use std::sync::Arc;
use tokio::sync::RwLock;

use crate::application::analyze::AnalyzeStockUseCase;
use crate::application::autotrade::AutoTradeUseCase;
use crate::application::suggest::SuggestStocksUseCase;
use crate::domain::port::broker::BrokerPort;
use crate::domain::port::repository::TradeRuleRepository;

#[derive(Clone)]
pub struct AppState {
    pub analyze: Arc<AnalyzeStockUseCase>,
    pub suggest: Arc<SuggestStocksUseCase>,
    pub autotrade: Arc<AutoTradeUseCase>,
    pub rules: Arc<dyn TradeRuleRepository>,
    pub broker: Arc<dyn BrokerPort>,
    pub token: Arc<RwLock<String>>,
}
