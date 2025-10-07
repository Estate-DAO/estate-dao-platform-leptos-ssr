use leptos::*;
use leptos_router::use_navigate;

use crate::api::auth::auth_state::AuthStateSignal;
use crate::api::client_side_api::ClientSideApiClient;
use crate::app::AppRoutes;
use crate::component::{Footer, Navbar, SkeletonCards};
use crate::domain::DomainHotelDetailsWithoutRates;
use crate::log;
use crate::page::HotelCardTile;

#[derive(Clone, Debug)]
pub struct WishlistHotelDetails {
    pub hotel_code: String,
    pub details: Option<DomainHotelDetailsWithoutRates>,
    pub loading: bool,
    pub error: Option<String>,
}

#[component]
pub fn WishlistPage() -> impl IntoView {
    let navigate = use_navigate();

    // Get wishlist hotel codes from auth state
    let wishlist_hotel_codes = move || AuthStateSignal::wishlist_hotel_codes();

    // Create signals for managing hotel details
    let (hotel_details_map, set_hotel_details_map) =
        create_signal(std::collections::HashMap::<String, WishlistHotelDetails>::new());

    // API client for fetching hotel details
    let api_client = ClientSideApiClient::new();

    // Effect to load hotel details when wishlist changes
    create_effect(move |_| {
        let hotel_codes = wishlist_hotel_codes();
        log!("Wishlist hotel codes: {:?}", hotel_codes);

        if hotel_codes.is_empty() {
            set_hotel_details_map.set(std::collections::HashMap::new());
            return;
        }

        // Initialize loading state for all hotels
        let mut initial_map = std::collections::HashMap::new();
        for hotel_code in &hotel_codes {
            initial_map.insert(
                hotel_code.clone(),
                WishlistHotelDetails {
                    hotel_code: hotel_code.clone(),
                    details: None,
                    loading: true,
                    error: None,
                },
            );
        }
        set_hotel_details_map.set(initial_map);

        // Fetch details for each hotel
        for hotel_code in hotel_codes {
            let hotel_code_clone = hotel_code.clone();
            let api_client_clone = api_client.clone();
            let set_hotel_details_map_clone = set_hotel_details_map.clone();

            spawn_local(async move {
                log!("Fetching details for hotel: {}", hotel_code_clone);

                match api_client_clone
                    .get_hotel_details_without_rates(&hotel_code_clone)
                    .await
                {
                    Ok(details) => {
                        log!(
                            "Successfully fetched details for hotel: {}",
                            hotel_code_clone
                        );
                        set_hotel_details_map_clone.update(|map| {
                            if let Some(hotel_detail) = map.get_mut(&hotel_code_clone) {
                                hotel_detail.details = Some(details);
                                hotel_detail.loading = false;
                                hotel_detail.error = None;
                            }
                        });
                    }
                    Err(e) => {
                        log!(
                            "Failed to fetch details for hotel {}: {}",
                            hotel_code_clone,
                            e
                        );
                        set_hotel_details_map_clone.update(|map| {
                            if let Some(hotel_detail) = map.get_mut(&hotel_code_clone) {
                                hotel_detail.loading = false;
                                hotel_detail.error = Some(e.to_string());
                            }
                        });
                    }
                }
            });
        }
    });

    view! {
        <div class="min-h-screen bg-slate-50">
            <Navbar blue_header=false />

            <div class="container mx-auto px-4 py-8">
                <div class="mb-8">
                    <h1 class="text-3xl font-bold text-gray-900 mb-2">"My Wishlist"</h1>
                    <p class="text-gray-600">"Your saved hotels"</p>
                </div>

                <Show
                    when=move || !AuthStateSignal::auth_state().get().is_authenticated()
                    fallback=|| view! { <></> }
                >
                    <div class="flex flex-col items-center justify-center py-12">
                        <div class="text-center max-w-md">
                            <h2 class="text-xl font-semibold text-gray-900 mb-4">
                                "Please sign in to view your wishlist"
                            </h2>
                            <p class="text-gray-600 mb-6">
                                "You need to be logged in to save and view your favorite hotels."
                            </p>
                            <button class="bg-blue-600 text-white px-6 py-3 rounded-lg font-medium hover:bg-blue-700 transition-colors">
                                "Sign In"
                            </button>
                        </div>
                    </div>
                </Show>

                <Show
                    when=move || AuthStateSignal::auth_state().get().is_authenticated()
                    fallback=|| view! { <></> }
                >
                    <Show
                        when=move || !AuthStateSignal::wishlist_hotel_codes().is_empty()
                        fallback=|| view! {
                            <div class="flex flex-col items-center justify-center py-12">
                                <div class="text-center max-w-md">
                                    <div class="w-24 h-24 mx-auto mb-6 bg-gray-100 rounded-full flex items-center justify-center">
                                        <svg class="w-12 h-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
                                        </svg>
                                    </div>
                                    <h2 class="text-xl font-semibold text-gray-900 mb-4">
                                        "Your wishlist is empty"
                                    </h2>
                                    <p class="text-gray-600 mb-6">
                                        "Start exploring hotels and save your favorites to see them here."
                                    </p>
                                    <a href="/" class="bg-blue-600 text-white px-6 py-3 rounded-lg font-medium hover:bg-blue-700 transition-colors">
                                        "Explore Hotels"
                                    </a>
                                </div>
                            </div>
                        }
                    >
                        <div class="space-y-4">
                            <For
                                each=move || {
                                    let details_map = hotel_details_map.get();
                                    details_map.values().cloned().collect::<Vec<_>>()
                                }
                                key=|hotel_detail| hotel_detail.hotel_code.clone()
                                let:hotel_detail
                            >
                                {
                                    if hotel_detail.loading {
                                        view! {
                                            <SkeletonCards />
                                        }.into_view()
                                    } else if let Some(ref error) = hotel_detail.error {
                                        view! {
                                            <div class="bg-red-50 border border-red-200 rounded-lg p-4">
                                                <div class="flex items-center">
                                                    <svg class="w-5 h-5 text-red-400 mr-2" fill="currentColor" viewBox="0 0 20 20">
                                                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                                                    </svg>
                                                    <p class="text-red-800">
                                                        "Failed to load hotel details for " {hotel_detail.hotel_code.clone()} ": " {error.clone()}
                                                    </p>
                                                </div>
                                            </div>
                                        }.into_view()
                                    } else if let Some(ref details) = hotel_detail.details {
                                        // Create a default image if none provided
                                        let img = if details.images.is_empty() {
                                            "https://via.placeholder.com/300x200?text=No+Image".to_string()
                                        } else {
                                            details.images[0].clone()
                                        };

                                        // Format address
                                        let address = if details.address.trim().is_empty() {
                                            if !details.city.trim().is_empty() && !details.country.trim().is_empty() {
                                                Some(format!("{}, {}", details.city, details.country))
                                            } else if !details.city.trim().is_empty() {
                                                Some(details.city.clone())
                                            } else {
                                                None
                                            }
                                        } else {
                                            Some(details.address.clone())
                                        };

                                        view! {
                                            <HotelCardTile
                                                img=img
                                                guest_score=None
                                                rating=details.star_rating as u8
                                                hotel_name=details.hotel_name.clone()
                                                hotel_code=details.hotel_code.clone()
                                                price=None // Wishlist doesn't show pricing
                                                discount_percent=None
                                                amenities=details.amenities.clone()
                                                property_type=None
                                                class="w-full mb-4 bg-white".to_string()
                                                hotel_address=address
                                                disabled=false
                                            />
                                        }.into_view()
                                    } else {
                                        view! {
                                            <div class="bg-gray-50 border border-gray-200 rounded-lg p-4">
                                                <p class="text-gray-600">"Hotel details not available"</p>
                                            </div>
                                        }.into_view()
                                    }
                                }
                            </For>
                        </div>
                    </Show>
                </Show>
            </div>

            <Footer />
        </div>
    }
}
