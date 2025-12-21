//! # URL Handlers
//!
//! Handler‌های مربوط به URL shortening

use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use tracing::info;

use crate::{
    error::{AppError, Result},
    models::{CreateUrlRequest, UrlResponse, ApiResponse},
    services::AppState,
    api::extractors::OptionalAuth,
};

// =====================================
// Create Short URL
// =====================================
/// ساخت URL کوتاه جدید
///
/// # مفاهیم:
/// - `State<AppState>`: استخراج state از request
/// - `Json<T>`: استخراج و deserialize بدنه JSON
/// - `OptionalAuth`: کاربر اختیاری
/// - `impl IntoResponse`: هر نوعی که به response تبدیل بشه
///
/// # Endpoint
/// `POST /api/urls`
///
/// # Request Body
/// ```json
/// {
///   "url": "https://example.com/long-url",
///   "custom_code": "mylink",  // optional
///   "title": "My Link",        // optional
///   "expires_in_hours": 24     // optional
/// }
/// ```
///
/// # Response
/// ```json
/// {
///   "success": true,
///   "data": {
///     "id": "abc123...",
///     "short_code": "mylink",
///     "short_url": "http://localhost:3000/mylink",
///     "original_url": "https://example.com/long-url",
///     ...
///   }
/// }
/// ```
pub async fn create_url(
    State(state): State<AppState>,
    auth: OptionalAuth,
    Json(request): Json<CreateUrlRequest>,
) -> Result<impl IntoResponse> {
    // گرفتن user_id اگه لاگین باشه
    let user_id = auth.user_id();
    
    // فراخوانی سرویس
    let url = state.url_service.create_short_url(request, user_id).await?;
    
    // برگردوندن response با status 201 Created
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(url))
    ))
}

// =====================================
// Redirect
// =====================================
/// Redirect به URL اصلی
///
/// # مفاهیم:
/// - `Path<String>`: استخراج پارامتر از URL
/// - `Redirect`: نوع خاص axum برای redirect
/// - این handler اصلی‌ترین عملکرد URL shortener هست
///
/// # Endpoint
/// `GET /:code`
///
/// # Response
/// - 302 Redirect به URL اصلی
/// - 404 اگه پیدا نشه
pub async fn redirect_handler(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Response> {
    // گرفتن URL اصلی
    let original_url = state.url_service.get_original_url(&code).await?;
    
    info!(short_code = %code, "Redirecting");
    
    // ساخت redirect response
    // 302 Found برای temporary redirect
    // میتونید 301 Permanent هم استفاده کنید
    Ok(Redirect::temporary(&original_url).into_response())
}

// =====================================
// Get URL Info
// =====================================
/// گرفتن اطلاعات URL
///
/// # Endpoint
/// `GET /api/urls/:code`
///
/// # Response
/// ```json
/// {
///   "success": true,
///   "data": {
///     "short_code": "abc123",
///     "original_url": "https://...",
///     "clicks": 42,
///     ...
///   }
/// }
/// ```
pub async fn get_url_info(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<ApiResponse<UrlResponse>>> {
    let url = state.url_service.get_url_info(&code).await?;
    
    Ok(Json(ApiResponse::success(url)))
}

// =====================================
// Delete URL
// =====================================
/// حذف URL
///
/// # مفاهیم:
/// - Authorization: فقط مالک میتونه حذف کنه
/// - 204 No Content: پاسخ بدون بدنه
///
/// # Endpoint
/// `DELETE /api/urls/:code`
pub async fn delete_url(
    State(state): State<AppState>,
    auth: OptionalAuth,
    Path(code): Path<String>,
) -> Result<impl IntoResponse> {
    let user_id = auth.user_id();
    
    state.url_service.delete_url(&code, user_id.as_deref()).await?;
    
    // 204 No Content
    Ok(StatusCode::NO_CONTENT)
}

