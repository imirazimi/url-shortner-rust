//! # URL Shortener Library
//!
//! این کتابخانه یک سرویس کوتاه‌کننده URL کامل ارائه میده.
//!
//! ## ساختار پروژه
//!
//! ```text
//! src/
//! ├── lib.rs          # نقطه ورود کتابخانه - اینجا!
//! ├── main.rs         # نقطه ورود باینری
//! ├── config/         # مدیریت تنظیمات
//! ├── error/          # تعریف خطاها
//! ├── database/       # لایه دیتابیس
//! ├── models/         # مدل‌های داده
//! ├── services/       # منطق کسب‌وکار
//! ├── api/            # لایه API
//! └── utils/          # توابع کمکی
//! ```
//!
//! ## مفاهیم Rust در این فایل
//!
//! - **Module System**: سیستم ماژول‌ها برای سازماندهی کد
//! - **Public API**: با `pub` مشخص میکنیم چی از بیرون قابل دسترسی باشه
//! - **Re-exports**: با `pub use` آیتم‌ها رو re-export میکنیم
//!
//! ## مثال استفاده
//!
//! ```rust,no_run
//! use url_shortener::{config::Config, database::Database};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = Config::from_env().unwrap();
//!     let db = Database::connect(&config.database_url).await.unwrap();
//! }
//! ```

// =====================================
// Module Declarations
// =====================================
// در Rust، هر ماژول باید در lib.rs یا main.rs declare بشه
// `pub mod` یعنی این ماژول از بیرون کتابخانه قابل دسترسی هست

/// ماژول مدیریت تنظیمات برنامه
pub mod config;

/// ماژول تعریف و مدیریت خطاها
pub mod error;

/// ماژول ارتباط با دیتابیس
pub mod database;

/// ماژول مدل‌های داده (Domain Models)
pub mod models;

/// ماژول سرویس‌ها (Business Logic)
pub mod services;

/// ماژول API و HTTP Handlers
pub mod api;

/// ماژول توابع کمکی
pub mod utils;

// =====================================
// Re-exports
// =====================================
// Re-export کردن آیتم‌های پرکاربرد برای دسترسی راحت‌تر
// کاربر به جای `url_shortener::error::Result` میتونه بنویسه `url_shortener::Result`

/// نتیجه عملیات با خطای سفارشی ما
pub use error::Result;

/// خطای اصلی برنامه
pub use error::AppError;

// =====================================
// Prelude Module
// =====================================
/// ماژول prelude برای import راحت‌تر آیتم‌های پرکاربرد
///
/// کاربرد:
/// ```rust
/// use url_shortener::prelude::*;
/// ```
pub mod prelude {
    pub use crate::config::Config;
    pub use crate::database::Database;
    pub use crate::error::{AppError, Result};
    pub use crate::models::*;
    pub use crate::services::*;
}

