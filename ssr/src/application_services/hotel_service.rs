use crate::adapters::ProvabAdapter;
use crate::api::provab::Provab;
use crate::api::ApiError;
use crate::application_services::filter_types::{
    DomainSortDirection, DomainSortField, UISearchFilters, UISortOptions,
};
use crate::domain::{
    DomainHotelDetails, DomainHotelInfoCriteria, DomainHotelListAfterSearch,
    DomainHotelSearchCriteria,
};
use crate::ports::hotel_provider_port::{HotelProviderPort, ProviderError};
use std::cmp::Ordering;
use std::sync::Arc;

// #[derive(Clone)]
// pub struct HotelService {
//     provider: Arc<dyn  HotelProviderPort+ Send+ Sync>,
// }

#[derive(Clone)]
pub struct HotelService<T: HotelProviderPort + Clone> {
    provider: T,
}

impl<T: HotelProviderPort + Clone> HotelService<T> {
    pub fn init(provider: T) -> Self {
        // todo (provab) is the default adaptyer for now. we will test this later.
        // let provab_client = Provab::default();
        // let provab_adapter  = ProvabAdapter::new(provab_client);
        HotelService::new(provider)
    }

    pub fn new(provider: T) -> Self {
        Self { provider }
    }

    pub async fn search_hotels_with_filters(
        &self,
        core_criteria: DomainHotelSearchCriteria,
        ui_filters: UISearchFilters,
        sort_options: UISortOptions,
    ) -> Result<DomainHotelListAfterSearch, ProviderError> {
        // <!-- 1. Call the provider with core criteria and UI filters -->
        // <!-- The adapter will try to use UI filters if its specific API supports them -->
        let domain_result = self
            .provider
            .search_hotels(core_criteria, ui_filters)
            // .await
            .await?;

        // todo (filtering) is not implemented at the moment
        // filtering is of two types. API based (ui filters get transformed to query params) or application based (filtering is done in the application)

        // <!-- 2. Apply any UI filters that were NOT handled by the provider -->
        // <!-- For simplicity, we always apply certain filters here for consistency -->
        // if let Some(ref mut search) = domain_result.search {
        //     let hotel_results_vec = &mut search.hotel_search_result.hotel_results;

        //     // // <!-- Apply star rating filter -->
        //     // if let Some(min_rating) = ui_filters.min_star_rating {
        //     //     hotel_results_vec.retain(|hotel| hotel.star_rating >= min_rating);
        //     // }

        //     // // <!-- Apply price range filters -->
        //     // if let Some(max_price) = ui_filters.max_price_per_night {
        //     //     hotel_results_vec.retain(|hotel| hotel.price.room_price <= max_price);
        //     // }
        //     // if let Some(min_price) = ui_filters.min_price_per_night {
        //     //     hotel_results_vec.retain(|hotel| hotel.price.room_price >= min_price);
        //     // }

        //     // // <!-- Apply hotel name search filter -->
        //     // if let Some(ref search_name) = ui_filters.hotel_name_search {
        //     //     if !search_name.is_empty() {
        //     //         let search_name_lower = search_name.to_lowercase();
        //     //         hotel_results_vec.retain(|hotel| {
        //     //             hotel.hotel_name.to_lowercase().contains(&search_name_lower)
        //     //         });
        //     //     }
        //     // }

        //     // <!-- TODO: Apply amenities and property types filtering when available in domain model -->

        //     // <!-- 3. Apply Sorting -->
        //     // if let Some(sort_field) = sort_options.sort_by {
        //     //     Self::sort_hotels_by_field(hotel_results_vec, sort_field, sort_options.sort_direction);
        //     // }
        // }

        Ok(domain_result)
    }

    // <!-- Keep the original method for backward compatibility -->
    pub async fn search_hotels(
        &self,
        criteria: DomainHotelSearchCriteria,
    ) -> Result<DomainHotelListAfterSearch, ProviderError> {
        // <!-- Use the new method with empty filters -->
        self.search_hotels_with_filters(
            criteria,
            UISearchFilters::default(),
            UISortOptions::default(),
        )
        .await
    }

    pub async fn get_hotel_details(
        &self,
        criteria: DomainHotelInfoCriteria,
    ) -> Result<DomainHotelDetails, ProviderError> {
        // <!-- Core business logic for getting hotel details can be added here -->
        // <!-- For now, we just delegate to the provider -->
        self.provider.get_hotel_details(criteria).await
    }

    // <!-- Future methods for room operations, booking, etc. -->
    // pub async fn get_room_options(&self, hotel_id: String, token: String) -> Result<DomainRoomOptions, ProviderError> {
    //     self.provider.get_room_options(hotel_id, token).await
    // }

    // pub async fn block_room(&self, block_request: DomainBlockRoomRequest) -> Result<DomainBlockRoomResponse, ProviderError> {
    //     self.provider.block_room(block_request).await
    // }

    // pub async fn book_room(&self, book_request: DomainBookRoomRequest) -> Result<DomainBookRoomResponse, ProviderError> {
    //     self.provider.book_room(book_request).await
    // }

    // <!-- Helper methods for filtering and sorting -->

    // fn sort_hotels_by_field(
    //     hotels: &mut Vec<DomainHotelResult>,
    //     sort_field: DomainSortField,
    //     sort_direction: Option<DomainSortDirection>
    // ) {
    //     hotels.sort_by(|a, b| {
    //         let comparison = match sort_field {
    //             DomainSortField::Price => a.price.room_price.partial_cmp(&b.price.room_price).unwrap_or(Ordering::Equal),
    //             DomainSortField::Rating => {
    //                 // Default to descending for rating (best first)
    //                 b.star_rating.cmp(&a.star_rating)
    //             }
    //             DomainSortField::Name => {
    //                 a.hotel_name.cmp(&b.hotel_name)
    //             }
    //         };

    //         // Apply sort direction
    //         match sort_direction.as_ref().unwrap_or(&DomainSortDirection::Ascending) {
    //             DomainSortDirection::Descending => comparison.reverse(),
    //             DomainSortDirection::Ascending => comparison,
    //         }
    //     });
    // }
}
