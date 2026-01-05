use leptos::*;
use std::sync::Arc;

use crate::{
    application_services::HotelService,
    domain::{DomainHotelInfoCriteria, DomainHotelStaticDetails},
    init::get_liteapi_driver,
    ports::hotel_provider_port::ProviderError,
};

#[server(GetHotelStaticDetailsApi, "/api")]
pub async fn get_hotel_static_details_api(
    hotel_id: String,
) -> Result<DomainHotelStaticDetails, ServerFnError> {
    let liteapi_driver = Arc::new(get_liteapi_driver());
    let hotel_service = HotelService::new(liteapi_driver);

    hotel_service
        .get_hotel_static_details(&hotel_id)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server(GetHotelRatesApi, "/api")]
pub async fn get_hotel_rates_api(
    criteria: DomainHotelInfoCriteria,
) -> Result<Vec<crate::domain::DomainRoomOption>, ServerFnError> {
    let liteapi_driver = Arc::new(get_liteapi_driver());
    let hotel_service = HotelService::new(liteapi_driver);

    hotel_service
        .get_hotel_rates(criteria)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}
