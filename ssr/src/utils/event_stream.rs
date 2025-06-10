use axum::{
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
};
use futures::{Stream, StreamExt};
use serde::Deserialize;
use std::{convert::Infallible, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::instrument;
use tracing::{debug, error, info};

use crate::utils::{
    notifier::Notifier,
    notifier_event::NotifierEvent,
    tokio_event_bus::{Event as BusEvent, EventBus},
};
use crate::view_state_layer::AppState;

/// Query parameters for the event stream endpoint
#[derive(Debug, Deserialize)]
pub struct EventStreamParams {
    /// Order ID for payment notifications
    pub order_id: Option<String>,
    /// Email for booking notifications
    pub email: Option<String>,
    /// Type of events to subscribe to (payment, booking, all)
    pub event_type: Option<String>,
}

/// Handler for the /api/events endpoint
#[instrument(
    skip(state),
    fields(
        order_id = ?params.order_id,
        email = ?params.email,
        event_type = ?params.event_type
    )
)]
pub async fn event_stream_handler(
    State(state): State<AppState>,
    Query(params): Query<EventStreamParams>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Get the event bus from the notifier
    let notifier = state.notifier_for_pipeline;

    // Create a one-time initial event
    let status_message = if notifier.bus.is_none() {
        String::from("{\"status\":\"no_bus_available\"}")
    } else {
        // Determine subscription pattern based on params
        let subscription_pattern =
            determine_subscription_pattern(&params.order_id, &params.email, &params.event_type);

        info!(
            "Creating event stream with pattern: {}",
            subscription_pattern
        );

        format!(
            "{{\"status\":\"connected\",\"pattern\":\"{}\"}}",
            subscription_pattern
        )
    };

    // let event_bus = if let Some(bus) = notifier.bus {
    //     bus
    // } else {
    //     // If no event bus, return an empty stream with initial connection event
    //     let initial_event = Event::default()
    //         .event("connection")
    //         .data("{\"status\":\"no_bus_available\"}");

    //     let stream = futures::stream::once(async { Ok(initial_event) });
    //     return Sse::new(stream).keep_alive(KeepAlive::default());
    // };

    let initial_event = Event::default().event("connection").data(status_message);

    // Create a stream that starts with the initial event
    let initial_stream = futures::stream::once(async { Ok(initial_event) });

    // Subscribe to the event bus
    let subscription_pattern =
        determine_subscription_pattern(&params.order_id, &params.email, &params.event_type);
    let (_, receiver) = notifier
        .bus
        .as_ref()
        .unwrap()
        .subscribe(subscription_pattern)
        .await;

    // Convert NotifierEvent receiver to SSE event stream
    let notification_stream =
        ReceiverStream::new(receiver).map(|event: BusEvent<NotifierEvent>| {
            let data = match event.payload {
                NotifierEvent {
                    order_id,
                    step_name,
                    correlation_id,
                    event_type,
                    ..
                } => {
                    let val = serde_json::json!({
                        "order_id": order_id,
                        "step": step_name,
                        "correlation_id": correlation_id,
                        "type": event_type,
                    })
                    .to_string();

                    info!("serde_json event value : {}", val);
                    val
                }
            };

            Ok(Event::default().data(data))
        });

    // Combine the initial stream with the notification stream
    let combined_stream = initial_stream.chain(notification_stream);

    // Create SSE response with keep-alive
    // Sse::new(combined_stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(1)).text("keep-alive-text"))
    Sse::new(combined_stream).keep_alive(KeepAlive::default())
}

/// Determines the subscription pattern based on the provided parameters
fn determine_subscription_pattern(
    order_id: &Option<String>,
    email: &Option<String>,
    event_type: &Option<String>,
) -> String {
    // match (order_id, email) {
    //     // Uncomment and implement your filtering logic here
    //     // (Some(order_id), _, Some(event_type)) if event_type == "payment" => format!("payment:{}:*", order_id),
    //     // (_, Some(email), Some(event_type)) if event_type == "booking" => format!("booking:{}:*", email),
    //     // (Some(order_id), _, _) => format!("*:{}:*", order_id),
    //     // (_, Some(email), _) => format!("*:{}:*", email),
    //     _ => "*".to_string(), // Subscribe to all events if no specific identifier
    // }

    // todo (event_bus): for now, we are only subscribing to event based on user_email
    NotifierEvent::make_topic_pattern(email.as_deref(), None, None)
}
