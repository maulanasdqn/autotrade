use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Stock not found: {0}")]
    StockNotFound(String),

    #[error("Insufficient balance: need {need}, have {have}")]
    InsufficientBalance {
        need: rust_decimal::Decimal,
        have: rust_decimal::Decimal,
    },

    #[error("Trade execution failed: {0}")]
    TradeExecutionFailed(String),

    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Validation error: {0}")]
    Validation(String),
}
