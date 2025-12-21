//! # مدل URL
//!
//! Entity و DTO‌های مربوط به URL

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

// =====================================
// URL Entity
// =====================================
/// Entity اصلی URL
///
/// # مفاهیم:
/// - `#[derive(FromRow)]`: تبدیل خودکار از ردیف دیتابیس
/// - `#[sqlx(rename_all = "snake_case")]`: نام‌گذاری ستون‌ها
/// - این struct مستقیم از دیتابیس خونده میشه
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Url {
    /// شناسه یکتا
    pub id: String,
    
    /// کد کوتاه (مثلا "abc123")
    pub short_code: String,
    
    /// آدرس اصلی
    pub original_url: String,
    
    /// عنوان (اختیاری)
    pub title: Option<String>,
    
    /// تعداد کلیک
    pub clicks: i64,
    
    /// شناسه کاربر مالک (اختیاری)
    pub user_id: Option<String>,
    
    /// تاریخ انقضا (اختیاری)
    pub expires_at: Option<DateTime<Utc>>,
    
    /// تاریخ ایجاد
    pub created_at: DateTime<Utc>,
    
    /// تاریخ آخرین بروزرسانی
    pub updated_at: DateTime<Utc>,
}

impl Url {
    /// آیا URL منقضی شده؟
    ///
    /// # مفاهیم:
    /// - Option handling با `map_or`
    /// - مقایسه DateTime
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map_or(false, |exp| exp < Utc::now())
    }
    
    /// گرفتن لینک کوتاه کامل
    #[must_use]
    pub fn short_url(&self, base_url: &str) -> String {
        format!("{}/{}", base_url.trim_end_matches('/'), self.short_code)
    }
}

// =====================================
// Create URL DTO
// =====================================
/// داده برای ساخت URL جدید (داخلی)
///
/// این DTO برای ارسال به Repository استفاده میشه
#[derive(Debug, Clone)]
pub struct CreateUrl {
    pub id: String,
    pub short_code: String,
    pub original_url: String,
    pub title: Option<String>,
    pub user_id: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

// =====================================
// API Request DTOs
// =====================================
/// درخواست ساخت URL کوتاه
///
/// # مفاهیم:
/// - `#[derive(Validate)]`: اعتبارسنجی خودکار
/// - `#[validate(...)]`: قوانین اعتبارسنجی
/// - این DTO از API دریافت میشه
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateUrlRequest {
    /// آدرس اصلی
    #[validate(url(message = "Invalid URL format"))]
    #[validate(length(max = 2048, message = "URL is too long"))]
    pub url: String,
    
    /// کد سفارشی (اختیاری)
    #[validate(length(min = 3, max = 20, message = "Custom code must be 3-20 characters"))]
    #[validate(regex(path = "crate::utils::VALID_SHORT_CODE", message = "Invalid custom code format"))]
    pub custom_code: Option<String>,
    
    /// عنوان (اختیاری)
    #[validate(length(max = 200, message = "Title is too long"))]
    pub title: Option<String>,
    
    /// مدت اعتبار به ساعت (اختیاری)
    pub expires_in_hours: Option<u32>,
}

/// درخواست بروزرسانی URL
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateUrlRequest {
    /// عنوان جدید
    #[validate(length(max = 200, message = "Title is too long"))]
    pub title: Option<String>,
    
    /// مدت اعتبار جدید
    pub expires_in_hours: Option<u32>,
}

// =====================================
// API Response DTOs
// =====================================
/// پاسخ URL
///
/// این DTO به کلاینت ارسال میشه
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlResponse {
    pub id: String,
    pub short_code: String,
    pub short_url: String,
    pub original_url: String,
    pub title: Option<String>,
    pub clicks: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl UrlResponse {
    /// تبدیل از Url entity
    ///
    /// # مفاهیم:
    /// - تبدیل دستی برای کنترل روی خروجی
    /// - base_url برای ساخت لینک کامل
    #[must_use]
    pub fn from_url(url: &Url, base_url: &str) -> Self {
        Self {
            id: url.id.clone(),
            short_code: url.short_code.clone(),
            short_url: url.short_url(base_url),
            original_url: url.original_url.clone(),
            title: url.title.clone(),
            clicks: url.clicks,
            expires_at: url.expires_at,
            created_at: url.created_at,
        }
    }
}

/// پاسخ redirect (فقط URL اصلی)
#[derive(Debug, Clone, Serialize)]
pub struct RedirectResponse {
    pub original_url: String,
}

// =====================================
// URL Builder (Builder Pattern)
// =====================================
/// Builder برای ساخت URL
///
/// # مفاهیم:
/// - Builder Pattern: ساخت تدریجی object
/// - Type State Pattern: اطمینان compile-time از صحت
/// - Method Chaining: زنجیره‌ای کردن متدها
///
/// # مثال
/// ```rust,ignore
/// let url = UrlBuilder::new("https://example.com")
///     .title("My Link")
///     .custom_code("mylink")
///     .build()?;
/// ```
#[derive(Debug, Default)]
pub struct UrlBuilder {
    original_url: Option<String>,
    short_code: Option<String>,
    title: Option<String>,
    user_id: Option<String>,
    expires_at: Option<DateTime<Utc>>,
}

impl UrlBuilder {
    /// شروع builder با URL اصلی
    #[must_use]
    pub fn new(original_url: impl Into<String>) -> Self {
        Self {
            original_url: Some(original_url.into()),
            ..Default::default()
        }
    }
    
    /// تنظیم کد کوتاه سفارشی
    #[must_use]
    pub fn custom_code(mut self, code: impl Into<String>) -> Self {
        self.short_code = Some(code.into());
        self
    }
    
    /// تنظیم عنوان
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    
    /// تنظیم مالک
    #[must_use]
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
    
    /// تنظیم تاریخ انقضا
    #[must_use]
    pub fn expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// تنظیم انقضا بعد از n ساعت
    #[must_use]
    pub fn expires_in_hours(mut self, hours: u32) -> Self {
        let expires = Utc::now() + chrono::Duration::hours(hours as i64);
        self.expires_at = Some(expires);
        self
    }
    
    /// ساخت CreateUrl
    ///
    /// # Errors
    /// خطا برمیگردونه اگه URL اصلی تنظیم نشده باشه
    pub fn build(self) -> crate::error::Result<CreateUrl> {
        let original_url = self.original_url
            .ok_or_else(|| crate::error::AppError::BadRequest(
                "Original URL is required".to_string()
            ))?;
        
        // اگه کد سفارشی نداریم، یکی تولید میکنیم
        let short_code = self.short_code
            .unwrap_or_else(crate::utils::generate_short_code);
        
        Ok(CreateUrl {
            id: nanoid::nanoid!(21),
            short_code,
            original_url,
            title: self.title,
            user_id: self.user_id,
            expires_at: self.expires_at,
        })
    }
}

