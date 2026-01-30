use std::pin::Pin;

use crate::domain::CharonEvent;
use maiko::Meta;

// #[async_trait::async_trait]
pub trait Processor: Send + Sync {
    fn process<'a, 'b>(
        &'a mut self,
        event: CharonEvent,
        meta: Meta,
    ) -> Pin<Box<dyn Future<Output = Vec<CharonEvent>> + Send + 'b>>
    where
        'a: 'b,
        Self: 'b;
}
