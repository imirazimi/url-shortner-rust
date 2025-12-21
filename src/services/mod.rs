//! # ماژول سرویس‌ها (Business Logic Layer)
//!
//! این ماژول منطق کسب‌وکار برنامه رو پیاده‌سازی میکنه.
//!
//! ## لایه‌بندی معماری
//!
//! ```text
//! ┌─────────────────┐
//! │    API Layer    │  <-- HTTP handlers (axum)
//! ├─────────────────┤
//! │  Service Layer  │  <-- Business logic (اینجا!)
//! ├─────────────────┤
//! │ Repository Layer│  <-- Data access
//! ├─────────────────┤
//! │    Database     │  <-- SQLite
//! └─────────────────┘
//! ```
//!
//! ## مفاهیم Rust:
//! - **Dependency Injection**: تزریق وابستگی‌ها
//! - **Traits برای Abstraction**: interface تعریف کردن
//! - **Arc<T>**: اشتراک امن بین threads
//! - **async/await**: عملیات غیرهمزمان

mod url_service;
mod auth_service;

pub use url_service::*;
pub use auth_service::*;

use std::sync::Arc;
use crate::{
    config::Config,
    database::{Database, UrlRepository, UserRepository},
};

// =====================================
// Application State
// =====================================
/// وضعیت برنامه که بین همه handlers اشتراک‌گذاری میشه
///
/// # مفاهیم:
/// - `Arc<T>`: Reference counting برای thread-safe sharing
/// - `Clone`: کپی کردن (فقط Arc clone میشه، نه داده)
/// - این state در axum به عنوان Extension تزریق میشه
///
/// ## چرا این ساختار؟
/// - هر request به یک handler میره
/// - handlers باید به services دسترسی داشته باشن
/// - Arc اجازه میده بدون کپی داده، reference share کنیم
#[derive(Clone)]
pub struct AppState {
    /// تنظیمات برنامه
    pub config: Arc<Config>,
    
    /// سرویس URL
    pub url_service: Arc<UrlService>,
    
    /// سرویس احراز هویت
    pub auth_service: Arc<AuthService>,
}

impl AppState {
    /// ساخت AppState جدید
    ///
    /// # مفاهیم:
    /// - Factory method: ساخت object پیچیده
    /// - Dependency Injection: همه وابستگی‌ها تزریق میشن
    #[must_use]
    pub fn new(db: Database, config: Config) -> Self {
        // ساخت repositories
        let url_repo = UrlRepository::new(db.clone());
        let user_repo = UserRepository::new(db);
        
        // ساخت config به صورت Arc
        let config = Arc::new(config);
        
        // ساخت services
        let url_service = Arc::new(UrlService::new(
            url_repo,
            config.clone(),
        ));
        
        let auth_service = Arc::new(AuthService::new(
            user_repo,
            config.clone(),
        ));
        
        Self {
            config,
            url_service,
            auth_service,
        }
    }
    
    /// دسترسی به config
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }
}

// =====================================
// Service Trait (اختیاری)
// =====================================
/// Trait پایه برای services
///
/// # مفاهیم:
/// - این یک marker trait هست
/// - همه services باید Send + Sync باشن برای thread-safety
/// - در پروژه‌های بزرگ‌تر میتونید متدهای مشترک اینجا بذارید
pub trait Service: Send + Sync {}

