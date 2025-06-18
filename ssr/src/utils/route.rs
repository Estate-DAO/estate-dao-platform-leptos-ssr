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
