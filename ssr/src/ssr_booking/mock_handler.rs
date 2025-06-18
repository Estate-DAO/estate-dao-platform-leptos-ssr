use super::pipeline::{PipelineDecision, PipelineExecutor, PipelineValidator};
use super::ServerSideBookingEvent;
use crate::utils::notifier::Notifier;
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// --------------------------
// Mock Step for Testing
// --------------------------

#[derive(Debug, Clone)]
pub struct MockStep {
    pub decision: PipelineDecision,
    pub executed: Arc<AtomicBool>,
}

impl Default for MockStep {
    fn default() -> Self {
        Self {
            decision: PipelineDecision::Run,
            executed: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait]
impl PipelineValidator for MockStep {
    async fn validate(&self, _event: &ServerSideBookingEvent) -> Result<PipelineDecision, String> {
        Ok(self.decision.clone())
    }
}

#[async_trait]
impl PipelineExecutor for MockStep {
    async fn execute(
        event: ServerSideBookingEvent,
        _notifier: Option<&Notifier>,
    ) -> Result<ServerSideBookingEvent, String> {
        println!("Executing MockStep");
        Ok(event)
    }
}
