use leptos::*;
use leptos_router::use_navigate;

use crate::api::auth::auth_state::AuthStateSignal;
use crate::api::client_side_api::ClientSideApiClient;
use crate::app::AppRoutes;
use crate::component::{Footer, Navbar, SkeletonCards};
use crate::domain::DomainHotelDetailsWithoutRates;
use crate::log;
use crate::page::HotelCardTile;

#[component]
pub fn WishlistPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-slate-50">
            <Navbar blue_header=false />
            <div class="mb-8">
                        <h1 class="text-3xl font-bold text-gray-900 mb-2">"My Wishlist"</h1>
                        <p class="text-gray-600">"Your saved hotels"</p>
            </div>
            <WishlistComponent />
            <Footer />
        </div>
    }
}

#[component]
pub fn WishlistComponent() -> impl IntoView {
    let navigate = use_navigate();

    // Get wishlist hotel codes from auth state
    let wishlist_hotel_codes = move || AuthStateSignal::wishlist_hotel_codes();

    // API client for fetching hotel details
    let api_client = ClientSideApiClient::new();

    let wishlist_details = Resource::local(
        move || AuthStateSignal::auth_state().get(),
        move |auth| async move {
            if auth.is_authenticated() {
                return Some(auth);
            }

            let url = format!("/api/user-wishlist");
            match gloo_net::http::Request::get(&url).send().await {
                Ok(response) => {
                    if response.status() == 200 {
                        if let Ok(user_data) = response.json::<Vec<String>>().await {
                            logging::log!("Fetched wishlist: {:?}", user_data);
                            AuthStateSignal::wishlist_set(Some(user_data));
                        }
                    }
                }
                Err(e) => {
                    logging::log!("Failed to fetch wishlist: {:?}", e);
                }
            }
            None
        },
    );

    // Create a resource that fetches all hotel details
    let hotel_details_resource = create_resource(
        move || wishlist_hotel_codes(),
        move |hotel_codes| {
            let api_client = api_client.clone();
            async move {
                if hotel_codes.is_empty() {
                    return Vec::new();
                }

                log!("Fetching details for hotels: {:?}", hotel_codes);

                let mut hotel_details = Vec::new();

                for hotel_code in hotel_codes {
                    match api_client
                        .get_hotel_details_without_rates(&hotel_code)
                        .await
                    {
                        Ok(details) => {
                            log!("Successfully fetched details for hotel: {}", hotel_code);
                            hotel_details.push(details);
                        }
                        Err(e) => {
                            log!("Failed to fetch details for hotel {}: {}", hotel_code, e);
                            // You could choose to continue with other hotels or handle errors differently
                        }
                    }
                }

                hotel_details
            }
        },
    );

    view! {
        <div class="container mx-auto px-4 py-8">
                {move || wishlist_details.get().map(|_| view! {
                    <></>
                })}

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
                            <Suspense
                                fallback=move || view! {
                                    <div class="space-y-4">
                                        <SkeletonCards />
                                        <SkeletonCards />
                                        <SkeletonCards />
                                    </div>
                                }
                            >
                                {move || {
                                    hotel_details_resource.get().map(|hotel_details| view! {
                                        <For
                                            each=move || hotel_details.clone()
                                            key=|hotel| hotel.hotel_code.clone()
                                            let:hotel
                                        >
                                            <HotelCardTile
                                                img={
                                                    if hotel.images.is_empty() {
                                                        "https://via.placeholder.com/300x200?text=No+Image".to_string()
                                                    } else {
                                                        hotel.images[0].clone()
                                                    }
                                                }
                                                guest_score=None
                                                rating=hotel.star_rating as u8
                                                hotel_name=hotel.hotel_name.clone()
                                                hotel_code=hotel.hotel_code.clone()
                                                price=None // Wishlist doesn't show pricing
                                                discount_percent=None
                                                amenities=hotel.amenities.clone()
                                                property_type=None
                                                class="w-full mb-4 bg-white".to_string()
                                                hotel_address={
                                                    if hotel.address.trim().is_empty() {
                                                        if !hotel.city.trim().is_empty() && !hotel.country.trim().is_empty() {
                                                            Some(format!("{}, {}", hotel.city, hotel.country))
                                                        } else if !hotel.city.trim().is_empty() {
                                                            Some(hotel.city.clone())
                                                        } else {
                                                            None
                                                        }
                                                    } else {
                                                        Some(hotel.address.clone())
                                                    }
                                                }
                                                disabled=false
                                            />
                                        </For>
                                    })
                                }}
                            </Suspense>
                        </div>
                    </Show>
                </Show>
            </div>
    }
}
