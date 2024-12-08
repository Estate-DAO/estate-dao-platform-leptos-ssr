pub mod admin;
pub mod app_reference;
pub mod backend_default_impl;
pub mod ic;
pub mod icon;
pub mod parent_resource;
pub mod route;

pub use backend_default_impl::*;

pub fn pluralize(count: u32, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("{} {}", count, singular)
    } else {
        format!("{} {}", count, plural)
    }
}
