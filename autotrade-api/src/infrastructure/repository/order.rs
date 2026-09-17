use std::sync::Mutex;

use async_trait::async_trait;

use crate::domain::entity::order::Order;
use crate::domain::error::DomainError;
use crate::domain::port::repository::OrderRepository;

pub struct InMemoryOrderRepo {
    data: Mutex<Vec<Order>>,
}

impl InMemoryOrderRepo {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl OrderRepository for InMemoryOrderRepo {
    async fn save(&self, order: &Order) -> Result<(), DomainError> {
        let mut data = self.data.lock().unwrap();
        if let Some(existing) = data.iter_mut().find(|o| o.id == order.id) {
            *existing = order.clone();
        } else {
            data.push(order.clone());
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &str,
    ) -> Result<Option<Order>, DomainError> {
        let data = self.data.lock().unwrap();
        let result = data
            .iter()
            .find(|o| o.id.to_string() == id)
            .cloned();
        Ok(result)
    }

    async fn find_open_orders(&self) -> Result<Vec<Order>, DomainError> {
        let data = self.data.lock().unwrap();
        let open = data.iter().filter(|o| !o.is_terminal()).cloned().collect();
        Ok(open)
    }
}
