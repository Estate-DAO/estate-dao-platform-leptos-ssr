// use serde::{Deserialize, Serialize};
// use application_services::{UISearchFilters, UISortOptions};
// use view_models::HotelCardViewModel;

// /// Web DTO for hotel search requests
// /// This is the boundary between the web layer and application layer
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SearchHotelsWebRequest {
//     // Destination information
//     pub destination_city_id: u32,
//     pub destination_city_name: String,
//     pub destination_country_code: String,
//     pub destination_country_name: String,
    
//     // Date information
//     pub check_in_date: String,  // Format: "YYYY-MM-DD"
//     pub check_out_date: String, // Format: "YYYY-MM-DD"
//     pub no_of_nights: u32,
    
//     // Guest information
//     pub no_of_rooms: u32,
//     pub room_guests: Vec<WebRoomGuest>,
//     pub guest_nationality: String,
    
//     // Filters and sorting
//     pub filters: UISearchFilters,
//     pub sorting: UISortOptions,
// }

// /// Web DTO for room guest information
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct WebRoomGuest {
//     pub no_of_adults: u32,
//     pub no_of_child: u32,
//     pub child_age: Vec<u32>,
// }

// /// Web DTO for hotel details request
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct HotelDetailsWebRequest {
//     pub token: String,
// }

// impl SearchHotelsWebRequest {
//     /// Convert web DTO to domain search criteria
//     pub fn into_domain_criteria(self) -> Result<domain::travel::DomainHotelSearchCriteria, String> {
//         if self.no_of_nights == 0 {
//             return Err("Number of nights must be greater than 0".to_string());
//         }
        
//         if self.room_guests.is_empty() {
//             return Err("At least one room guest is required".to_string());
//         }
        
//         Ok(domain::travel::DomainHotelSearchCriteria {
//             check_in_date: self.check_in_date,
//             no_of_nights: self.no_of_nights,
//             country_code: self.destination_country_code,
//             city_id: self.destination_city_id,
//             guest_nationality: self.guest_nationality,
//             no_of_rooms: self.no_of_rooms,
//             room_guests: self.room_guests.into_iter().map(|guest| {
//                 domain::travel::DomainRoomGuest {
//                     no_of_adults: guest.no_of_adults,
//                     no_of_child: guest.no_of_child,
//                     child_age: guest.child_age,
//                 }
//             }).collect(),
//         })
//     }
// }

// impl HotelDetailsWebRequest {
//     /// Convert web DTO to domain criteria
//     pub fn into_domain_criteria(self) -> domain::travel::DomainHotelInfoCriteria {
//         domain::travel::DomainHotelInfoCriteria {
//             token: self.token,
//         }
//     }
// }