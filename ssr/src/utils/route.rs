use std::fmt::Display;

use leptos_router::use_navigate;
use reqwest::Url;

use crate::state::input_group_state::{InputGroupState, OpenDialogComponent};

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
    let base_url = Url::parse(base).map_err(|e| format!("Invalid base URL: {}", e))?;

    // Use the join method on Url
    let full_url = base_url
        .join(path)
        .map_err(|e| format!("Invalid path: {}", e))?;

    Ok(full_url.to_string())
}
