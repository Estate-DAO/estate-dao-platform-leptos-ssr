use crate::{canister::backend, error, log, view_state_layer::GlobalStateForLeptos, warn};

use leptos::prelude::*;
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

        // TODO: Reimplement SSE for Leptos 0.8
        // create_signal_from_stream doesn't exist in 0.8
        // Need to use spawn_local with a manual signal update loop

        warn!(
            "NotificationListener: SSE functionality temporarily disabled for Leptos 0.8 migration"
        );
        warn!("TODO: Reimplement using spawn_local + manual stream handling");

        // Temporary stub implementation
        // let mut source = GlooEventSource::new(&url).expect("couldn't connect to SSE stream");
        // let stream = source.subscribe("message").expect("couldn't subscribe to messages");

        // TODO: Implement with:
        // spawn_local(async move {
        //     let mut stream = stream;
        //     while let Some(result) = stream.next().await {
        //         // Handle stream events and update signals manually
        //     }
        // });

        view! {}
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
