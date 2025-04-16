//! Module for defining notifier events and their types.
//!
//! This module provides structures and enums for representing various events
//! that occur during pipeline and step execution. These events are used for
//! notification purposes across the system.

use crate::utils::uuidv7;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// All possible notifier event types.
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

/// Trait for topic segments that can be serialized into a topic string
pub trait TopicSegment {
    fn segment_key(&self) -> &'static str;
    fn segment_value(&self) -> String;
}

/// Represents a step name segment in a topic
#[derive(Debug, Clone)]
pub struct StepNameSegment {
    pub name: Option<String>,
}

impl TopicSegment for StepNameSegment {
    fn segment_key(&self) -> &'static str {
        "step"
    }

    fn segment_value(&self) -> String {
        self.name.as_deref().unwrap_or("*").to_string()
    }
}

/// Represents a step type segment in a topic
#[derive(Debug, Clone)]
pub struct StepTypeSegment {
    pub event_type: Option<NotifierEventType>,
}

impl TopicSegment for StepTypeSegment {
    fn segment_key(&self) -> &'static str {
        "step_type"
    }

    fn segment_value(&self) -> String {
        self.event_type
            .as_ref()
            .map(format_event_type)
            .unwrap_or("*")
            .to_string()
    }
}

/// Represents an identifier segment in a topic (e.g. booking_id, email, payment_id)
#[derive(Debug, Clone)]
pub struct IdentifierSegment {
    key: &'static str,
    value: Option<String>,
}

impl IdentifierSegment {
    pub fn new(key: &'static str, value: Option<String>) -> Self {
        Self { key, value }
    }
}

impl TopicSegment for IdentifierSegment {
    fn segment_key(&self) -> &'static str {
        self.key
    }

    fn segment_value(&self) -> String {
        self.value.as_deref().unwrap_or("*").to_string()
    }
}

