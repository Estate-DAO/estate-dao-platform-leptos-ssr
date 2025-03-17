//! Module for defining notifier events and their types.
//!
//! This module provides structures and enums for representing various events
//! that occur during pipeline and step execution. These events are used for
//! notification purposes across the system.

use crate::utils::uuidv7;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

/// All possible notifier event types.
///
/// These represent the different stages and outcomes of pipeline and step execution
/// that can trigger notifications.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum NotifierEventType {
    OnStepStart,
    OnStepCompleted,
    OnStepSkipped,
    OnPipelineStart,
    OnPipelineEnd,
    OnPipelineAbort,
}

type Uuid = String;

/// A structured notifier event with metadata.
///
/// This struct represents a single notification event, containing all relevant
/// information about the event that occurred.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotifierEvent {
    /// Unique identifier for this specific event
    pub event_id: Uuid,
    /// Correlation ID for tracking related events - started in a pipeline.
    // this could be pipeline_id
    pub correlation_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub order_id: String,
    pub step_name: Option<String>, // None for pipeline-level events.
    pub event_type: NotifierEventType,
}

impl NotifierEvent {
    pub fn new_step_start(order_id: String, step_name: String, corr_id: Uuid) -> Self {
        Self {
            event_id: uuidv7::create(),
            correlation_id: corr_id,
            timestamp: Utc::now(),
            order_id,
            step_name: Some(step_name),
            event_type: NotifierEventType::OnStepStart,
        }
    }

    /// Constructs a topic string based on the event details.
    ///
    /// The format is: "booking:{order_id}:step:{step_name}:{event_type}"
    /// If step_name is None, a placeholder (*) is used.
    pub fn topic(&self) -> String {
        let step = self.step_name.as_deref().unwrap_or("*");
        format!(
            "booking:{}:step:{}:{}",
            self.order_id,
            step,
            format_event_type(&self.event_type)
        )
    }
}

/// Formats the event type as a string representation.
fn format_event_type(event_type: &NotifierEventType) -> &'static str {
    match event_type {
        NotifierEventType::OnStepStart => "on_step_start",
        NotifierEventType::OnStepCompleted => "on_step_completed",
        NotifierEventType::OnStepSkipped => "on_step_skipped",
        NotifierEventType::OnPipelineStart => "on_pipeline_start",
        NotifierEventType::OnPipelineEnd => "on_pipeline_end",
        NotifierEventType::OnPipelineAbort => "on_pipeline_abort",
    }
}
