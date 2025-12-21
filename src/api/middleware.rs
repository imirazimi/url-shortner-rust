//! # Middleware
//!
//! Middleware‌های سفارشی برای پردازش request/response
//!
//! ## مفاهیم:
//! - **Middleware**: کد که قبل/بعد از handler اجرا میشه
//! - **Tower**: کتابخانه middleware در اکوسیستم Rust
//! - **Layer**: wrapper برای اضافه کردن middleware
//! - **Service**: trait اصلی Tower
//!
//! ## چرا Middleware؟
//! - Logging
//! - Authentication
//! - Rate Limiting
//! - CORS
//! - Error handling
//! - Request ID

use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use std::time::Instant;
use tracing::{info, warn, Span};

use crate::error::AppError;

// =====================================
// Request Timing Middleware
// =====================================
/// اندازه‌گیری زمان پردازش request
///
/// # مفاهیم:
/// - `async fn` middleware
/// - `Next`: ادامه زنجیره middleware
/// - `Instant`: اندازه‌گیری زمان
///
/// # استفاده:
/// ```rust,ignore
/// let app = Router::new()
///     .layer(axum::middleware::from_fn(request_timing));
/// ```
pub async fn request_timing(
    request: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    let method = request.method().clone();
    let uri = request.uri().clone();
    
    // شروع تایمر
    let start = Instant::now();
    
    // اجرای بقیه زنجیره
    let response = next.run(request).await;
    
    // محاسبه زمان
    let duration = start.elapsed();
    
    // لاگ کردن
    info!(
        method = %method,
        uri = %uri,
        status = %response.status(),
        duration_ms = %duration.as_millis(),
        "Request completed"
    );
    
    response
}

// =====================================
// Request ID Middleware
// =====================================
/// اضافه کردن Request ID به هر request
///
/// # مفاهیم:
/// - تولید ID یکتا برای هر request
/// - اضافه کردن به response header
/// - مفید برای debugging و tracing
pub async fn request_id(
    mut request: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    use axum::http::header::HeaderValue;
    
    // تولید یا استفاده از request ID موجود
    let request_id = request
        .headers()
        .get("X-Request-Id")
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string)
        .unwrap_or_else(|| nanoid::nanoid!(12));
    
    // اضافه کردن به request headers
    request.headers_mut().insert(
        "X-Request-Id",
        HeaderValue::from_str(&request_id).unwrap(),
    );
    
    // اجرای بقیه
    let mut response = next.run(request).await;
    
    // اضافه کردن به response
    response.headers_mut().insert(
        "X-Request-Id",
        HeaderValue::from_str(&request_id).unwrap(),
    );
    
    response
}

// =====================================
// Error Handler Middleware
// =====================================
/// مدیریت خطاهای unhandled
///
/// # مفاهیم:
/// - Catch-all برای خطاها
/// - تبدیل panic به response
pub async fn error_handler(
    request: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    let response = next.run(request).await;
    
    // اگه status نامعلوم بود، لاگ کن
    if response.status().is_server_error() {
        warn!(
            status = %response.status(),
            "Server error response"
        );
    }
    
    response
}

// =====================================
// Auth Middleware (Alternative)
// =====================================
use crate::services::AppState;
use axum::extract::State;

/// Middleware احراز هویت
///
/// این یه روش جایگزین برای Extractor هست.
/// معمولا Extractor بهتره، ولی این برای route‌های گروهی مفیده.
///
/// # استفاده:
/// ```rust,ignore
/// let protected = Router::new()
///     .route("/protected", get(handler))
///     .layer(axum::middleware::from_fn_with_state(state.clone(), require_auth));
/// ```
pub async fn require_auth(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    // گرفتن header
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing auth token".to_string()))?;
    
    // چک کردن prefix
    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized("Invalid auth format".to_string()));
    }
    
    let token = &auth_header[7..];
    
    // Verify token
    state.auth_service.verify_token(token)?;
    
    // ادامه
    Ok(next.run(request).await)
}

// =====================================
// Rate Limiting (Simple In-Memory)
// =====================================
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// State برای rate limiter ساده
///
/// # مفاهیم:
/// - `RwLock`: قفل خواندن/نوشتن async
/// - `Arc`: اشتراک امن بین threads
/// - `HashMap`: نگهداری counter برای هر IP
#[derive(Debug, Clone, Default)]
pub struct RateLimiterState {
    requests: Arc<RwLock<HashMap<String, (u32, Instant)>>>,
    max_requests: u32,
    window_seconds: u64,
}

impl RateLimiterState {
    /// ساخت rate limiter جدید
    #[must_use]
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_seconds,
        }
    }
    
    /// چک کردن rate limit
    ///
    /// # Returns
    /// - `Ok(())` اگه مجاز باشه
    /// - `Err(AppError)` اگه محدود شده باشه
    pub async fn check(&self, key: &str) -> Result<(), AppError> {
        let now = Instant::now();
        let mut requests = self.requests.write().await;
        
        // گرفتن یا ساختن entry
        let entry = requests.entry(key.to_string()).or_insert((0, now));
        
        // چک کردن window
        let window = std::time::Duration::from_secs(self.window_seconds);
        if now.duration_since(entry.1) > window {
            // Window جدید
            *entry = (1, now);
            return Ok(());
        }
        
        // چک کردن تعداد
        if entry.0 >= self.max_requests {
            return Err(AppError::RateLimited);
        }
        
        // افزایش counter
        entry.0 += 1;
        
        Ok(())
    }
    
    /// پاکسازی entry‌های قدیمی
    pub async fn cleanup(&self) {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_seconds);
        let mut requests = self.requests.write().await;
        
        requests.retain(|_, (_, time)| now.duration_since(*time) <= window);
    }
}

// =====================================
// Security Headers Middleware
// =====================================
/// اضافه کردن header‌های امنیتی
///
/// # Headers:
/// - X-Content-Type-Options
/// - X-Frame-Options
/// - X-XSS-Protection
/// - Strict-Transport-Security
pub async fn security_headers(
    request: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    use axum::http::header::HeaderValue;
    
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    
    // جلوگیری از MIME sniffing
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    
    // جلوگیری از clickjacking
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY"),
    );
    
    // محافظت XSS (برای مرورگرهای قدیمی)
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    
    // Referrer policy
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    
    response
}

