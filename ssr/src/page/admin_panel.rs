use crate::component::data_table_3::DataTableV3;
use leptos::prelude::*;

#[component]
pub fn AdminPanelPage() -> impl IntoView {
    view! {
       <DataTableV3 />
    }
    .into_any()
}
