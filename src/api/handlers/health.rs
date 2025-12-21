//! # Health Check Handler
//!
//! برای بررسی سلامت سرویس

use axum::{
    extract::State,
    Json,
};

use crate::{
    error::Result,
    models::HealthResponse,
    services::AppState,
};

// =====================================
// Health Check
// =====================================
/// بررسی سلامت سرویس
///
/// # مفاهیم:
/// - Health check برای Kubernetes/Docker
/// - بررسی اتصال دیتابیس
///
/// # Endpoint
/// `GET /health`
///
/// # Response
/// ```json
/// {
///   "status": "healthy",
///   "version": "0.1.0",
///   "database": true
/// }
/// ```
pub async fn health_check(
    State(_state): State<AppState>,
) -> Result<Json<HealthResponse>> {
    // TODO: بررسی واقعی دیتابیس
    // let db_ok = state.database.health_check().await.is_ok();
    let db_ok = true;
    
    Ok(Json(HealthResponse::healthy(db_ok)))
}

