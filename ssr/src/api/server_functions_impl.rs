use leptos::*;
use std::sync::Arc;

use crate::{
    adapters::LiteApiAdapter,
    api::liteapi::LiteApiHTTPClient,
    application_services::HotelService,
    domain::{DomainHotelInfoCriteria, DomainHotelStaticDetails},
    ports::hotel_provider_port::ProviderError,
};

#[server(GetHotelStaticDetailsApi, "/api")]
pub async fn get_hotel_static_details_api(
    hotel_id: String,
) -> Result<DomainHotelStaticDetails, ServerFnError> {
    let liteapi_http_client = LiteApiHTTPClient::default();
    let liteapi_adapter = Arc::new(LiteApiAdapter::new(liteapi_http_client));
    let hotel_service = HotelService::new(liteapi_adapter);

    hotel_service
        .get_hotel_static_details(&hotel_id)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server(GetHotelRatesApi, "/api")]
pub async fn get_hotel_rates_api(
    criteria: DomainHotelInfoCriteria,
) -> Result<Vec<crate::domain::DomainRoomOption>, ServerFnError> {
    let liteapi_http_client = LiteApiHTTPClient::default();
    let liteapi_adapter = Arc::new(LiteApiAdapter::new(liteapi_http_client));
    let hotel_service = HotelService::new(liteapi_adapter);

    hotel_service
        .get_hotel_rates(criteria)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}
