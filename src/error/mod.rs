//! # ماژول مدیریت خطاها (Error Handling)
//!
//! این ماژول سیستم مدیریت خطای برنامه رو تعریف میکنه.
//!
//! ## مفاهیم Rust:
//! - **Custom Error Types**: تعریف نوع خطای سفارشی
//! - **thiserror**: derive macro برای Error trait
//! - **Error Trait**: trait استاندارد خطا
//! - **From Trait**: تبدیل خودکار نوع‌ها
//! - **Result Type Alias**: alias برای ساده‌تر شدن کد
//! - **Error Propagation**: انتشار خطا با `?`
//!
//! ## اهمیت Error Handling در Rust
//!
//! Rust از exceptions استفاده نمیکنه! به جاش از `Result<T, E>` استفاده میکنه.
//! این باعث میشه:
//! - خطاها صریح باشن
//! - نتونید خطا رو نادیده بگیرید
//! - کد قابل پیش‌بینی‌تر بشه

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

// =====================================
// Result Type Alias
// =====================================
/// نوع Result سفارشی برنامه
///
/// # مفاهیم:
/// - Type Alias: نام مستعار برای یک نوع
/// - Generic با default: `T` پارامتر، `E` ثابت
///
/// به جای نوشتن `Result<User, AppError>` میتونیم بنویسیم `Result<User>`
pub type Result<T, E = AppError> = std::result::Result<T, E>;

// =====================================
// Custom Error Enum
// =====================================
/// خطای اصلی برنامه
///
/// # مفاهیم:
/// - `enum`: نوع شمارشی با انواع مختلف خطا
/// - `#[derive(Error)]`: از thiserror برای پیاده‌سازی Error trait
/// - `#[error("...")]`: پیام خطا برای هر نوع
/// - `#[from]`: تبدیل خودکار از نوع‌های دیگه
///
/// # چرا enum؟
/// هر variant نمایانگر یک نوع خطا با داده‌های متفاوت هست.
/// این الگو در Rust خیلی رایجه.
#[derive(Debug, Error)]
pub enum AppError {
    // ----------------------------------------
    // خطاهای کاربر (4xx)
    // ----------------------------------------
    
    /// درخواست نامعتبر - 400
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    /// احراز هویت نشده - 401
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    /// دسترسی ممنوع - 403
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    /// پیدا نشد - 404
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// تکراری - 409
    #[error("Conflict: {0}")]
    Conflict(String),
    
    /// محدودیت نرخ - 429
    #[error("Too many requests")]
    RateLimited,
    
    /// خطای اعتبارسنجی - 422
    #[error("Validation error: {0}")]
    Validation(String),
    
    // ----------------------------------------
    // خطاهای سرور (5xx)
    // ----------------------------------------
    
    /// خطای داخلی سرور - 500
    #[error("Internal server error: {0}")]
    Internal(String),
    
    /// خطای سرور
    #[error("Server error: {0}")]
    Server(String),
    
    /// خطای تنظیمات
    #[error("Configuration error: {0}")]
    Config(String),
    
    // ----------------------------------------
    // خطاهای تبدیل شده از کتابخانه‌ها
    // ----------------------------------------
    
    /// خطای دیتابیس
    /// `#[from]` یعنی sqlx::Error خودکار به این تبدیل میشه
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    /// خطای IO
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// خطای JSON
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// خطای JWT
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    
    /// خطای URL
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),
}

impl AppError {
    /// گرفتن HTTP status code متناسب با خطا
    ///
    /// # مفاهیم:
    /// - `match`: pattern matching
    /// - `&self`: reference به خودش
    /// - `Self::Variant`: مراجعه به variant‌ها
    #[must_use]
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 4xx Client Errors
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            
            // 5xx Server Errors
            Self::Internal(_) 
            | Self::Server(_) 
            | Self::Config(_)
            | Self::Database(_)
            | Self::Io(_)
            | Self::Json(_)
            | Self::Jwt(_)
            | Self::UrlParse(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    
    /// آیا این یه خطای سرور هست؟
    #[must_use]
    pub fn is_server_error(&self) -> bool {
        self.status_code().is_server_error()
    }
    
    /// ساخت خطای Not Found برای URL
    #[must_use]
    pub fn url_not_found(short_code: &str) -> Self {
        Self::NotFound(format!("URL with code '{}' not found", short_code))
    }
    
    /// ساخت خطای Not Found برای کاربر
    #[must_use]
    pub fn user_not_found(user_id: &str) -> Self {
        Self::NotFound(format!("User '{}' not found", user_id))
    }
}

// =====================================
// Error Response DTO
// =====================================
/// ساختار پاسخ خطا در API
///
/// # مفاهیم:
/// - DTO (Data Transfer Object): برای ارسال به کلاینت
/// - `Serialize`: تبدیل به JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// کد خطا (مثلا "NOT_FOUND")
    pub error: String,
    
