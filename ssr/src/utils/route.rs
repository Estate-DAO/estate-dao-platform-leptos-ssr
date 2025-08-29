use std::fmt::Display;

use leptos_router::use_navigate;
use reqwest::Url;

use crate::view_state_layer::input_group_state::{InputGroupState, OpenDialogComponent};

#[macro_export]
macro_rules! try_or_redirect {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                use $crate::utils::route::failure_redirect;
                failure_redirect(e);
                return;
            }
        }
    };
}

#[macro_export]
macro_rules! try_or_redirect_opt {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                use $crate::utils::route::failure_redirect;
                failure_redirect(e);
                return None;
            }
        }
    };
}

pub fn failure_redirect<E: Display>(err: E) {
    let nav = use_navigate();
    nav(&format!("/error?err={err}"), Default::default());
}

pub fn go_to_root() {
    let nav = use_navigate();
    nav("/", Default::default());

    // close all the dialogs
    InputGroupState::toggle_dialog(OpenDialogComponent::None);
}

pub fn join_base_and_path_url(base: &str, path: &str) -> Result<String, String> {
    // Parse the base URL first
    let mut base_url = Url::parse(base).map_err(|e| format!("Invalid base URL: {}", e))?;

    // Get the existing path from base URL and append the new path
    let mut full_path = base_url.path().trim_end_matches('/').to_string();
    if !path.starts_with('/') {
        full_path.push('/');
    }
    full_path.push_str(path);

    // Set the combined path
    base_url.set_path(&full_path);

    Ok(base_url.to_string())
}

/// Joins a path with query parameters to create a complete URL
/// Similar to join_base_and_path_url but for adding query strings to paths
/// Properly merges new query parameters with any existing ones in the path
///
/// # Arguments
/// * `path` - The base path (e.g., "/hotel-list" or "/hotel-list?existing=param")
/// * `query_params` - Iterator of key-value pairs for query parameters to add/merge
///
/// # Returns
/// * `Result<String, String>` - Complete URL with merged query parameters or error
///
/// # Example
/// ```
/// // Path without existing query params
/// let params = vec![("state", "encoded_state"), ("page", "1")];
/// let url = join_path_and_query_params("/hotel-list", &params)?;
/// // Result: "/hotel-list?state=encoded_state&page=1"
///
/// // Path with existing query params - merges them
/// let params = vec![("page", "2"), ("sort", "price")];
/// let url = join_path_and_query_params("/hotel-list?state=abc", &params)?;
/// // Result: "/hotel-list?state=abc&page=2&sort=price"
/// ```
pub fn join_path_and_query_params<I, K, V>(path: &str, query_params: I) -> Result<String, String>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    // Create a dummy base URL to properly parse the path and existing query params
    let dummy_base = "http://example.com";
    let full_dummy_url = if path.starts_with('/') {
        format!("{}{}", dummy_base, path)
    } else {
        format!("{}/{}", dummy_base, path)
    };

    // Parse the full URL to extract existing query parameters
    let mut url = Url::parse(&full_dummy_url).map_err(|e| format!("URL parsing error: {}", e))?;

    // Get existing query parameters and convert them to a Vec for manipulation
    let mut all_params: Vec<(String, String)> = url.query_pairs().into_owned().collect();

    // Add new query parameters (this will append, not overwrite)
    // If you want to overwrite duplicate keys, you could implement that logic here
    for (key, value) in query_params {
        all_params.push((key.as_ref().to_string(), value.as_ref().to_string()));
    }

    // Clear existing query and rebuild with merged parameters
    url.set_query(None);

    if !all_params.is_empty() {
        let query_string = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(&all_params)
            .finish();
        url.set_query(Some(&query_string));
    }

    // Extract just the path and query from the full URL
    let path_with_query = if let Some(query) = url.query() {
        format!("{}?{}", url.path(), query)
    } else {
        url.path().to_string()
    };

    Ok(path_with_query)
}
