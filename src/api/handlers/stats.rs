//! # Stats Handler
//!
//! آمار و گزارشات

use axum::{
    extract::State,
    Json,
};

use crate::{
    error::Result,
    models::ApiResponse,
    services::AppState,
    database::UrlStats,
};

// =====================================
// Get Stats
// =====================================
/// گرفتن آمار کلی
///
/// # Endpoint
/// `GET /api/stats`
pub async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<UrlStats>>> {
    let stats = state.url_service.get_stats().await?;
    
    Ok(Json(ApiResponse::success(stats)))
}

