use leptos::*;
use leptos_router::use_navigate;

use crate::{
    api::client_side_api::ClientSideApiClient,
    app::AppRoutes,
    log,
    page::{HotelListParams, PreviousSearchContext},
    utils::route::join_path_and_query_params,
    view_state_layer::{
        input_group_state::{InputGroupState, OpenDialogComponent},
        ui_search_state::{SearchListResults, UISearchCtx},
    },
};

/// Configuration for search action behavior
#[derive(Clone)]
pub struct SearchActionConfig {
    /// Whether to close input group dialogs
    pub close_dialogs: bool,
    /// Optional signal to manage UI disabled state
    pub manage_ui_state: Option<RwSignal<bool>>,
    /// Whether to reset search results before navigation
    pub reset_search_results: bool,
    /// Whether to navigate with query parameters (recommended)
    pub navigate_with_params: bool,
}

impl Default for SearchActionConfig {
    fn default() -> Self {
        Self {
            close_dialogs: true,
            manage_ui_state: None,
            reset_search_results: true,
            navigate_with_params: true,
        }
    }
}

/// Creates a standardized search action that navigates to hotel list with query parameters
/// This ensures consistent behavior across all search components (root.rs, most_popular.rs, etc.)
pub fn create_search_action(config: SearchActionConfig) -> Action<(), ()> {
    let navigate = use_navigate();
    let search_ctx: UISearchCtx = expect_context();
    let previous_search_ctx = expect_context::<PreviousSearchContext>();

    create_action(move |_| {
        log!(
            "Search action triggered with config: close_dialogs={}, navigate_with_params={}",
            config.close_dialogs,
            config.navigate_with_params
        );

        // Check if criteria changed
        let current_place = search_ctx.place.get_untracked();
        let current_date_range = search_ctx.date_range.get_untracked();
        let current_adults = search_ctx.guests.adults.get_untracked();
        let current_rooms = search_ctx.guests.rooms.get_untracked();

        let criteria_changed = current_place != previous_search_ctx.place.get_untracked()
            || Some(current_date_range) != previous_search_ctx.date_range.get_untracked()
            || current_adults != previous_search_ctx.adults.get_untracked()
            || current_rooms != previous_search_ctx.rooms.get_untracked();

        if config.reset_search_results && criteria_changed {
            SearchListResults::reset();
        }

        if config.close_dialogs {
            InputGroupState::toggle_dialog(OpenDialogComponent::None);
        }

        let nav = navigate.clone();
        let search_ctx = search_ctx.clone();
        let config = config.clone();

        if let Some(disabled_signal) = config.manage_ui_state {
            disabled_signal.set(true);
        }

        let api_client = ClientSideApiClient::new();

        async move {
            log!("Search action async execution started");

            let mut selected_place = search_ctx.place.get_untracked();
            if selected_place.is_none() {
                // Fallback: user may type a destination and hit Search without selecting
                // from the dropdown. Resolve the first matching place so search can proceed.
                let typed_destination = search_ctx
                    .destination
                    .get_untracked()
                    .map(|dest| dest.city.trim().to_string())
                    .filter(|city| !city.is_empty());

                if let Some(query) = typed_destination {
                    match api_client.search_places(query.clone()).await {
                        Ok(results) => {
                            if let Some(place) = results.first().cloned() {
                                UISearchCtx::set_place(place.clone());
                                selected_place = Some(place);
                            } else {
                                log!("No place results found for typed destination: {}", query);
                                if let Some(disabled_signal) = config.manage_ui_state {
                                    disabled_signal.set(false);
                                }
                                return;
                            }
                        }
                        Err(e) => {
                            log!("Error resolving typed destination '{}': {:?}", query, e);
                            if let Some(disabled_signal) = config.manage_ui_state {
                                disabled_signal.set(false);
                            }
                            return;
                        }
                    }
                } else {
                    log!("Search blocked: no selected place and no typed destination");
                    if let Some(disabled_signal) = config.manage_ui_state {
                        disabled_signal.set(false);
                    }
                    return;
                }
            }

            let place_id = if let Some(place) = selected_place {
                place.place_id
            } else {
                if let Some(disabled_signal) = config.manage_ui_state {
                    disabled_signal.set(false);
                }
                return;
            };

            // Always fetch fresh details for the selected place ID.
            // Cached place_details can belong to the previous selection because PlaceData
            // doesn't include a place_id, which can cause stale lat/lng.
            let place_details = match api_client.get_place_details_by_id(place_id).await {
                Ok(details) => details,
                Err(e) => {
                    log!("Error fetching place details: {:?}", e);
                    UISearchCtx::set_place_details(None);
                    if let Some(disabled_signal) = config.manage_ui_state {
                        disabled_signal.set(false);
                    }
                    return;
                }
            };

            UISearchCtx::set_place_details(Some(place_details.clone()));

            let hotel_list_url = if config.navigate_with_params {
                // Generate URL with individual query params (recommended approach)
                // This preserves search state in the URL for shareable links and page reloads
                let hotel_list_params = HotelListParams::from_search_context(&search_ctx);

                // Use individual query params instead of base64 state
                match join_path_and_query_params(
                    AppRoutes::HotelList.to_string(),
                    &hotel_list_params.to_query_params(),
                ) {
                    Ok(url) => {
                        log!("Generated hotel list URL with individual params: {}", url);
                        url
                    }
                    Err(e) => {
                        log!(
                            "Error generating URL with params: {}, falling back to simple path",
                            e
                        );
                        AppRoutes::HotelList.to_string().to_string()
                    }
                }
            } else {
                // Simple navigation without params (legacy approach - not recommended)
                log!("Using simple navigation without query params");
                AppRoutes::HotelList.to_string().to_string()
            };

            log!("[hotel_search_resource] hotel_list_url: {}", hotel_list_url);

            // Navigate to hotel list page
            // The hotel list page will handle loading data based on query parameters
            nav(&hotel_list_url, Default::default());
            log!("Navigation triggered to: {}", hotel_list_url);

            if config.close_dialogs {
                InputGroupState::set_show_full_input(false);
            }

            if let Some(disabled_signal) = config.manage_ui_state {
                disabled_signal.set(false);
            }

            log!("Search action async execution completed");
        }
    })
}

/// Convenience function for components that need the default search behavior
/// This is equivalent to create_search_action(SearchActionConfig::default())
pub fn create_default_search_action() -> Action<(), ()> {
    create_search_action(SearchActionConfig::default())
}

/// Convenience function for components that need search action with UI state management
/// Commonly used in root.rs where there's a local disabled state
pub fn create_search_action_with_ui_state(disabled_signal: RwSignal<bool>) -> Action<(), ()> {
    create_search_action(SearchActionConfig {
        manage_ui_state: Some(disabled_signal),
        ..Default::default()
    })
}
