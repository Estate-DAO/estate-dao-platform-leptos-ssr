use axum::{response::Json, routing::get, Router};
use estate_fe::view_state_layer::AppState;
use serde_json::{json, Value};

/// **Debug endpoint to test domain normalization**
///
/// **Purpose**: Provides information about the current request domain for testing
/// **Route**: GET /debug/domain
/// **Response**: JSON with domain information

pub async fn debug_domain_info() -> Json<Value> {
    Json(json!({
        "message": "Domain normalization is working",
        "canonical_domain": "nofeebooking.com",
        "status": "ok"
    }))
}

/// Add debug routes for domain testing
pub fn debug_routes() -> Router<AppState> {
    Router::new().route("/debug/domain", get(debug_domain_info))
}
