//! # سرویس URL
//!
//! منطق کسب‌وکار مربوط به URL‌ها
//!
//! ## مفاهیم Rust:
//! - Business Logic: قوانین برنامه اینجا پیاده‌سازی میشن
//! - Separation of Concerns: جداسازی از لایه داده
//! - Error Handling: مدیریت خطا در سطح business

use std::sync::Arc;
use chrono::Utc;
use tracing::{info, warn, instrument};
use validator::Validate;

use crate::{
    config::Config,
    database::UrlRepository,
    error::{AppError, Result, OptionExt},
    models::{
        CreateUrl, CreateUrlRequest, Url, UrlBuilder, UrlResponse,
    },
    utils,
};

use super::Service;

// =====================================
// URL Service
// =====================================
/// سرویس مدیریت URL‌ها
///
/// # مسئولیت‌ها:
/// - ساخت URL کوتاه
/// - Redirect و افزایش counter
/// - اعتبارسنجی
/// - مدیریت انقضا
#[derive(Debug, Clone)]
pub struct UrlService {
    repo: UrlRepository,
    config: Arc<Config>,
}

// پیاده‌سازی marker trait
impl Service for UrlService {}

impl UrlService {
    /// ساخت سرویس جدید
    #[must_use]
    pub fn new(repo: UrlRepository, config: Arc<Config>) -> Self {
        Self { repo, config }
    }
    
    /// ساخت URL کوتاه جدید
    ///
    /// # مفاهیم:
    /// - `#[instrument]`: macro برای tracing خودکار
    /// - `skip(self)`: از لاگ کردن self صرفنظر کن
    /// - Validation در لایه service
    /// - Transaction handling
    ///
    /// # Arguments
    /// * `request` - درخواست ساخت URL
    /// * `user_id` - شناسه کاربر (اختیاری)
    ///
    /// # Errors
    /// - `BadRequest`: URL نامعتبر
    /// - `Conflict`: کد کوتاه تکراری
    #[instrument(skip(self), fields(url = %request.url))]
    pub async fn create_short_url(
        &self,
        request: CreateUrlRequest,
        user_id: Option<String>,
    ) -> Result<UrlResponse> {
        // Step 1: اعتبارسنجی
        // `?` خطا رو به بالا منتقل میکنه
        request.validate()?;
        
        // Step 2: بررسی URL اصلی
        if !utils::is_valid_url(&request.url) {
            return Err(AppError::BadRequest("Invalid URL format".to_string()));
        }
        
        // Step 3: تولید یا اعتبارسنجی کد کوتاه
        let short_code = match &request.custom_code {
            Some(code) => {
                // اعتبارسنجی کد سفارشی
                if !utils::is_valid_short_code(code) {
                    return Err(AppError::BadRequest(
                        "Invalid custom code format".to_string()
                    ));
                }
                
                // بررسی تکراری نبودن
                if self.repo.exists(code).await? {
                    return Err(AppError::Conflict(
                        format!("Short code '{}' already exists", code)
                    ));
                }
                
                code.clone()
            }
            None => {
                // تولید کد یکتا
                self.generate_unique_code().await?
            }
        };
        
        // Step 4: ساخت URL با Builder Pattern
        let mut builder = UrlBuilder::new(&request.url)
            .custom_code(&short_code);
        
        if let Some(title) = request.title {
            builder = builder.title(title);
        }
        
        if let Some(user) = user_id {
            builder = builder.user_id(user);
        }
        
        if let Some(hours) = request.expires_in_hours {
            builder = builder.expires_in_hours(hours);
        }
        
        let create_url = builder.build()?;
        
        // Step 5: ذخیره در دیتابیس
        let url = self.repo.create(&create_url).await?;
        
        info!(short_code = %url.short_code, "Created new short URL");
        
        // Step 6: تبدیل به response
        Ok(UrlResponse::from_url(&url, &self.config.base_url))
    }
    
