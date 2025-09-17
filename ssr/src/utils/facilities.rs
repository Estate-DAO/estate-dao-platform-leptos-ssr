use crate::{log, utils::facilities};
use leptos::*;
use serde::*;

#[derive(Clone)]
pub struct Facilities {
    pub facilities: RwSignal<Vec<Facility>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Facility {
    pub facility_id: u32,
    pub facility: String,
    pub sort: u32,
}

#[server]
pub async fn read_facilities_from_file() -> Result<Vec<Facility>, ServerFnError> {
    let file_path = "facilities.json".to_string();
    let file = std::fs::File::open(file_path.as_str())?;
    let reader = std::io::BufReader::new(file);
    let result: Vec<Facility> = serde_json::from_reader(reader)?;
    log!(" read_facilities_from_file - {:?}", result.first());
    Ok(result)
}

impl Facilities {
    pub fn init() -> Self {
        let this = Self {
            facilities: RwSignal::new(Vec::new()),
        };
        provide_context(this.clone());
        this
    }

    pub async fn fetch() {
        match read_facilities_from_file().await {
            Ok(facilities) => {
                Self::set(facilities);
            }
            Err(e) => {
                log!("Error reading facilities from file: {}", e);
            }
        }
    }

    pub fn set(facilities: Vec<Facility>) {
        let facilities_context = use_context::<Self>().unwrap_or_else(Self::init);
        facilities_context.facilities.set(facilities);
    }

    fn get() -> Self {
        let this =  use_context::<Self>().unwrap_or_else(Self::init);
        if this.facilities.get().is_empty() {
            spawn_local(async move {
                Self::fetch().await;
            });
        }
        this
    }

    pub fn get_by_facility_id(id: u64) -> Option<Facility> {
        let this = Self::get();
        this.facilities.get().iter().find(|f| f.facility_id as u64 == id).cloned()
    }
}
