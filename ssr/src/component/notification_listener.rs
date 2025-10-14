use crate::{canister::backend, error, log, view_state_layer::GlobalStateForLeptos, warn};

use leptos::*;
use serde::Deserialize;
use web_sys::EventSource;

/// Notification data received from the server
#[derive(Debug, Deserialize, Clone)]
pub struct NotificationData {
    pub correlation_id: String,
    pub order_id: String,
    pub step: Option<String>,
    #[serde(rename = "type")]
    pub event_type: String,
    pub backend_booking_details: Option<backend::Booking>,
}

/// Component that listens to server-sent events and handles notifications
#[component]
pub fn NotificationListener(
    /// Order ID to filter notifications (optional)
    #[prop(optional)]
    order_id: Option<String>,
    /// Email to filter notifications (optional)
    #[prop(optional)]
    email: Option<String>,
    /// Type of events to subscribe to (optional)
    #[prop(optional)]
    event_type: Option<String>,
    // Callback when a notification is received
    on_notification: Box<dyn Fn(NotificationData)>,
) -> impl IntoView {
    #[cfg(not(feature = "ssr"))]
    {
        use futures::StreamExt;
        use gloo_net::eventsource::futures::EventSource as GlooEventSource;
        use std::time::Duration;

        // Build the URL with query parameters
        let mut url = "/stream/events".to_string();
        let mut params = Vec::new();

        if let Some(id) = &order_id {
            params.push(format!("order_id={}", id));
        }
        if let Some(email) = &email {
            params.push(format!("email={}", email));
        }
        if let Some(event_type) = &event_type {
            params.push(format!("event_type={}", event_type));
        }

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        log!("Setting up SSE connection to: {}", url);

        // Create signals for connection state monitoring
        let (connection_status, set_connection_status) = create_signal("connecting".to_string());

        // Create event source and subscribe to messages
        let mut source = GlooEventSource::new(&url).expect("couldn't connect to SSE stream");

        // Monitor connection state
        let connection_state = source.ready_state();
        let initial_state = match connection_state {
            0 => "connecting",
            1 => "open",
            2 => "closed",
            _ => "unknown",
        };
        set_connection_status(initial_state.to_string());
        log!("Initial SSE connection state: {}", initial_state);

        let stream = source
            .subscribe("message")
            .expect("couldn't subscribe to messages");

        // Create a signal from the stream with enhanced error handling
        let s = create_signal_from_stream(stream.map(move |value| match value {
            Ok(event) => {
                let data = event.1.data().as_string().expect("expected string value");
                log!("SSE notification received: {}", data);

                // Update connection status to indicate active communication
                set_connection_status("active".to_string());

                match serde_json::from_str::<NotificationData>(&data) {
                    Ok(notification) => {
                        // Store notification in global state
                        log!("SSE notification parsed successfully: {:#?}", notification);
                        NotificationState::add_notification(notification.clone());
                        // invoke the handler
                        (on_notification)(notification.clone());
                        Some(notification)
                    }
                    Err(e) => {
                        error!(
                            "Failed to parse SSE notification: {} - Raw data: {}",
                            e, data
                        );
                        None
                    }
                }
            }
            Err(e) => {
                error!("Error in SSE stream: {}", e);
                set_connection_status("error".to_string());

                // Log connection state for debugging
                let state = source.ready_state();
                let state_str = match state {
                    0 => "connecting",
                    1 => "open",
                    2 => "closed",
                    _ => "unknown",
                };
                error!(
                    "SSE connection state after error: {} ({})",
                    state_str, state
                );
                None
            }
        }));

        // Periodic connection health check
        let url_for_health_check = url.clone();
        spawn_local(async move {
            let mut interval = gloo_timers::future::IntervalStream::new(5_000); // Check every 5 seconds

            while let Some(_) = interval.next().await {
                let current_state = source.ready_state();
                let state_str = match current_state {
                    0 => "connecting",
                    1 => "open",
                    2 => "closed",
                    _ => "unknown",
                };

                if current_state == 2 {
                    // Closed
                    warn!(
                        "SSE connection closed unexpectedly. State: {} ({})",
                        state_str, current_state
                    );
                    set_connection_status("closed".to_string());
                    break;
                } else if current_state == 1 {
                    // Connection is open and healthy
                    if connection_status.get_untracked() != "active" {
                        set_connection_status("open".to_string());
                    }
                }
            }
        });

        // Cleanup when component is destroyed
        on_cleanup(move || {
            log!("Cleaning up SSE connection");
            source.close();
        });

        // Return connection status indicator for debugging
        view! {
            <div class="hidden" data-sse-status=move || connection_status.get()>
                "SSE: " {move || connection_status.get()}
            </div>
        }
    }

    #[cfg(feature = "ssr")]
    {
        // Server-side rendering - return empty view
        view! {}
    }
}

// /// Example usage component that demonstrates how to use NotificationListener
// #[component]
// pub fn NotificationExample() -> impl IntoView {
//     // let handle_notification = Box::new(move |notification: NotificationData| {
//     //     set_notifications.update(|list| {
//     //         let mut new_list = list.clone();
//     //         new_list.push(notification);
//     //         // Keep only last 5 notifications
//     //         if new_list.len() > 5 {
//     //             new_list.remove(0);
//     //         }
//     //         new_list
//     //     });
//     // });

//     view! {
//         <div>
//             <h2>"Notifications"</h2>
//             <NotificationListener
//                 order_id="NP$6:ABC123$16:user@example.com".to_string()
//                 email="user@example.com".to_string()
//                 event_type="nowpayments".to_string()
//                 on_notification={Box::new(move |notification: NotificationData| {
//                     // SSEBookingStatusUpdates::update_from_notification(&notification);
//                     log!("invoking_on_notification: {:#?}", notification);
//                 })}
//             />
//             <ul>
//                 {move || NotificationState::get().notifications.get().into_iter().map(|n: NotificationData| {
//                     view! {
//                         <li>
//                             {format!("{}: {} - {}", n.event_type, n.step.unwrap_or_default(), n.order_id)}
//                         </li>
//                     }
//                 }).collect::<Vec<_>>()}
//             </ul>
//         </div>
//     }
// }

// state management for notifications

#[derive(Clone, Default)]
pub struct NotificationState {
    pub notifications: RwSignal<Vec<NotificationData>>,
}

impl GlobalStateForLeptos for NotificationState {}

impl NotificationState {
    pub fn add_notification(notification: NotificationData) {
        let mut current_notifications = Self::get_notifications().get_untracked();
        current_notifications.push(notification.clone());
        // Keep only last 20 notifications
        if current_notifications.len() > 20 {
            current_notifications.remove(0);
        }

        Self::get().notifications.set(current_notifications);
    }

    pub fn get_notifications() -> RwSignal<Vec<NotificationData>> {
        Self::get().notifications
    }
}
