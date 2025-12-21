//! # Data Transfer Objects (DTOs)
//!
//! DTO‌های عمومی که در API استفاده میشن
//!
//! ## مفاهیم:
//! - DTO: برای انتقال داده بین لایه‌ها
//! - Request/Response separation: جداسازی ورودی از خروجی

use serde::{Deserialize, Serialize};

// =====================================
// Generic API Responses
// =====================================
/// پاسخ موفق عمومی
///
/// # مفاهیم:
/// - Generic: کار با هر نوع داده
/// - `T: Serialize`: T باید قابل تبدیل به JSON باشه
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    /// ساخت پاسخ موفق
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            message: None,
        }
    }
    
    /// اضافه کردن پیام
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

/// پاسخ خالی برای عملیات‌هایی که داده برنمیگردونن
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyResponse {
    pub success: bool,
    pub message: String,
}

impl EmptyResponse {
    /// ساخت پاسخ خالی موفق
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
        }
    }
}

// =====================================
// Health Check
// =====================================
/// پاسخ health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub database: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_seconds: Option<u64>,
}

impl HealthResponse {
    /// ساخت پاسخ healthy
    #[must_use]
    pub fn healthy(database_ok: bool) -> Self {
        Self {
            status: if database_ok { "healthy" } else { "degraded" }.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database: database_ok,
            uptime_seconds: None,
        }
    }
}

// =====================================
// Statistics
// =====================================
/// آمار کلی سیستم
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub total_urls: i64,
    pub total_clicks: i64,
    pub total_users: i64,
    pub average_clicks_per_url: f64,
}

// =====================================
// Search & Filter
// =====================================
/// پارامترهای جستجو
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SearchParams {
    /// عبارت جستجو
    #[serde(default)]
    pub query: Option<String>,
    
    /// فیلتر بر اساس وضعیت
    #[serde(default)]
    pub status: Option<String>,
    
    /// مرتب‌سازی بر اساس
    #[serde(default)]
    pub sort_by: Option<String>,
    
    /// ترتیب مرتب‌سازی
    #[serde(default)]
    pub order: Option<String>,
}

// =====================================
// Batch Operations
// =====================================
/// درخواست حذف دسته‌ای
#[derive(Debug, Clone, Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<String>,
}

/// پاسخ عملیات دسته‌ای
#[derive(Debug, Clone, Serialize)]
pub struct BatchOperationResponse {
    pub success_count: usize,
    pub failed_count: usize,
    
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub failed_ids: Vec<String>,
}

impl BatchOperationResponse {
    /// ساخت پاسخ جدید
    pub fn new(success_count: usize, failed_ids: Vec<String>) -> Self {
        Self {
            success_count,
            failed_count: failed_ids.len(),
            failed_ids,
        }
    }
}

