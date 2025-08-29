mod destination_picker_v5;
pub use destination_picker_v5::DestinationPickerV5;

use leptos::html::Input;
use leptos::*;
use leptos_use::on_click_outside;
use serde::{Deserialize, Serialize};

use crate::{
    component::{Divider, HSettingIcon, LiveSelect},
    view_state_layer::{
        input_group_state::{InputGroupState, OpenDialogComponent},
        // search_state::SearchCtx,
        GlobalStateForLeptos,
    },
};
// use leptos::logging::log;
use crate::log;
use leptos_icons::*;
use std::time::Duration;

use leptos_query::{query_persister, *};

pub fn destinations_query() -> QueryScope<bool, Option<Vec<Destination>>> {
    // log!("destinations_query called");
    leptos_query::create_query(
        |_| async move {
            // log!("will call read_destinations_from_file in async move");
            read_destinations_from_file("city.json".into()).await.ok()
        },
        QueryOptions {
            default_value: None,
            refetch_interval: None,
            resource_option: Some(ResourceOption::NonBlocking),
            stale_time: Some(Duration::from_secs(2 * 60)),
            gc_time: Some(Duration::from_secs(5 * 60)),
        },
    )
}

#[server(GetCityList)]
pub async fn read_destinations_from_file(
    file_path: String,
) -> Result<Vec<Destination>, ServerFnError> {
    let file = std::fs::File::open(file_path.as_str())?;
    let reader = std::io::BufReader::new(file);
    let result: Vec<Destination> = serde_json::from_reader(reader)?;
    log!("{:?}", result.first());

    // let result = vec![Destination::default()];
    // log!("read destinations from file called");
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Destination {
    #[serde(rename = "city_name")]
    pub city: String,
    pub country_name: String,
    pub country_code: String,
    #[serde(rename = "city_code")]
    pub city_id: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl Default for Destination {
    fn default() -> Self {
        Self {
            city: "New Delhi".into(),
            country_name: "IN".into(),
            city_id: "5100".into(),
            country_code: "IN".into(),
            latitude: Some(28.6139391),
            longitude: Some(77.2090212),
        }
    }
}

// impl Destination {
//     pub fn get_country_code(ctx: &SearchCtx) -> String {
//         ctx.destination
//             .get_untracked()
//             .map(|d| d.country_code.clone())
//             .unwrap_or_default()
//     }

//     pub fn get_city_id(ctx: &SearchCtx) -> u32 {
//         ctx.destination
//             .get_untracked()
//             .map(|d| d.city_id.parse::<u32>().unwrap_or_default())
//             .unwrap_or_default()
//     }
// }
