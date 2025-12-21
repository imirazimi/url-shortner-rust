//! # HTTP Handlers
//!
//! این ماژول handler‌های HTTP رو تعریف میکنه.
//!
//! ## Handler چیه؟
//! تابعی که request میگیره و response برمیگردونه.
//! در axum، هر handler یک async function هست.
//!
//! ## مفاهیم:
//! - Extractors: برای گرفتن داده از request
//! - State: برای دسترسی به سرویس‌ها
//! - IntoResponse: برای ساخت response

pub mod url;
pub mod auth;
pub mod user;
pub mod health;
pub mod stats;

