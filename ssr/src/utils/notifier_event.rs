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
    /// User's email associated with this event
    pub email: String,
    pub step_name: Option<String>, // None for pipeline-level events.
    pub event_type: NotifierEventType,
}

impl NotifierEvent {
    pub fn new_step_start(
        order_id: String,
        email: String,
        step_name: String,
        corr_id: Uuid,
    ) -> Self {
        Self {
            event_id: uuidv7::create(),
            correlation_id: corr_id,
            timestamp: Utc::now(),
            order_id,
            email,
            step_name: Some(step_name),
            event_type: NotifierEventType::OnStepStart,
        }
    }

    /// Internal helper to format a topic string with the given components
    fn format_topic(step: &str, event_type: Option<&str>, order_id: &str, email: &str) -> String {
        format!(
            "step:{}:{}:booking:{}:email:{}",
            step,                      // part 2
            event_type.unwrap_or("*"), // part 3
            order_id,                  // part 5
            email                      // part 7
                                       // Keywords are part 1 (step), 4 (booking), 6 (email)
        )
    }

    /// Constructs a topic string based on the event details.
    ///
    /// Format: `step:<step_name>:<event_type>:booking:<booking_id>:email:<email>`
    /// Supports filtering by:
    /// - Booking ID → `step:*:*:booking:<id>:email:*`
    /// - Email → `step:*:booking:*:email:<email>`
    /// - Both → `step:*:booking:<id>:email:<email>`
    pub fn topic(&self) -> String {
        let step = self.step_name.as_deref().unwrap_or("*");
        Self::format_topic(
            step,
            Some(format_event_type(&self.event_type)),
            &self.order_id,
            &self.email,
        )
    }

    /// Creates a topic pattern for subscribing to events.
    ///
    /// # Arguments
    /// * `email` - Optional email to filter by
    /// * `order_id` - Optional order_id to filter by
    /// * `step_name` - Optional step name to filter by
    ///
    /// # Examples
    /// ```
    /// // Filter by email only
    /// let topic = NotifierEvent::make_topic_pattern(Some("user@example.com"), None, None);
    /// assert_eq!(topic, "step:*:*:booking:*:email:user@example.com");
    ///
    /// // Filter by order_id only
    /// let topic = NotifierEvent::make_topic_pattern(None, Some("ABC123"), None);
    /// assert_eq!(topic, "step:*:*:booking:ABC123:email:*");
    ///
    /// // Filter by step_name
    /// let topic = NotifierEvent::make_topic_pattern(None, None, Some("payment"));
    /// assert_eq!(topic, "step:payment:*:booking:*:email:*");
    ///
    /// // Filter by all
    /// let topic = NotifierEvent::make_topic_pattern(Some("user@example.com"), Some("ABC123"), Some("payment"));
    /// assert_eq!(topic, "step:payment:*:booking:ABC123:email:user@example.com");
    /// ```
    pub fn make_topic_pattern(
        email: Option<&str>,
        order_id: Option<&str>,
        step_name: Option<&str>,
    ) -> String {
        Self::format_topic(
            step_name.unwrap_or("*"),
            None,
            order_id.unwrap_or("*"),
            email.unwrap_or("*"),
        )
    }

    /// Creates a topic pattern for subscribing to all events within the 'step' category.
    /// Pattern: `step:*:*:booking:*:email:*`
    pub fn subscribe_all_steps_pattern() -> String {
        Self::make_topic_pattern(None, None, None)
    }

    /// Creates a topic pattern for subscribing to events by email.
    /// Pattern: `step:*:*:booking:*:email:{email}`
    pub fn subscribe_by_email_pattern(email: &str) -> String {
        Self::make_topic_pattern(Some(email), None, None)
    }

    /// Creates a topic pattern for subscribing to events by order ID.
    /// Pattern: `step:*:*:booking:{order_id}:email:*`
    pub fn subscribe_by_order_id_pattern(order_id: &str) -> String {
        Self::make_topic_pattern(None, Some(order_id), None)
    }

    /// Creates a topic pattern for subscribing to events by email and order ID.
    /// Pattern: `step:*:*:booking:{order_id}:email:{email}`
    pub fn subscribe_by_email_and_order_id_pattern(email: &str, order_id: &str) -> String {
        Self::make_topic_pattern(Some(email), Some(order_id), None)
    }

