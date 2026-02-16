use leptos::*;

use crate::{
    application_services::HotelService,
    domain::{DomainHotelInfoCriteria, DomainHotelStaticDetails},
    init::get_provider_registry,
};

#[server(GetHotelStaticDetailsApi, "/api")]
pub async fn get_hotel_static_details_api(
    hotel_id: String,
) -> Result<DomainHotelStaticDetails, ServerFnError> {
    let provider = get_provider_registry().hotel_provider();
    let hotel_service = HotelService::new(provider);

    hotel_service
        .get_hotel_static_details(&hotel_id)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server(GetHotelRatesApi, "/api")]
pub async fn get_hotel_rates_api(
    criteria: DomainHotelInfoCriteria,
) -> Result<crate::domain::DomainGroupedRoomRates, ServerFnError> {
    let provider = get_provider_registry().hotel_provider();
    let hotel_service = HotelService::new(provider);

    hotel_service
        .get_hotel_rates(criteria)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}
