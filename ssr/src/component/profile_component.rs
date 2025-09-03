use codee::string::JsonSerdeCodec;
use leptos::*;
use leptos_use::{use_cookie_with_options, UseCookieOptions};

use crate::{
    api::{
        consts::USER_IDENTITY,
    },
    component::{profile_dropdown::ProfileDropdown, user_profile_icon::UserProfileIcon},
};