    /// Creates a topic pattern for subscribing to events by step name.
    /// Pattern: `step:{step_name}:*:booking:*:email:*`
    pub fn subscribe_by_step_name_pattern(step_name: &str) -> String {
        Self::make_topic_pattern(None, None, Some(step_name))
    }

    /// Checks if a topic matches a pattern with wildcards.
    ///
    /// Pattern can contain '*' as wildcards for any segment.
    /// For example:
    /// - "step:*:booking:ABC123:email:*" matches "step:payment:booking:ABC123:email:user@example.com"
    /// - "step:payment:*" matches "step:payment:anything"
    pub fn matches_pattern(pattern: &str, topic: &str) -> bool {
        use tracing::{info, span, Level};
        let span = span!(
            Level::INFO,
            "event_bus_match",
            pattern = pattern,
            topic = topic
        );
        let _enter = span.enter();
        info!(
            "[EVENT_BUS MATCH] pattern: '{}', topic: '{}'",
            pattern, topic
        );

        // Helper to extract logical segments from a topic string
        fn extract_segments(s: &str) -> Option<(String, String, String)> {
            // Expect format: step:<step_name>:<event_type>:booking:<booking_id>:email:<email>
            let mut step_name = None;
            let mut event_type = None;
            let mut booking_id = None;
            let mut email = None;
            let mut iter = s.split(':').peekable();
            while let Some(seg) = iter.next() {
                match seg {
                    "step" => {
                        step_name = iter.next().map(|s| s.to_string());
                        event_type = iter.next().map(|s| s.to_string());
                    }
                    "booking" => {
                        booking_id = iter.next().map(|s| s.to_string());
                    }
                    "email" => {
                        email = iter.next().map(|s| s.to_string());
                    }
                    _ => {}
                }
            }
            match (step_name, event_type, booking_id, email) {
                (Some(step_name), Some(event_type), Some(booking_id), Some(email)) => {
                    Some((format!("{}:{}", step_name, event_type), booking_id, email))
                }
                _ => None,
            }
        }

        let pattern_segs = extract_segments(pattern);
        let topic_segs = extract_segments(topic);

        if pattern_segs.is_none() || topic_segs.is_none() {
            info!("[EVENT_BUS MATCH] Could not extract logical segments for matching");
            return false;
        }

        let (pattern_step, pattern_booking, pattern_email) = pattern_segs.unwrap();
        let (topic_step, topic_booking, topic_email) = topic_segs.unwrap();

        // Match step (with wildcard support)
        let step_match = pattern_step == "*:*"
            || pattern_step
                .split(':')
                .zip(topic_step.split(':'))
                .all(|(p, t)| p == "*" || p == t);
        let booking_match = pattern_booking == "*" || pattern_booking == topic_booking;
        let email_match = pattern_email == "*" || pattern_email == topic_email;

        info!(
            "[EVENT_BUS MATCH] step_match: {}, booking_match: {}, email_match: {}",
            step_match, booking_match, email_match
        );
        let result = step_match && booking_match && email_match;
        if result {
            info!(
                "[EVENT_BUS MATCH] pattern '{}' matches topic '{}'",
                pattern, topic
            );
        } else {
            info!(
                "[EVENT_BUS MATCH] pattern '{}' does NOT match topic '{}'",
                pattern, topic
            );
        }
        result
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscribe_all_steps_pattern() {
        let pattern = NotifierEvent::subscribe_all_steps_pattern();
        assert_eq!(pattern, "step:*:*:booking:*:email:*");
    }

    #[test]
    fn test_subscribe_by_email_pattern() {
        let pattern = NotifierEvent::subscribe_by_email_pattern("user@example.com");
        assert_eq!(pattern, "step:*:*:booking:*:email:user@example.com");
    }

    #[test]
    fn test_subscribe_by_order_id_pattern() {
        let pattern = NotifierEvent::subscribe_by_order_id_pattern("ORDER123");
        assert_eq!(pattern, "step:*:*:booking:ORDER123:email:*");
    }

    #[test]
    fn test_subscribe_by_email_and_order_id_pattern() {
        let pattern =
            NotifierEvent::subscribe_by_email_and_order_id_pattern("user@example.com", "ORDER123");
        assert_eq!(pattern, "step:*:*:booking:ORDER123:email:user@example.com");
    }

    #[test]
    fn test_subscribe_by_step_name_pattern() {
        let pattern = NotifierEvent::subscribe_by_step_name_pattern("payment");
        assert_eq!(pattern, "step:payment:*:booking:*:email:*");
    }
}
