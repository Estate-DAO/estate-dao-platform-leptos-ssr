use axum::response::IntoResponse;
use estate_fe::app::AppRoutes;

pub async fn sitemap_handler() -> impl IntoResponse {
    AppRoutes::generate_sitemap()
}