/// A structured notifier event with metadata.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotifierEvent {
    pub event_id: Uuid,
    pub correlation_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub order_id: String,
    pub email: String,
    pub step_name: Option<String>,
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

    /// Creates a topic string from a list of segments
    fn create_topic(segments: &[Box<dyn TopicSegment>]) -> String {
        segments
            .iter()
            .map(|seg| format!("{}:{}", seg.segment_key(), seg.segment_value()))
            .collect::<Vec<_>>()
            .join(":")
    }

    /// Creates a topic string based on the event details
    pub fn topic(&self) -> String {
        let segments: Vec<Box<dyn TopicSegment>> = vec![
            Box::new(StepNameSegment {
                name: self.step_name.clone(),
            }),
            Box::new(StepTypeSegment {
                event_type: Some(self.event_type.clone()),
            }),
            Box::new(IdentifierSegment::new(
                "booking",
                Some(self.order_id.clone()),
            )),
            Box::new(IdentifierSegment::new("email", Some(self.email.clone()))),
        ];
        Self::create_topic(&segments)
    }

    /// Creates a topic pattern for subscribing to events
    pub fn make_topic_pattern(
        email: Option<&str>,
        order_id: Option<&str>,
        step_name: Option<&str>,
    ) -> String {
        let segments: Vec<Box<dyn TopicSegment>> = vec![
            Box::new(StepNameSegment {
                name: step_name.map(String::from),
            }),
            Box::new(StepTypeSegment { event_type: None }),
            Box::new(IdentifierSegment::new(
                "booking",
                order_id.map(String::from),
            )),
            Box::new(IdentifierSegment::new("email", email.map(String::from))),
        ];
        Self::create_topic(&segments)
    }

    /// Parses a topic string into a map of segment keys and values
    fn parse_topic(topic: &str) -> HashMap<String, String> {
        let mut segments = HashMap::new();
        let mut iter = topic.split(':').peekable();

        while let Some(key) = iter.next() {
            if let Some(value) = iter.next() {
                segments.insert(key.to_string(), value.to_string());
            }
        }
        segments
    }

    /// Checks if a topic matches a pattern with wildcards
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

        let pattern_segments = Self::parse_topic(pattern);
        let topic_segments = Self::parse_topic(topic);

        // Check if all pattern segments match corresponding topic segments
        pattern_segments.iter().all(|(key, pattern_value)| {
            if let Some(topic_value) = topic_segments.get(key) {
                pattern_value == "*" || pattern_value == topic_value
            } else {
                false
            }
        })
    }

    // Convenience methods for creating topic patterns
    pub fn subscribe_all_steps_pattern() -> String {
        Self::make_topic_pattern(None, None, None)
    }

    pub fn subscribe_by_email_pattern(email: &str) -> String {
        Self::make_topic_pattern(Some(email), None, None)
    }

    pub fn subscribe_by_order_id_pattern(order_id: &str) -> String {
        Self::make_topic_pattern(None, Some(order_id), None)
    }

    pub fn subscribe_by_email_and_order_id_pattern(email: &str, order_id: &str) -> String {
        Self::make_topic_pattern(Some(email), Some(order_id), None)
    }

    pub fn subscribe_by_step_name_pattern(step_name: &str) -> String {
        Self::make_topic_pattern(None, None, Some(step_name))
    }

    // /// Checks if a topic matches a pattern with wildcards.
    // ///
    // /// Pattern can contain '*' as wildcards for any segment.
    // /// For example:
    // /// - "step:*:booking:ABC123:email:*" matches "step:payment:booking:ABC123:email:user@example.com"
    // /// - "step:payment:*" matches "step:payment:anything"
    // pub fn matches_pattern(pattern: &str, topic: &str) -> bool {
    //     use tracing::{info, span, Level};
    //     let span = span!(
    //         Level::INFO,
    //         "event_bus_match",
    //         pattern = pattern,
    //         topic = topic
    //     );
    //     let _enter = span.enter();
    //     info!(
    //         "[EVENT_BUS MATCH] pattern: '{}', topic: '{}'",
    //         pattern, topic
    //     );

    //     // Helper to extract logical segments from a topic string
    //     fn extract_segments(s: &str) -> Option<(String, String, String)> {
    //         // Expect format: step:<step_name>:<event_type>:booking:<booking_id>:email:<email>
    //         let mut step_name = None;
    //         let mut event_type = None;
    //         let mut booking_id = None;
    //         let mut email = None;
    //         let mut iter = s.split(':').peekable();
    //         while let Some(seg) = iter.next() {
    //             match seg {
    //                 "step" => {
    //                     step_name = iter.next().map(|s| s.to_string());
    //                     event_type = iter.next().map(|s| s.to_string());
    //                 }
    //                 "booking" => {
    //                     booking_id = iter.next().map(|s| s.to_string());
    //                 }
    //                 "email" => {
    //                     email = iter.next().map(|s| s.to_string());
    //                 }
    //                 _ => {}
    //             }
    //         }
    //         match (step_name, event_type, booking_id, email) {
    //             (Some(step_name), Some(event_type), Some(booking_id), Some(email)) => {
    //                 Some((format!("{}:{}", step_name, event_type), booking_id, email))
    //             }
    //             _ => None,
    //         }
    //     }

    //     let pattern_segs = extract_segments(pattern);
    //     let topic_segs = extract_segments(topic);

    //     if pattern_segs.is_none() || topic_segs.is_none() {
    //         info!("[EVENT_BUS MATCH] Could not extract logical segments for matching");
    //         return false;
    //     }

    //     let (pattern_step, pattern_booking, pattern_email) = pattern_segs.unwrap();
    //     let (topic_step, topic_booking, topic_email) = topic_segs.unwrap();

    //     // Match step (with wildcard support)
    //     let step_match = pattern_step == "*:*"
    //         || pattern_step
    //             .split(':')
    //             .zip(topic_step.split(':'))
    //             .all(|(p, t)| p == "*" || p == t);
    //     let booking_match = pattern_booking == "*" || pattern_booking == topic_booking;
    //     let email_match = pattern_email == "*" || pattern_email == topic_email;

    //     info!(
    //         "[EVENT_BUS MATCH] step_match: {}, booking_match: {}, email_match: {}",
    //         step_match, booking_match, email_match
    //     );
    //     let result = step_match && booking_match && email_match;
    //     if result {
    //         info!(
    //             "[EVENT_BUS MATCH] pattern '{}' matches topic '{}'",
    //             pattern, topic
    //         );
    //     } else {
    //         info!(
    //             "[EVENT_BUS MATCH] pattern '{}' does NOT match topic '{}'",
    //             pattern, topic
    //         );
    //     }
    //     result
    // }
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
        assert_eq!(pattern, "step:*:step_type:*:booking:*:email:*");
    }

    #[test]
    fn test_subscribe_by_email_pattern() {
        let pattern = NotifierEvent::subscribe_by_email_pattern("user@example.com");
        assert_eq!(
            pattern,
            "step:*:step_type:*:booking:*:email:user@example.com"
        );
    }

    #[test]
    fn test_subscribe_by_order_id_pattern() {
        let pattern = NotifierEvent::subscribe_by_order_id_pattern("ORDER123");
        assert_eq!(pattern, "step:*:step_type:*:booking:ORDER123:email:*");
    }

    #[test]
    fn test_subscribe_by_email_and_order_id_pattern() {
        let pattern =
            NotifierEvent::subscribe_by_email_and_order_id_pattern("user@example.com", "ORDER123");
        assert_eq!(
            pattern,
            "step:*:step_type:*:booking:ORDER123:email:user@example.com"
        );
    }

    #[test]
    fn test_subscribe_by_step_name_pattern() {
        let pattern = NotifierEvent::subscribe_by_step_name_pattern("payment");
        assert_eq!(pattern, "step:payment:step_type:*:booking:*:email:*");
    }

    #[test]
    fn test_topic_format() {
        let event = NotifierEvent::new_step_start(
            "ORDER123".to_string(),
            "user@example.com".to_string(),
            "payment".to_string(),
            "corr123".to_string(),
        );
        assert_eq!(
            event.topic(),
            "step:payment:step_type:on_step_start:booking:ORDER123:email:user@example.com"
        );
    }

    #[test]
    fn test_topic_pattern() {
        let pattern = NotifierEvent::make_topic_pattern(
            Some("user@example.com"),
            Some("ORDER123"),
            Some("payment"),
        );
        assert_eq!(
            pattern,
            "step:payment:step_type:*:booking:ORDER123:email:user@example.com"
        );
    }

    #[test]
    fn test_matches_pattern_with_new_segment() {
        // Test adding a new payment_id segment
        let topic = "step:payment:step_type:on_step_start:booking:ORDER123:email:user@example.com:payment_id:PAY123";
        let pattern = "step:payment:step_type:*:booking:*:email:*:payment_id:PAY123";
        assert!(NotifierEvent::matches_pattern(pattern, topic));
    }
}
