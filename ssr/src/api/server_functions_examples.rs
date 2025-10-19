// <!-- Example server functions demonstrating the new hexagonal architecture -->

use leptos::prelude::*;
use std::sync::Arc;

use crate::domain::DomainHotelSearchCriteria;
use crate::application_services::{
    HotelService, 
    UISearchFilters, 
    UISortOptions, 
    DomainSortField, 
    DomainSortDirection
};
use crate::adapters::ProvabAdapter;
use crate::api::provab::Provab;
use crate::adapters::LiteApiAdapter;
use crate::api::liteapi::LiteApiHTTPClient;

// <!-- Example server function using the new architecture with filters -->
#[server(SearchHotelsFilteredAppService)]
pub async fn search_hotels_filtered_app_service(
    core_criteria: DomainHotelSearchCriteria,
    ui_filters: UISearchFilters,
    sort_options: UISortOptions,
) -> Result<crate::domain::DomainHotelSearchResponse, ServerFnError> {
    // <!-- 1. Create the provider adapter -->
    let liteapi_http_client = LiteApiHTTPClient::default();
    let liteapi_adapter = Arc::new(LiteApiAdapter::new(liteapi_http_client));
    
    // <!-- 2. Create the hotel service -->
    let hotel_service = HotelService::new(liteapi_adapter);

    // <!-- 3. Call the service with filters -->
    hotel_service.search_hotels_with_filters(core_criteria, ui_filters, sort_options).await
        .map_err(|e| ServerFnError::ServerError(format!("Service error: {:?}", e)))
}

// <!-- Example server function with preset filters for common use cases -->
#[server(SearchHotelsLuxury)]
pub async fn search_hotels_luxury(
    core_criteria: DomainHotelSearchCriteria,
) -> Result<crate::domain::DomainHotelSearchResponse, ServerFnError> {
    // <!-- Preset filters for luxury hotels -->
    let luxury_filters = UISearchFilters {
        min_star_rating: Some(4), // <!-- 4+ star hotels -->
        min_price_per_night: Some(200.0), // <!-- Minimum luxury price point -->
        ..Default::default()
    };
    
    let sort_options = UISortOptions {
        sort_by: Some(DomainSortField::Rating),
        sort_direction: Some(DomainSortDirection::Descending), // <!-- Best rating first -->
    };

    search_hotels_filtered_app_service(core_criteria, luxury_filters, sort_options).await
}

// <!-- Example server function with preset filters for budget hotels -->
#[server(SearchHotelsBudget)]
pub async fn search_hotels_budget(
    core_criteria: DomainHotelSearchCriteria,
) -> Result<crate::domain::DomainHotelSearchResponse, ServerFnError> {
    // <!-- Preset filters for budget hotels -->
    let budget_filters = UISearchFilters {
        max_price_per_night: Some(100.0), // <!-- Budget price point -->
        ..Default::default()
    };
    
    let sort_options = UISortOptions {
        sort_by: Some(DomainSortField::Price),
        sort_direction: Some(DomainSortDirection::Ascending), // <!-- Cheapest first -->
    };

    search_hotels_filtered_app_service(core_criteria, budget_filters, sort_options).await
}

// <!-- Example usage in a component (pseudo-code) -->
/*
pub fn SearchComponent() -> impl IntoView {
    let search_action = create_server_action::<SearchHotelsFilteredAppService>();
    
    view! {
        <form on:submit=move |ev| {
            ev.prevent_default();
            
            let core_criteria = DomainHotelSearchCriteria {
                destination_city_id: 1254, // Mumbai
                destination_country_code: "IN".into(),
                check_in_date: "25-12-2024".into(),
                no_of_nights: 2,
                no_of_rooms: 1,
                room_guests: vec![/* ... */],
                guest_nationality: "IN".into(),
            };
            
            let filters = UISearchFilters {
                min_star_rating: Some(3),
                max_price_per_night: Some(500.0),
                hotel_name_search: Some("Taj".into()),
                ..Default::default()
            };
            
            let sort_options = UISortOptions::price_low_to_high();
            
            search_action.dispatch(SearchHotelsFilteredAppServiceArgs {
                core_criteria,
                ui_filters: filters,
                sort_options,
            });
        }>
            <input type="submit" value="Search Hotels"/>
        </form>
        
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || {
                search_action.value().get().map(|result| {
                    match result {
                        Ok(response) => view! { 
                            <div>"Found hotels: " {response.hotel_results().len()}</div>
                        }.into_any(),
                        Err(e) => view! { 
                            <div>"Error: " {e.to_string()}</div>
                        }.into_any(),
                    }
                })
            }}
        </Suspense>
    }
}
*/