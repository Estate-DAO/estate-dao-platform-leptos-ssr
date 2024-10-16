use crate::{
    api::{HotelSearchRequest, HotelSearchResponse},
    component::{GuestSelection, SelectedDateRange},
};
use leptos::RwSignal;
use leptos::*;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct HotelViewInfoCtx {
    pub hotel_code: RwSignal<Option<String>>,
}