    /// گرفتن URL اصلی برای redirect
    ///
    /// # مفاهیم:
    /// - Side effect: افزایش counter
    /// - Expiration check
    #[instrument(skip(self))]
    pub async fn get_original_url(&self, short_code: &str) -> Result<String> {
        // پیدا کردن URL
        let url = self.repo
            .find_by_short_code(short_code)
            .await?
            .ok_or_not_found(format!("URL '{}' not found", short_code))?;
        
        // بررسی انقضا
        if url.is_expired() {
            warn!(short_code = %short_code, "Attempted to access expired URL");
            return Err(AppError::NotFound(
                "This URL has expired".to_string()
            ));
        }
        
        // افزایش counter (در پس‌زمینه انجام میشه)
        // Clone کردن برای انتقال به task
        let repo = self.repo.clone();
        let code = short_code.to_string();
        
        // Spawn یک task برای افزایش counter
        // این باعث میشه redirect سریع‌تر باشه
        tokio::spawn(async move {
            if let Err(e) = repo.increment_clicks(&code).await {
                warn!(error = %e, "Failed to increment click count");
            }
        });
        
        Ok(url.original_url)
    }
    
    /// گرفتن اطلاعات کامل URL
    #[instrument(skip(self))]
    pub async fn get_url_info(&self, short_code: &str) -> Result<UrlResponse> {
        let url = self.repo
            .find_by_short_code(short_code)
            .await?
            .ok_or_not_found(format!("URL '{}' not found", short_code))?;
        
        Ok(UrlResponse::from_url(&url, &self.config.base_url))
    }
    
    /// لیست URL‌های یک کاربر
    pub async fn get_user_urls(&self, user_id: &str) -> Result<Vec<UrlResponse>> {
        let urls = self.repo.find_by_user(user_id).await?;
        
        let responses: Vec<UrlResponse> = urls
            .iter()
            .map(|url| UrlResponse::from_url(url, &self.config.base_url))
            .collect();
        
        Ok(responses)
    }
    
    /// حذف URL
    ///
    /// # Arguments
    /// * `short_code` - کد کوتاه
    /// * `user_id` - شناسه کاربر (برای authorization)
    #[instrument(skip(self))]
    pub async fn delete_url(
        &self,
        short_code: &str,
        user_id: Option<&str>,
    ) -> Result<()> {
        // پیدا کردن URL
        let url = self.repo
            .find_by_short_code(short_code)
            .await?
            .ok_or_not_found(format!("URL '{}' not found", short_code))?;
        
        // بررسی مالکیت
        if let Some(uid) = user_id {
            if url.user_id.as_deref() != Some(uid) {
                return Err(AppError::Forbidden(
                    "You don't have permission to delete this URL".to_string()
                ));
            }
        }
        
        // حذف
        self.repo.delete(&url.id).await?;
        
        info!(short_code = %short_code, "Deleted URL");
        Ok(())
    }
    
    /// تولید کد یکتا
    ///
    /// # مفاهیم:
    /// - Loop با retry
    /// - تضمین یکتا بودن
    async fn generate_unique_code(&self) -> Result<String> {
        // حداکثر 10 بار تلاش
        for _ in 0..10 {
            let code = utils::generate_short_code();
            
            if !self.repo.exists(&code).await? {
                return Ok(code);
            }
        }
        
        // اگه بعد از 10 بار هنوز تکراری بود
        Err(AppError::Internal(
            "Failed to generate unique short code".to_string()
        ))
    }
    
    /// پاکسازی URL‌های منقضی
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let deleted = self.repo.delete_expired().await?;
        
        if deleted > 0 {
            info!(count = deleted, "Cleaned up expired URLs");
        }
        
        Ok(deleted)
    }
    
    /// گرفتن آمار
    pub async fn get_stats(&self) -> Result<crate::database::UrlStats> {
        self.repo.get_stats().await
    }
}

// =====================================
// Tests
// =====================================
#[cfg(test)]
mod tests {
    use super::*;
    
    // تست‌های unit برای توابع pure
    // تست‌های integration با database mock
    
    #[test]
    fn test_url_validation() {
        assert!(utils::is_valid_url("https://example.com"));
        assert!(!utils::is_valid_url("not-a-url"));
    }
    
    #[test]
    fn test_short_code_generation() {
        let code = utils::generate_short_code();
        assert!(utils::is_valid_short_code(&code));
    }
}

