use crate::{error, log, state::GlobalStateForLeptos, warn};

use leptos::*;
use serde::Deserialize;
use web_sys::EventSource;

/// Notification data received from the server
#[derive(Debug, Deserialize, Clone)]
pub struct NotificationData {
    pub order_id: String,
    pub step: String,
    // pub correlation_id: String,
    #[serde(rename = "type")]
    pub event_type: String,
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
    // on_notification: Box<dyn Fn(NotificationData)>,
) -> impl IntoView {
    #[cfg(not(feature = "ssr"))]
    {
        use futures::StreamExt;
        use gloo_net::eventsource::futures::EventSource as GlooEventSource;

        // Build the URL with query parameters
        let mut url = "/api/events".to_string();
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

        // Create event source and subscribe to messages
        let mut source = GlooEventSource::new(&url).expect("couldn't connect to SSE stream");

        let stream = source
            .subscribe("message")
            .expect("couldn't subscribe to messages");

        // Create a signal from the stream
        let s = create_signal_from_stream(stream.map(move |value| match value {
            Ok(event) => {
                let data = event.1.data().as_string().expect("expected string value");

                match serde_json::from_str::<NotificationData>(&data) {
                    Ok(notification) => {
                        // Store notification in global state
                        NotificationState::add_notification(notification.clone());
                        Some(notification)
                    }
                    Err(e) => {
                        error!("Failed to parse notification: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Error in event stream: {}", e);
                None
            }
        }));

        // Cleanup when component is destroyed
        on_cleanup(move || source.close());

        // Return an empty view - this component only handles events
        view! {}
    }

    #[cfg(feature = "ssr")]
    {
        // Server-side rendering - return empty view
        view! {}
    }
}

/// Example usage component that demonstrates how to use NotificationListener
#[component]
pub fn NotificationExample() -> impl IntoView {
    // let (notifications, set_notifications) = create_signal(Vec::new());
    let notifications = NotificationState::get().notifications;

    // let handle_notification = Box::new(move |notification: NotificationData| {
    //     set_notifications.update(|list| {
    //         let mut new_list = list.clone();
    //         new_list.push(notification);
    //         // Keep only last 5 notifications
    //         if new_list.len() > 5 {
    //             new_list.remove(0);
    //         }
    //         new_list
    //     });
    // });

    view! {
        <div>
            <h2>"Notifications"</h2>
            <NotificationListener
                order_id="NP$6:ABC123$16:user@example.com".to_string()
                email="user@example.com".to_string()
                event_type="nowpayments".to_string()
                // on_notification=handle_notification
            />
            <ul>
                {move || notifications.get().into_iter().map(|n: NotificationData| {
                    view! {
                        <li>
                            {format!("{}: {} - {}", n.event_type, n.step, n.order_id)}
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ul>
        </div>
    }
}

// state management for notifications

#[derive(Clone, Default)]
pub struct NotificationState {
    pub notifications: RwSignal<Vec<NotificationData>>,
}

impl GlobalStateForLeptos for NotificationState {}

impl NotificationState {
    pub fn add_notification(notification: NotificationData) {
        Self::get().notifications.update(|list| {
            let mut new_list = list.clone();
            new_list.push(notification);
            // Keep only last 20 notifications
            if new_list.len() > 20 {
                new_list.remove(0);
            }
        });
    }
}
