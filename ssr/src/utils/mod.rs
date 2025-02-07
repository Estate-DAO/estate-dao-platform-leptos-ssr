pub mod admin;
pub mod app_reference;
pub mod backend_default_impl;
mod colored_print;
pub mod date;
pub mod ic;
pub mod icon;
pub mod parent_resource;
pub mod route;
pub mod sort_json;

pub use backend_default_impl::*;
pub use colored_print::*;

pub fn pluralize(count: u32, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("{} {}", count, singular)
    } else {
        format!("{} {}", count, plural)
    }
}
