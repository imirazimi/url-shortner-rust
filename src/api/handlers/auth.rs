//! # Auth Handlers
//!
//! Handler‌های مربوط به احراز هویت

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    error::Result,
    models::{
        LoginRequest, LoginResponse, RegisterRequest, RegisterResponse,
        ApiResponse,
    },
    services::AppState,
    api::extractors::BearerToken,
};

// =====================================
// Register
// =====================================
/// ثبت‌نام کاربر جدید
///
/// # مفاهیم:
/// - Validation در سرویس انجام میشه
/// - Password hashing اتوماتیک
///
/// # Endpoint
/// `POST /api/auth/register`
///
/// # Request Body
/// ```json
/// {
///   "email": "user@example.com",
///   "password": "securepassword",
///   "name": "John Doe"  // optional
/// }
/// ```
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<impl IntoResponse> {
    let response = state.auth_service.register(request).await?;
    
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(response))
    ))
}

// =====================================
// Login
// =====================================
/// ورود کاربر
///
/// # Endpoint
/// `POST /api/auth/login`
///
/// # Request Body
/// ```json
/// {
///   "email": "user@example.com",
///   "password": "securepassword"
/// }
/// ```
///
/// # Response
/// ```json
/// {
///   "success": true,
///   "data": {
///     "user": { ... },
///     "token": "eyJ...",
///     "expires_at": "2024-..."
///   }
/// }
/// ```
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>> {
    let response = state.auth_service.login(request).await?;
    
    Ok(Json(ApiResponse::success(response)))
}

// =====================================
// Refresh Token
// =====================================
/// Refresh کردن توکن
///
/// # مفاهیم:
/// - `BearerToken`: استخراج توکن از header
/// - این endpoint توکن جدید صادر میکنه
///
/// # Endpoint
/// `POST /api/auth/refresh`
///
/// # Headers
/// `Authorization: Bearer <token>`
pub async fn refresh_token(
    State(state): State<AppState>,
    BearerToken(token): BearerToken,
) -> Result<Json<ApiResponse<LoginResponse>>> {
    let response = state.auth_service.refresh_token(&token).await?;
    
    Ok(Json(ApiResponse::success(response)))
}

