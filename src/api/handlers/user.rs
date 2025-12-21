//! # User Handlers
//!
//! Handler‌های مربوط به کاربر

use axum::{
    extract::State,
    Json,
};

use crate::{
    error::Result,
    models::{ApiResponse, UrlResponse, UserResponse},
    services::AppState,
    api::extractors::AuthUser,
};

// =====================================
// Get Profile
// =====================================
/// گرفتن پروفایل کاربر فعلی
///
/// # مفاهیم:
/// - `AuthUser`: استخراج کاربر احراز هویت شده
/// - اگه توکن نداشته باشه، 401 برمیگرده
///
/// # Endpoint
/// `GET /api/me`
///
/// # Headers
/// `Authorization: Bearer <token>`
pub async fn get_profile(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<ApiResponse<UserResponse>>> {
    let user = state.auth_service.get_user(&user_id).await?;
    
    Ok(Json(ApiResponse::success(user)))
}

// =====================================
// Get My URLs
// =====================================
/// گرفتن URL‌های کاربر فعلی
///
/// # Endpoint
/// `GET /api/me/urls`
///
/// # Headers
/// `Authorization: Bearer <token>`
pub async fn get_my_urls(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<ApiResponse<Vec<UrlResponse>>>> {
    let urls = state.url_service.get_user_urls(&user_id).await?;
    
    Ok(Json(ApiResponse::success(urls)))
}

