use crate::utils::notifier_event::NotifierEvent;
use crate::utils::tokio_event_bus::{self, EventBus};
use std::sync::Arc;

/// Notifier publishes events to an EventBus.
/// We use the concrete type: EventBus<NotifierEvent>
pub struct Notifier {
    pub bus: Option<Arc<crate::utils::tokio_event_bus::EventBus<NotifierEvent>>>,
}

impl Notifier {
    /// Create a new notifier with an optional event bus.
    pub fn new(bus: Option<Arc<crate::utils::tokio_event_bus::EventBus<NotifierEvent>>>) -> Self {
        Notifier { bus }
    }

    /// Create a new notifier with a new event bus.
    pub fn with_bus() -> Self {
        let bus = Arc::new(crate::utils::tokio_event_bus::EventBus::<NotifierEvent>::new());
        Notifier { bus: Some(bus) }
    }

    /// Publish an event to the event bus.
    /// If the notifier is uninitialized, this is a no-op.
    pub async fn notify(&self, event: NotifierEvent) {
        if let Some(bus) = &self.bus {
            let topic = event.topic();

            let bus_event = crate::utils::tokio_event_bus::Event {
                topic,
                payload: event,
            };

            bus.publish(bus_event).await;
        }
    }
}

#[cfg(test)]
mod notifier_tests {
    use super::*;
    use tokio::test;
    use tokio_stream::wrappers::ReceiverStream;
    use tokio_stream::StreamExt;

    #[test]
    async fn test_notify_with_bus() {
        let notifier = Notifier::with_bus();

        // Get the bus and create a subscription
        let bus = notifier.bus.as_ref().unwrap().clone();
        let (_, subscription_receiver) = bus.subscribe("booking:*".to_string()).await;
        let mut subscription_stream = ReceiverStream::new(subscription_receiver);

        // Send an event
        let event = NotifierEvent::new_step_start(
            "booking123".to_string(),
            "step1".to_string(),
            "corr123".to_string(),
        );
        notifier.notify(event.clone()).await;

        // Verify the event was received
        let received_event = subscription_stream
            .next()
            .await
            .expect("Should receive an event");
        let received = received_event.payload;

        // Compare individual fields
        assert_eq!(received.order_id, event.order_id);
        assert_eq!(received.step_name, event.step_name);
        assert_eq!(received.correlation_id, event.correlation_id);
        assert_eq!(received.event_type, event.event_type);
    }

    #[test]
    async fn test_notify_without_bus() {
        let notifier = Notifier::new(None);
        let event = NotifierEvent::new_step_start(
            "booking123".to_string(),
            "step1".to_string(),
            "corr123".to_string(),
        );

        // This should complete successfully without error, even though there's no bus
        notifier.notify(event).await;
        // No assertion needed - if we got here without panic, the test passed
    }
}
