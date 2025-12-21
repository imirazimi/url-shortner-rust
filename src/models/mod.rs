//! # ماژول مدل‌ها (Domain Models)
//!
//! این ماژول مدل‌های داده برنامه رو تعریف میکنه.
//!
//! ## مفاهیم Rust:
//! - **Structs**: ساختار داده
//! - **Enums**: نوع‌های شمارشی
//! - **Derive Macros**: تولید خودکار کد
//! - **FromRow**: تبدیل از ردیف دیتابیس
//! - **Serialize/Deserialize**: تبدیل JSON
//! - **Validation**: اعتبارسنجی داده
//! - **Newtype Pattern**: کپسوله کردن نوع‌ها
//!
//! ## تفاوت انواع مدل:
//! - **Entity**: داده‌ای که در دیتابیس ذخیره میشه
//! - **DTO (Data Transfer Object)**: برای ارسال/دریافت از API
//! - **Domain Model**: منطق کسب‌وکار

mod url;
mod user;
mod dto;

// Re-export همه مدل‌ها
pub use url::*;
pub use user::*;
pub use dto::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// =====================================
// Common Types (Newtype Pattern)
// =====================================
/// شناسه یکتا
///
/// # Newtype Pattern
/// این الگو یه نوع ساده رو wrap میکنه برای:
/// - Type safety: جلوگیری از اشتباه
/// - اضافه کردن متد
/// - پنهان کردن جزئیات
///
/// # مثال
/// ```rust
/// use url_shortener::models::Id;
///
/// let id = Id::new();
/// println!("ID: {}", id.as_str());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]  // در JSON فقط مقدار داخلی نمایش داده میشه
pub struct Id(String);

impl Id {
    /// ساخت ID جدید
    #[must_use]
    pub fn new() -> Self {
        // استفاده از nanoid برای ID کوتاه و یکتا
        Self(nanoid::nanoid!(21))
    }
    
    /// ساخت از string موجود
    #[must_use]
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    
    /// گرفتن به عنوان &str
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// تبدیل به String
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

// تبدیل از String
impl From<String> for Id {
    fn from(s: String) -> Self {
        Self(s)
    }
}

// تبدیل از &str
impl From<&str> for Id {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

// نمایش به عنوان String
impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// برای استفاده در SQL queries
impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// =====================================
// Timestamps
// =====================================
/// ساختار برای timestamps
///
/// # مفاهیم:
/// - Composition: ترکیب کردن داده‌های مرتبط
/// - این ساختار در entity‌ها استفاده میشه
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamps {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Timestamps {
    /// ساخت timestamps جدید
    #[must_use]
    pub fn now() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
        }
    }
    
    /// بروزرسانی updated_at
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl Default for Timestamps {
    fn default() -> Self {
        Self::now()
    }
}

// =====================================
// Pagination
// =====================================
/// پارامترهای صفحه‌بندی
///
/// # مفاهیم:
/// - Default: مقادیر پیش‌فرض
/// - Validation: اعتبارسنجی
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// شماره صفحه (از 1 شروع میشه)
    #[serde(default = "default_page")]
    pub page: u32,
    
    /// تعداد آیتم در صفحه
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 20 }

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
        }
    }
}

impl Pagination {
    /// محاسبه offset برای SQL
    #[must_use]
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.per_page
    }
    
    /// محاسبه limit
    #[must_use]
    pub fn limit(&self) -> u32 {
        self.per_page.min(100) // حداکثر 100
    }
}

/// نتیجه صفحه‌بندی شده
///
/// # مفاهیم:
/// - Generic: کار با هر نوع داده
/// - Serialization: تبدیل به JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    /// داده‌ها
    pub data: Vec<T>,
    
    /// اطلاعات صفحه‌بندی
    pub pagination: PaginationInfo,
}

/// اطلاعات صفحه‌بندی
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub current_page: u32,
    pub per_page: u32,
    pub total_items: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationInfo {
    /// ساخت اطلاعات صفحه‌بندی
    #[must_use]
    pub fn new(pagination: &Pagination, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (pagination.per_page as f64)).ceil() as u32;
        
        Self {
            current_page: pagination.page,
            per_page: pagination.per_page,
            total_items,
            total_pages,
            has_next: pagination.page < total_pages,
            has_prev: pagination.page > 1,
        }
    }
}

impl<T> PaginatedResult<T> {
    /// ساخت نتیجه صفحه‌بندی شده
    pub fn new(data: Vec<T>, pagination: &Pagination, total_items: u64) -> Self {
        Self {
            data,
            pagination: PaginationInfo::new(pagination, total_items),
        }
    }
}

// =====================================
// Sort Order
// =====================================
/// ترتیب مرتب‌سازی
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

impl SortOrder {
    /// تبدیل به SQL
    #[must_use]
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

