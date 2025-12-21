//! # لایه API
//!
//! این ماژول HTTP handlers و routing رو مدیریت میکنه.
//!
//! ## مفاهیم Rust + Axum:
//! - **Router**: تعریف مسیرها
//! - **Handler Functions**: پردازش request‌ها
//! - **Extractors**: استخراج داده از request
//! - **State**: اشتراک state بین handlers
//! - **Middleware**: پردازش قبل/بعد از handler
//! - **Tower**: زیرساخت middleware
//!
//! ## ساختار URL‌ها:
//! - `POST /api/urls` - ساخت URL کوتاه
//! - `GET /:code` - Redirect به URL اصلی
//! - `GET /api/urls/:code` - اطلاعات URL
//! - `DELETE /api/urls/:code` - حذف URL
//! - `POST /api/auth/register` - ثبت‌نام
//! - `POST /api/auth/login` - ورود
//! - `GET /api/me` - پروفایل کاربر
//! - `GET /health` - Health check

mod handlers;
mod middleware;
mod extractors;

pub use handlers::*;
pub use middleware::*;
pub use extractors::*;

use axum::{
    routing::{get, post, delete},
    Router,
    middleware as axum_middleware,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
    timeout::TimeoutLayer,
    compression::CompressionLayer,
};
use std::time::Duration;

use crate::{
    config::Config,
    database::Database,
    services::AppState,
};

// =====================================
// Router Builder
// =====================================
/// ساخت Router اصلی برنامه
///
/// # مفاهیم:
/// - `Router::new()`: شروع router خالی
/// - `.route()`: اضافه کردن route
/// - `.nest()`: گروه‌بندی route‌ها
/// - `.layer()`: اضافه کردن middleware
/// - `.with_state()`: تزریق state
///
/// # Arguments
/// * `db` - اتصال دیتابیس
/// * `config` - تنظیمات برنامه
pub fn create_router(db: Database, config: Config) -> Router {
    // ساخت AppState
    let state = AppState::new(db, config.clone());
    
    // ساخت router با گروه‌بندی
    Router::new()
        // Route اصلی redirect
        .route("/:code", get(handlers::url::redirect_handler))
        
        // API routes
        .nest("/api", api_routes())
        
        // Health check
        .route("/health", get(handlers::health::health_check))
        
        // Middleware‌های عمومی
        .layer(
            ServiceBuilder::new()
                // Tracing - لاگ کردن request‌ها
                .layer(TraceLayer::new_for_http())
                
                // Timeout - حداکثر زمان پردازش
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                
                // Compression - فشرده‌سازی response
                .layer(CompressionLayer::new())
                
                // CORS - اجازه دسترسی از دامنه‌های دیگه
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any)
                )
        )
        
        // تزریق state به همه handlers
        .with_state(state)
}

/// Route‌های API
///
/// # مفاهیم:
/// - Nested routing: گروه‌بندی route‌ها
/// - RESTful design: طراحی REST
fn api_routes() -> Router<AppState> {
    Router::new()
        // URL endpoints
        .nest("/urls", url_routes())
        
        // Auth endpoints
        .nest("/auth", auth_routes())
        
        // User endpoints (نیاز به احراز هویت)
        .route("/me", get(handlers::user::get_profile))
        .route("/me/urls", get(handlers::user::get_my_urls))
        
        // Stats
        .route("/stats", get(handlers::stats::get_stats))
}

/// Route‌های URL
fn url_routes() -> Router<AppState> {
    Router::new()
        // ساخت URL کوتاه
        .route("/", post(handlers::url::create_url))
        
        // اطلاعات URL
        .route("/:code", get(handlers::url::get_url_info))
        
        // حذف URL
        .route("/:code", delete(handlers::url::delete_url))
}

/// Route‌های احراز هویت
fn auth_routes() -> Router<AppState> {
    Router::new()
        // ثبت‌نام
        .route("/register", post(handlers::auth::register))
        
        // ورود
        .route("/login", post(handlers::auth::login))
        
        // Refresh token
        .route("/refresh", post(handlers::auth::refresh_token))
}

// =====================================
// Request ID Middleware
// =====================================
/// تولید request ID برای tracing
#[must_use]
pub fn generate_request_id() -> String {
    nanoid::nanoid!(12)
}

