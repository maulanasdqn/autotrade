use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;

use crate::domain::error::DomainError;
use crate::domain::port::repository::TradeRuleRepository;
use crate::domain::value::TradeRule;

pub struct InMemoryTradeRuleRepo {
    data: Mutex<HashMap<String, TradeRule>>,
}

impl InMemoryTradeRuleRepo {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl TradeRuleRepository for InMemoryTradeRuleRepo {
    async fn save(
        &self,
        symbol: &str,
        rule: &TradeRule,
    ) -> Result<(), DomainError> {
        self.data
            .lock()
            .unwrap()
            .insert(symbol.to_string(), rule.clone());
        Ok(())
    }

    async fn find(
        &self,
        symbol: &str,
    ) -> Result<Option<TradeRule>, DomainError> {
        Ok(self.data.lock().unwrap().get(symbol).cloned())
    }

    async fn find_all(
        &self,
    ) -> Result<Vec<(String, TradeRule)>, DomainError> {
        let data = self.data.lock().unwrap();
        Ok(data.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    }

    async fn delete(&self, symbol: &str) -> Result<(), DomainError> {
        self.data.lock().unwrap().remove(symbol);
        Ok(())
    }
}
