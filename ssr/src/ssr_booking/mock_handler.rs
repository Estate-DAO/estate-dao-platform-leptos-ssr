use super::pipeline::{PipelineDecision, PipelineValidator};
use super::ServerSideBookingEvent;
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

#[async_trait]
impl PipelineValidator for MockStep {
    async fn validate(&self, _event: &ServerSideBookingEvent) -> Result<PipelineDecision, String> {
        Ok(self.decision.clone())
    }
}