    /// پیام خطا
    pub message: String,
    
    /// کد وضعیت HTTP
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    
    /// جزئیات اضافی (اختیاری)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    /// ساخت پاسخ خطای جدید
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            status_code: None,
            details: None,
        }
    }
    
    /// اضافه کردن کد وضعیت
    #[must_use]
    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status_code = Some(status.as_u16());
        self
    }
    
    /// اضافه کردن جزئیات
    #[must_use]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

// =====================================
// IntoResponse Implementation
// =====================================
/// تبدیل AppError به Response HTTP
///
/// # مفاهیم:
/// - `impl Trait for Type`: پیاده‌سازی trait
/// - `IntoResponse`: trait خاص axum برای تبدیل به response
/// - این باعث میشه بتونیم AppError رو مستقیم از handler برگردونیم
///
/// # مثال
/// ```rust,ignore
/// async fn handler() -> Result<Json<Data>, AppError> {
///     // اگه Err برگرده، خودکار به response تبدیل میشه
///     Ok(Json(data))
/// }
/// ```
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // لاگ کردن خطاهای سرور
        if self.is_server_error() {
            error!(error = %self, "Server error occurred");
        }
        
        let status = self.status_code();
        
        // ساخت پاسخ خطا
        // در production، جزئیات خطاهای داخلی رو مخفی میکنیم
        let error_response = ErrorResponse::new(
            status.canonical_reason().unwrap_or("Error"),
            self.to_string(),
        )
        .with_status(status);
        
        // برگردوندن tuple که axum بلده تبدیل کنه
        (status, Json(error_response)).into_response()
    }
}

// =====================================
// From Implementations
// =====================================
// این‌ها برای تبدیل خودکار خطاهای دیگه به AppError هستن
// با `?` میتونیم خطا رو propagate کنیم

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Internal(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Internal(s.to_string())
    }
}

// تبدیل validator error
impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        AppError::Validation(err.to_string())
    }
}

// =====================================
// Result Extensions
// =====================================
/// Extension trait برای Result
///
/// # مفاهیم:
/// - Extension Trait: اضافه کردن متد به نوع‌های موجود
/// - Generic: کار با هر نوع T و E
pub trait ResultExt<T, E> {
    /// تبدیل خطا به AppError::Internal
    fn map_internal(self) -> Result<T>;
    
    /// تبدیل خطا به نوع دلخواه
    fn map_app_err<F>(self, f: F) -> Result<T>
    where
        F: FnOnce(E) -> AppError;
}

impl<T, E: std::fmt::Display> ResultExt<T, E> for std::result::Result<T, E> {
    fn map_internal(self) -> Result<T> {
        self.map_err(|e| AppError::Internal(e.to_string()))
    }
    
    fn map_app_err<F>(self, f: F) -> Result<T>
    where
        F: FnOnce(E) -> AppError,
    {
        self.map_err(f)
    }
}

// =====================================
// Option Extensions
// =====================================
/// Extension trait برای Option
pub trait OptionExt<T> {
    /// تبدیل None به AppError::NotFound
    fn ok_or_not_found(self, message: impl Into<String>) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_not_found(self, message: impl Into<String>) -> Result<T> {
        self.ok_or_else(|| AppError::NotFound(message.into()))
    }
}

// =====================================
// Tests
// =====================================
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_status_codes() {
        assert_eq!(
            AppError::NotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        
        assert_eq!(
            AppError::BadRequest("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        
        assert_eq!(
            AppError::Internal("test".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
    
    #[test]
    fn test_error_response() {
        let response = ErrorResponse::new("NOT_FOUND", "Resource not found")
            .with_status(StatusCode::NOT_FOUND);
        
        assert_eq!(response.status_code, Some(404));
    }
    
    #[test]
    fn test_option_extension() {
        let some_value: Option<i32> = Some(42);
        let none_value: Option<i32> = None;
        
        assert!(some_value.ok_or_not_found("not found").is_ok());
        assert!(none_value.ok_or_not_found("not found").is_err());
    }
    
    #[test]
    fn test_result_extension() {
        let ok: std::result::Result<i32, &str> = Ok(42);
        let err: std::result::Result<i32, &str> = Err("original error");
        
        assert!(ok.map_internal().is_ok());
        let mapped = err.map_internal();
        assert!(matches!(mapped, Err(AppError::Internal(_))));
    }
}

