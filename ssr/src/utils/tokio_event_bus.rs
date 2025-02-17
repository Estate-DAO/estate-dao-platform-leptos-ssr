/// A lightweight, asynchronous event bus implementation using Tokio.
///
/// The `EventBus` allows publishers to send events to subscribers based on topic patterns.
/// Subscribers can register for events matching a specific pattern (e.g., "booking:*").
/// Events are delivered asynchronously using bounded channels.
///
/// # Examples
///
/// ```rust
/// use tokio_event_bus::{Event, EventBus};
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() {
///     let event_bus = Arc::new(EventBus::<String>::new());
///
///     // Subscribe to all booking events
///     let (subscriber_id, mut receiver) = event_bus.subscribe("booking:*".to_string()).await;
///
///     // Publish an event
///     let event = Event {
///         topic: "booking:123".to_string(),
///         payload: "Booking confirmed".to_string(),
///     };
///     event_bus.publish(event).await;
///
///     // Receive the event
///     let received = receiver.recv().await.unwrap();
///     assert_eq!(received.topic, "booking:123");
/// }
/// ```
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{mpsc, RwLock};
use tracing::warn;

/// An event with a topic and a generic payload.
#[derive(Debug, Clone)]
pub struct Event<T: Clone + std::fmt::Debug + Send + 'static> {
    /// The event's topic (e.g., "booking:123").
    pub topic: String,
    /// The event's payload, which can be any type implementing `Clone`, `Debug`, and `Send`.
    pub payload: T,
}

/// A subscription holding a topic pattern and a bounded sender channel for events.
pub struct Subscription<T: Clone + std::fmt::Debug + Send + 'static> {
    /// The topic pattern this subscription listens to (e.g., "booking:*").
    pub pattern: String,
    /// The bounded channel sender for delivering events to the subscriber.
    pub sender: mpsc::Sender<Event<T>>,
}

/// The EventBus holds a registry of subscriptions and auto-generates subscriber IDs.
pub struct EventBus<T: Clone + std::fmt::Debug + Send + 'static> {
    /// A thread-safe map of subscriber IDs to their subscriptions.
    subscriptions: RwLock<HashMap<usize, Subscription<T>>>,
    /// An atomic counter for generating unique subscriber IDs.
    next_id: AtomicUsize,
}

impl<T: Clone + std::fmt::Debug + Send + 'static> EventBus<T> {
    /// Creates a new `EventBus`.
    ///
    /// # Returns
    /// A new instance of `EventBus`.
    pub fn new() -> Self {
        Self {
            subscriptions: RwLock::new(HashMap::new()),
            next_id: AtomicUsize::new(1),
        }
    }

    /// Subscribe to events matching the given topic pattern.
    ///
    /// # Arguments
    /// - `pattern`: The topic pattern to subscribe to (e.g., "booking:*").
    ///
    /// # Returns
    /// A tuple containing the subscriber ID and a receiver for events.
    pub async fn subscribe(&self, pattern: String) -> (usize, mpsc::Receiver<Event<T>>) {
        // Create a bounded channel with capacity 1000.
        let (sender, receiver) = mpsc::channel(1000);
        let subscription = Subscription { pattern, sender };

        // Generate a unique subscriber id.
        let subscriber_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut subs = self.subscriptions.write().await;
        subs.insert(subscriber_id, subscription);

        (subscriber_id, receiver)
    }

    /// Unsubscribe using the generated subscriber ID.
    ///
    /// # Arguments
    /// - `subscriber_id`: The ID of the subscriber to remove.
    pub async fn unsubscribe(&self, subscriber_id: usize) {
        let mut subs = self.subscriptions.write().await;
        subs.remove(&subscriber_id);
    }

    /// Publish an event to all matching subscriptions.
    ///
    /// # Arguments
    /// - `event`: The event to publish.
    ///
    /// # Notes
    /// If a subscriber's channel is full, the event will be dropped, and a warning will be logged.
    pub async fn publish(&self, event: Event<T>) {
        let subs = self.subscriptions.read().await;
        for subscription in subs.values() {
            let pattern = &subscription.pattern;
            // Simple wildcard matching: if pattern ends with '*', do a prefix match.
            let is_match = if pattern.ends_with('*') {
                let prefix = &pattern[..pattern.len() - 1];
                event.topic.starts_with(prefix)
            } else {
                event.topic == *pattern
            };

            if is_match {
                // Try to send the event; log a warning if the channel is full.
                if let Err(err) = subscription.sender.try_send(event.clone()) {
                    warn!(
                        "Dropping event on topic '{}' for pattern '{}'. Error: {:?}",
                        event.topic, pattern, err
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time;

    #[tokio::test]
    async fn test_subscribe_id_increment() {
        let bus = EventBus::<String>::new();
        let (id1, _) = bus.subscribe("test:*".to_string()).await;
        let (id2, _) = bus.subscribe("test:*".to_string()).await;
        assert_eq!(id1 + 1, id2);
    }

    #[tokio::test]
    async fn test_publish_to_matching_subscribers() {
        let bus = EventBus::<String>::new();
        let (_, mut receiver) = bus.subscribe("booking:*".to_string()).await;

        let event = Event {
            topic: "booking:123".to_string(),
            payload: "test".to_string(),
        };
        bus.publish(event).await;

        let received = receiver.recv().await.unwrap();
        assert_eq!(received.topic, "booking:123");
        assert_eq!(received.payload, "test");
    }

    #[tokio::test]
    async fn test_no_delivery_to_non_matching_topic() {
        let bus = EventBus::<String>::new();
        let (_, mut receiver) = bus.subscribe("booking:*".to_string()).await;

        let event = Event {
            topic: "payment:456".to_string(),
            payload: "test".to_string(),
        };
        bus.publish(event).await;

        time::sleep(time::Duration::from_millis(100)).await;
        assert!(receiver.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let bus = EventBus::<String>::new();
        let (id, mut receiver) = bus.subscribe("test:*".to_string()).await;
        bus.unsubscribe(id).await;

        let event = Event {
            topic: "test:123".to_string(),
            payload: "test".to_string(),
        };
        bus.publish(event).await;

        time::sleep(time::Duration::from_millis(100)).await;
        assert!(receiver.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_channel_full_drops_events() {
        let bus = EventBus::<String>::new();
        let (_, mut receiver) = bus.subscribe("test:*".to_string()).await;

        // Fill channel capacity
        for i in 0..1000 {
            bus.publish(Event {
                topic: "test:123".to_string(),
                payload: i.to_string(),
            })
            .await;
        }

        // This event should be dropped
        bus.publish(Event {
            topic: "test:123".to_string(),
            payload: "overflow".to_string(),
        })
        .await;

        // Verify first 1000 received
        for i in 0..1000 {
            let received = receiver.recv().await.unwrap();
            assert_eq!(received.payload, i.to_string());
        }

        time::sleep(time::Duration::from_millis(100)).await;
        assert!(receiver.try_recv().is_err());
    }
}
