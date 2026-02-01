// SPDX-License-Identifier: GPL-3.0-or-later
use std::pin::Pin;

use crate::domain::CharonEvent;
use maiko::Meta;

pub type ProcessorFuture<'a> = Pin<Box<dyn Future<Output = Vec<CharonEvent>> + Send + 'a>>;

// #[async_trait::async_trait]
pub trait Processor: Send + Sync {
    fn process<'a>(&'a mut self, event: CharonEvent, meta: Meta) -> ProcessorFuture<'a>;
}
