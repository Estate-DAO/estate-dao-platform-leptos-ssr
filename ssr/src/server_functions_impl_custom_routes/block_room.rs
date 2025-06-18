use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use estate_fe::domain::DomainBlockRoomRequest;
use estate_fe::view_state_layer::AppState;
use serde_json::json;

use super::{call_block_room_api, parse_json_request};

#[axum::debug_handler]
pub async fn block_room_api_server_fn_route(
    State(state): State<AppState>,
    body: String,
) -> Result<Response, Response> {
    // <!-- Parse input string to struct -->
    let request: DomainBlockRoomRequest = parse_json_request(&body)?;

    // Use the shared helper function for block room API call
    let block_response = call_block_room_api(&state, request).await.map_err(|e| {
        tracing::error!("Block room failed: {:?}", e);
        let error_response = json!({
            "error": format!("Block room failed: {}", e)
        });
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            error_response.to_string(),
        )
            .into_response()
    })?;

    // <!-- Serialize response to string -->
    let json_string = serde_json::to_string(&block_response).map_err(|e| {
        tracing::error!("Failed to serialize block room response: {:?}", e);
        let error_response = json!({
            "error": format!("Failed to serialize response: {}", e)
        });
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            error_response.to_string(),
        )
            .into_response()
    })?;

    Ok((StatusCode::OK, json_string).into_response())
}
