//! # Custom Extractors
//!
//! Extractor‌های سفارشی برای استخراج داده از request
//!
//! ## مفاهیم Rust + Axum:
//! - **Extractors**: نوع‌هایی که از request داده استخراج میکنن
//! - **FromRequestParts**: trait برای ساخت extractor
//! - **async_trait**: پشتیبانی از async در traits
//! - **Rejection**: نوع خطا برای extractors
//!
//! ## چطور کار میکنه؟
//! وقتی یه extractor به عنوان پارامتر handler تعریف میشه،
//! axum قبل از اجرای handler، extractor رو اجرا میکنه.

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header, request::Parts, HeaderMap},
};

use crate::{
    error::AppError,
    services::AppState,
};

// =====================================
// Bearer Token Extractor
// =====================================
/// استخراج توکن از header Authorization
///
/// # مفاهیم:
/// - Newtype pattern: wrapper دور String
/// - `FromRequestParts`: trait برای extractors
/// - Header parsing
///
/// # استفاده در handler:
/// ```rust,ignore
/// async fn handler(BearerToken(token): BearerToken) -> ... {
///     // token حالا یه String هست
/// }
/// ```
#[derive(Debug, Clone)]
pub struct BearerToken(pub String);

#[async_trait]
impl FromRequestParts<AppState> for BearerToken {
    type Rejection = AppError;
    
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // گرفتن header Authorization
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                AppError::Unauthorized("Missing Authorization header".to_string())
            })?;
        
        // چک کردن prefix و استخراج توکن
        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized(
                "Invalid Authorization header format".to_string()
            ));
        }
        
        let token = auth_header[7..].to_string();
        
        if token.is_empty() {
            return Err(AppError::Unauthorized(
                "Empty token".to_string()
            ));
        }
        
        Ok(BearerToken(token))
    }
}

// =====================================
// Auth User Extractor
// =====================================
/// استخراج کاربر احراز هویت شده
///
/// # مفاهیم:
/// - این extractor توکن رو verify میکنه
/// - user_id رو برمیگردونه
/// - اگه توکن نامعتبر باشه، 401 برمیگرده
///
/// # استفاده:
/// ```rust,ignore
/// async fn handler(AuthUser(user_id): AuthUser) -> ... {
///     // user_id شناسه کاربر هست
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthUser(pub String);

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;
    
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // اول توکن رو بگیر
        let BearerToken(token) = BearerToken::from_request_parts(parts, state).await?;
        
        // Verify کردن توکن
        let claims = state.auth_service.verify_token(&token)?;
        
        Ok(AuthUser(claims.sub))
    }
}

// =====================================
// Optional Auth Extractor
// =====================================
/// استخراج کاربر اختیاری
///
/// # مفاهیم:
/// - Option<T>: اگه توکن نداشته باشه، None برمیگرده
/// - این برای endpoint‌هایی که هم با و هم بدون لاگین کار میکنن
///
/// # استفاده:
/// ```rust,ignore
/// async fn handler(auth: OptionalAuth) -> ... {
///     if let Some(user_id) = auth.user_id() {
///         // کاربر لاگین کرده
///     }
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct OptionalAuth {
    user_id: Option<String>,
}

impl OptionalAuth {
    /// گرفتن user_id
    #[must_use]
    pub fn user_id(&self) -> Option<String> {
        self.user_id.clone()
    }
    
    /// آیا لاگین کرده؟
    #[must_use]
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }
}

#[async_trait]
impl FromRequestParts<AppState> for OptionalAuth {
    type Rejection = AppError;  // هیچوقت reject نمیکنه
    
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // تلاش برای گرفتن کاربر
        // اگه fail شد، None برگردون
        let user_id = match AuthUser::from_request_parts(parts, state).await {
            Ok(AuthUser(id)) => Some(id),
            Err(_) => None,
        };
        
        Ok(OptionalAuth { user_id })
    }
}

// =====================================
// Request ID Extractor
// =====================================
/// استخراج یا تولید Request ID
///
/// # مفاهیم:
/// - برای tracing و لاگینگ
/// - میتونه از header بخونه یا جدید بسازه
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl RequestId {
    /// Header name برای request ID
    pub const HEADER_NAME: &'static str = "X-Request-Id";
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for RequestId {
    type Rejection = std::convert::Infallible;  // هیچوقت fail نمیکنه
    
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // اول چک کن header هست یا نه
        let request_id = parts
            .headers
            .get(Self::HEADER_NAME)
            .and_then(|v| v.to_str().ok())
            .map(ToString::to_string)
            .unwrap_or_else(|| nanoid::nanoid!(12));
        
        Ok(RequestId(request_id))
    }
}

// =====================================
// Client IP Extractor
// =====================================
/// استخراج IP کلاینت
///
/// # مفاهیم:
/// - بررسی header‌های proxy
/// - X-Forwarded-For, X-Real-IP
#[derive(Debug, Clone)]
pub struct ClientIp(pub Option<String>);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for ClientIp {
    type Rejection = std::convert::Infallible;
    
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // اول X-Forwarded-For رو چک کن (برای پشت proxy)
        let ip = parts
            .headers
            .get("X-Forwarded-For")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim().to_string())
            // بعد X-Real-IP
            .or_else(|| {
                parts
                    .headers
                    .get("X-Real-IP")
                    .and_then(|v| v.to_str().ok())
                    .map(ToString::to_string)
            });
        
        Ok(ClientIp(ip))
    }
}

// =====================================
// User Agent Extractor
// =====================================
/// استخراج User-Agent
#[derive(Debug, Clone)]
pub struct UserAgent(pub Option<String>);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for UserAgent {
    type Rejection = std::convert::Infallible;
    
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let ua = parts
            .headers
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .map(ToString::to_string);
        
        Ok(UserAgent(ua))
    }
}

// =====================================
// JSON with Validation
// =====================================
use axum::extract::rejection::JsonRejection;
use serde::de::DeserializeOwned;
use validator::Validate;

/// استخراج JSON با اعتبارسنجی خودکار
///
/// # مفاهیم:
/// - Validation در سطح extractor
/// - ترکیب چند کار در یک extractor
///
/// # استفاده:
/// ```rust,ignore
/// async fn handler(ValidatedJson(data): ValidatedJson<CreateUrlRequest>) -> ... {
///     // data حتما valid هست
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = AppError;
    
    async fn from_request_parts(
        _parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // این باید با FromRequest پیاده‌سازی بشه، نه FromRequestParts
        // چون body داره
        // این فقط یه placeholder هست
        Err(AppError::Internal("Use axum::Json instead".to_string()))
    }
}

// نسخه صحیح با FromRequest
use axum::{
    extract::FromRequest,
    body::Body,
    http::Request,
    Json,
};

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = AppError;
    
    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        // اول JSON رو parse کن
        let Json(data): Json<T> = Json::from_request(req, state)
            .await
            .map_err(|e: JsonRejection| {
                AppError::BadRequest(format!("Invalid JSON: {}", e))
            })?;
        
        // بعد validate کن
        data.validate()?;
        
        Ok(ValidatedJson(data))
    }
}

