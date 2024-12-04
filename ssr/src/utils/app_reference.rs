use chrono::Local;
use codee::string::JsonSerdeCodec;
use leptos::Signal;
use leptos_use::storage::{use_local_storage, UseStorageOptions}; // Import necessary modules
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AppReference(String);

impl AppReference {
    pub fn value(&self) -> String {
        self.0.clone()
    }
}

pub fn generate_app_reference() -> Signal<AppReference> {
    let today = Local::now().format("%d%m").to_string();
    let rand_num1: u32 = rand::thread_rng().gen_range(100000..999999);
    let rand_num2: u32 = rand::thread_rng().gen_range(100000..999999);
    let app_reference_string = format!("HB{}-{}-{}", today, rand_num1, rand_num2);
    let app_reference = app_reference_string.clone();
    let (state, set_state, _) = use_local_storage::<AppReference, JsonSerdeCodec>("app_reference");
    set_state(AppReference(app_reference_string));
    state
}
