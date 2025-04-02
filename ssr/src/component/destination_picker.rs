use leptos::*;
use serde::{Deserialize, Serialize};

use crate::{
    component::{Divider, HSettingIcon},
    state::{
        input_group_state::{InputGroupState, OpenDialogComponent},
        search_state::SearchCtx,
    },
};
// use leptos::logging::log;
use crate::log;
use leptos_icons::*;
use std::time::Duration;

use leptos_query::{query_persister, *};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Destination {
    #[serde(rename = "city_name")]
    pub city: String,
    pub country_name: String,
    pub country_code: String,
    #[serde(rename = "city_code")]
    pub city_id: String,
}

impl Default for Destination {
    fn default() -> Self {
        Self {
            city: "Goa".into(),
            country_name: "IN".into(),
            city_id: "1254".into(),
            country_code: "IN".into(),
        }
    }
}
impl Destination {
    pub fn get_country_code(ctx: &SearchCtx) -> String {
        ctx.destination
            .get_untracked()
            .map(|d| d.country_code.clone())
            .unwrap_or_default()
    }

    pub fn get_city_id(ctx: &SearchCtx) -> u32 {
        ctx.destination
            .get_untracked()
            .map(|d| d.city_id.parse::<u32>().unwrap_or_default())
            .unwrap_or_default()
    }
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

fn destinations_query() -> QueryScope<bool, Option<Vec<Destination>>> {
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

#[component]
pub fn DestinationPicker() -> impl IntoView {
    let is_open = create_memo(move |_| {
        // log!("is_open called");
        InputGroupState::is_destination_open()
    });
    let search_ctx: SearchCtx = expect_context();

    // let destinations_resource = create_resource(is_open , move  |_| async move { read_destinations_from_file("city.json".into()).await});

    let QueryResult {
        data: destinations_resource,
        state,
        // is_loading,
        // is_fetching,
        // is_invalid,
        ..
    } = destinations_query().use_query(move || is_open.get());

    let display_value = create_memo(move |_| {
        search_ctx
            .destination
            .get()
            .map(|d| format!("{}, {}", d.city, d.country_name))
            .unwrap_or_else(|| "Where to?".to_string())
    });

    view! {

        <div class="absolute inset-0 flex items-center justify-center">
        <div class="w-full h-full flex items-center" on:click=move |_| InputGroupState::toggle_dialog(OpenDialogComponent::CityListComponent)>
            <input
                type="text"
                placeholder=""
                class="w-full h-full text-gray-800 bg-transparent border-none focus:outline-none text-sm font-semibold text-center"
                prop:value=display_value
                readonly=true
            />
        </div>

        <Show when=move || is_open()>
            <div class="absolute top-full left-0 mt-2 w-80 bg-white border border-gray-200 rounded-xl shadow-lg z-50">
                <div class="p-4">
                    <div class="space-y-4">
                        <Suspense fallback=move || {
                            view! { <p>"Loading..."</p> }
                        }>
                            {move || {
                                destinations_resource
                                    .get()
                                    .map(|dest_vec| {
                                        // log!("{dest_vec:?}");
                                        view! {
                                            <ShowDestinations
                                                dest_vec=dest_vec.unwrap_or_default()
                                            />
                                        }
                                    })
                            }}
                        </Suspense>
                    </div>
                </div>
            </div>
        </Show>
    </div>
    }
}

#[component]
fn ShowDestinations(dest_vec: Vec<Destination>) -> impl IntoView {
    view! {
        <div class="h-80 custom-scrollbar">
            {move || {
                dest_vec
                    .clone()
                    .into_iter()
                    .map(|dest| {
                        let country = dest.country_name.clone();
                        let city = dest.city.clone();
                        view! {
                            <div
                                class="cursor-pointer hover:bg-gray-50 p-2 rounded"
                                on:click=move |_| {
                                    SearchCtx::set_destination(dest.clone());
                                    InputGroupState::toggle_dialog(OpenDialogComponent::CityListComponent);
                                }
                            >
                                <span class="text-gray-800">
                                    {format!("{}, {}", &city, &country)}
                                </span>
                            </div>
                            <Divider />
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}
