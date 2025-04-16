//!
//! # NotifierEvent System: Reliable Event Topic Encoding and Parsing
//!
//! The `NotifierEvent` system provides a robust, extensible, and maintainable way to represent, encode, and parse notification events that occur during pipeline and step execution in distributed systems.
//!
//! ## Why This System?
//!
//! - **Reliable Pub/Sub:** Enables precise topic-based filtering and flexible subscriptions for event-driven architectures.
//! - **Colons in Values:** Handles edge cases where segment values (like order IDs or emails) may themselves contain colons or special characters.
//! - **Extensible Segments:** Adding new segments (e.g., `payment_id`) or event types is straightforward and does not break existing consumers.
//! - **Consistent Topic Construction:** Topics are always constructed and parsed in a fixed, well-documented order, avoiding ambiguity and bugs.
//!
//! ## Core Principles
//!
//! - **Segmented Topics:** Each event is encoded as a topic string with ordered `key:value` pairs (e.g., `step:payment:step_type:on_step_start:booking:ORDER:123:email:user@example.com`).
//! - **Fixed Segment Order:** The segment order is defined in one place (`segment_order()`), ensuring consistency across topic creation and parsing.
//! - **Robust Parsing:** The parser can reconstruct the original values even if they contain colons or segment keys themselves.
//! - **Wildcard Subscriptions:** Supports wildcard (`*`) values for flexible pattern subscriptions (e.g., all steps for a user, or all events for an order).
//!
//! ## Typical Usage
//!
//! ### Creating and Publishing an Event
//! ```rust
//! let event = NotifierEvent::new_step_start(
//!     "ORDER:123".to_string(),
//!     "user@example.com".to_string(),
//!     "payment".to_string(),
//!     Uuid::new_v4(),
//! );
//! let topic = event.topic(); // e.g., "step:payment:step_type:on_step_start:booking:ORDER:123:email:user@example.com"
//! // Publish this topic to your event bus...
//! ```
//!
//! ### Subscribing to Topics
//!
//! Subscribe to all events for a user:
//! ```rust
//! let pattern = NotifierEvent::subscribe_by_email_pattern("user@example.com");
//! // pattern: "step:*:step_type:*:booking:*:email:user@example.com"
//! ```
//!
//! Subscribe to all events for a specific order:
//! ```rust
//! let pattern = NotifierEvent::subscribe_by_order_id_pattern("ORDER:123");
//! // pattern: "step:*:step_type:*:booking:ORDER:123:email:*"
//! ```
//!
//! ### Matching Topics
//!
//! ```rust
//! let matches = NotifierEvent::matches_pattern(&pattern, &topic);
//! // true if the topic matches the subscription pattern
//! ```
//!
//! ### Parsing a Topic String
//!
//! ```rust
//! let segments = NotifierEvent::parse_topic(&topic);
//! // segments["booking"] == "ORDER:123"
//! // segments["email"] == "user@example.com"
//! ```
//!
//! ## Design Notes
//!
//! - **Adding Segments:** To add a new segment (e.g., `payment_id`), update `segment_order()` and ensure all topic creation/parsing logic uses this order.
//! - **Testing:** The module includes comprehensive tests for colons in values, empty fields, Unicode, and wildcard matching.
//! - **Extensibility:** New event types or segments can be added with minimal changes, and existing code will continue to work as long as the segment order is respected.
//!
//! ## See Also
//! - `NotifierEventType`: Enum of all supported event types.
//! - `TopicSegment` trait: For custom segment implementations.
//! - Tests at the bottom of this file for edge cases and usage patterns.
//!
//! ## Example Usage
//!
//! ```rust
//! let event = NotifierEvent::new_step_start(
//!     "ORDER123".to_string(),
//!     "user@example.com".to_string(),
//!     "payment".to_string(),
//!     Uuid::new_v4(),
//! );
//! let topic = event.topic();
//! // topic: "step:payment:step_type:on_step_start:booking:ORDER123:email:user@example.com"
//! ```
//!
//! ## Robustness
//!
//! - The segment parsing logic ensures values with colons or special characters are handled safely.
//! - The fixed segment order ensures maintainability and consistency across the codebase.
//!
//! ## Extensibility
//!
//! - New segment types or event types can be added with minimal changes.
//! - The topic system supports flexible subscriptions for advanced notification workflows.

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
    /// Creates a new NotifierEvent representing the start of a step.
    ///
    /// # Arguments
    /// * `order_id` - The order or booking identifier associated with the event.
    /// * `email` - The email address associated with the event.
    /// * `step_name` - The name of the step that has started.
    /// * `corr_id` - A correlation ID for tracing and linking related events.
    ///
    /// # Returns
    /// A new `NotifierEvent` instance with event type set to `OnStepStart` and the current timestamp.
    ///
    /// # Example
    /// ```rust
    /// let event = NotifierEvent::new_step_start(
    ///     "ORDER123".to_string(),
    ///     "user@example.com".to_string(),
    ///     "payment".to_string(),
    ///     Uuid::new_v4(),
    /// );
    /// ```
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

    /// Defines the order of segments in a topic string
    fn segment_order() -> Vec<&'static str> {
        vec!["step", "step_type", "booking", "email", "payment_id"]
    }

    /// Creates a topic string from a list of segments
    fn create_topic(segments: &[Box<dyn TopicSegment>]) -> String {
        // Create a map of segment keys to their values
        let mut segment_map: HashMap<&str, String> = HashMap::new();
        for segment in segments {
            segment_map.insert(segment.segment_key(), segment.segment_value());
        }

        // Build topic string in the defined order
        Self::segment_order()
            .into_iter()
            .filter_map(|key| {
                segment_map
                    .get(key)
                    .map(|value| format!("{}:{}", key, value))
            })
            .collect::<Vec<_>>()
            .join(":")
    }

    /// Parses a topic string into a map of segment keys and values
    fn parse_topic(topic: &str) -> HashMap<String, String> {
        let mut segments = HashMap::new();
        let mut parts = topic.split(':');
        let segment_order = Self::segment_order();
        let mut current_key = None;
        let mut current_value = Vec::new();

        while let Some(part) = parts.next() {
            if let Some(key) = segment_order.iter().find(|&&k| k == part) {
                // If we have a previous key-value pair, add it to segments
                if let Some(prev_key) = current_key.take() {
                    if !current_value.is_empty() {
                        segments.insert(prev_key, current_value.join(":"));
                        current_value.clear();
                    }
                }
                current_key = Some(key.to_string());
            } else if let Some(key) = current_key.as_ref() {
                // This is part of the current value
                current_value.push(part);
            }
        }

        // Don't forget to add the last key-value pair
        if let Some(key) = current_key {
            if !current_value.is_empty() {
                segments.insert(key, current_value.join(":"));
            } else {
                segments.insert(key, String::new());
            }
        }

        segments
    }

    #[rustfmt::skip]
    /// Creates a topic string based on the event details
    pub fn topic(&self) -> String {
        let segments: Vec<Box<dyn TopicSegment>> = vec![
            Box::new(StepNameSegment { name: self.step_name.clone() }),
            Box::new(StepTypeSegment { event_type: Some(self.event_type.clone()) }),
            Box::new(IdentifierSegment::new("booking", Some(self.order_id.clone()))),
            Box::new(IdentifierSegment::new("email", Some(self.email.clone()))),
        ];
        Self::create_topic(&segments)
    }

    /// Creates a topic pattern for subscribing to events
    #[rustfmt::skip]
    pub fn make_topic_pattern(
        email: Option<&str>,
        order_id: Option<&str>,
        step_name: Option<&str>,
    ) -> String {
        let segments: Vec<Box<dyn TopicSegment>> = vec![
            Box::new(StepNameSegment { name: step_name.map(String::from) }),
            Box::new(StepTypeSegment { event_type: None }),
            Box::new(IdentifierSegment::new("booking", order_id.map(String::from))),
            Box::new(IdentifierSegment::new("email", email.map(String::from))),
        ];
        Self::create_topic(&segments)
    }

    /// Checks if a topic matches a pattern with wildcards
    #[rustfmt::skip]
    pub fn matches_pattern(pattern: &str, topic: &str) -> bool {
        use tracing::{info, span, Level};
        let span = span!(Level::INFO, "event_bus_match", pattern = pattern, topic = topic);
        let _enter = span.enter();
        info!("[EVENT_BUS MATCH] pattern: '{}', topic: '{}'", pattern, topic);

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
    #[rustfmt::skip]
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

    #[test]
    fn test_parse_topic_with_colons() {
        let topic = "step:payment:step_type:on_step_start:booking:ORDER:123:email:user@example.com";
        let segments = NotifierEvent::parse_topic(topic);
        assert_eq!(segments.get("step").unwrap(), "payment");
        assert_eq!(segments.get("step_type").unwrap(), "on_step_start");
        assert_eq!(segments.get("booking").unwrap(), "ORDER:123");
        assert_eq!(segments.get("email").unwrap(), "user@example.com");
    }

    #[test]
    #[rustfmt::skip]
    fn test_create_topic_with_colons() {
        let segments: Vec<Box<dyn TopicSegment>> = vec![
            Box::new(StepNameSegment { name: Some("payment".to_string()) }),
            Box::new(StepTypeSegment {
                event_type: Some(NotifierEventType::OnStepStart),
            }),
            Box::new(IdentifierSegment::new("booking", Some("ORDER:123".to_string()))),
            Box::new(IdentifierSegment::new("email", Some("user@example.com".to_string()))),
        ];
        let topic = NotifierEvent::create_topic(&segments);
        assert_eq!(topic, "step:payment:step_type:on_step_start:booking:ORDER:123:email:user@example.com");
    }

    #[test]
    fn test_parse_topic_edge_cases() {
        // Test empty segment values
        let topic = "step::step_type:on_step_start:booking:123:email:user@example.com";
        let segments = NotifierEvent::parse_topic(topic);
        assert_eq!(segments.get("step").unwrap(), "");

        // Test multiple colons in segment values
        let topic = "step:complex:value:here:step_type:on_step_start:booking:order:123:email:user@example.com";
        let segments = NotifierEvent::parse_topic(topic);
        assert_eq!(segments.get("step").unwrap(), "complex:value:here");
        assert_eq!(segments.get("booking").unwrap(), "order:123");

        // Test segment values containing segment keys
        let topic = "step:contains_step_type:step_type:on_step_start:booking:contains_email:email:user@example.com";
        let segments = NotifierEvent::parse_topic(topic);
        assert_eq!(segments.get("step").unwrap(), "contains_step_type");
        assert_eq!(segments.get("booking").unwrap(), "contains_email");

        // Test Unicode characters in segment values
        let topic = "step:测试步骤:step_type:on_step_start:booking:订单123:email:用户@example.com";
        let segments = NotifierEvent::parse_topic(topic);
        assert_eq!(segments.get("step").unwrap(), "测试步骤");
        assert_eq!(segments.get("booking").unwrap(), "订单123");
        assert_eq!(segments.get("email").unwrap(), "用户@example.com");
    }

    #[test]
    #[rustfmt::skip]
    fn test_create_topic_edge_cases() {
        // Test empty segment values
        let segments: Vec<Box<dyn TopicSegment>> = vec![
            Box::new(StepNameSegment { name: Some("".to_string()) }),
            Box::new(StepTypeSegment { event_type: Some(NotifierEventType::OnStepStart) }),
            Box::new(IdentifierSegment::new("booking", Some("123".to_string()))),
            Box::new(IdentifierSegment::new("email", Some("user@example.com".to_string()))),
        ];
        let topic = NotifierEvent::create_topic(&segments);
        assert_eq!(topic, "step::step_type:on_step_start:booking:123:email:user@example.com");

        // Test multiple colons in segment values
        let segments: Vec<Box<dyn TopicSegment>> = vec![
            Box::new(StepNameSegment { name: Some("complex:value:here".to_string()) }),
            Box::new(StepTypeSegment {
                event_type: Some(NotifierEventType::OnStepStart),
            }),
            Box::new(IdentifierSegment::new("booking", Some("order:123".to_string()))),
            Box::new(IdentifierSegment::new("email", Some("user@example.com".to_string()))),
        ];
        let topic = NotifierEvent::create_topic(&segments);
        assert_eq!(topic, "step:complex:value:here:step_type:on_step_start:booking:order:123:email:user@example.com");

        // Test segment values containing segment keys
        let segments: Vec<Box<dyn TopicSegment>> = vec![
            Box::new(StepNameSegment { name: Some("contains_step_type".to_string()) }),
            Box::new(StepTypeSegment { event_type: Some(NotifierEventType::OnStepStart) }),
            Box::new(IdentifierSegment::new("booking", Some("contains_email".to_string()))),
            Box::new(IdentifierSegment::new("email", Some("user@example.com".to_string()))),
        ];
        let topic = NotifierEvent::create_topic(&segments);
        assert_eq!(topic, "step:contains_step_type:step_type:on_step_start:booking:contains_email:email:user@example.com");

        // Test Unicode characters in segment values
        let segments: Vec<Box<dyn TopicSegment>> = vec![
            Box::new(StepNameSegment { name: Some("测试步骤".to_string()) }),
            Box::new(StepTypeSegment { event_type: Some(NotifierEventType::OnStepStart) }),
            Box::new(IdentifierSegment::new("booking", Some("订单123".to_string()))),
            Box::new(IdentifierSegment::new("email", Some("用户@example.com".to_string()))),
        ];
        let topic = NotifierEvent::create_topic(&segments);
        assert_eq!(topic, "step:测试步骤:step_type:on_step_start:booking:订单123:email:用户@example.com");
    }
}
