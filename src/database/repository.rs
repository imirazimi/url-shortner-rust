//! # Repository Pattern
//!
//! این فایل الگوی Repository رو پیاده‌سازی میکنه.
//!
//! ## Repository Pattern چیه؟
//! یه لایه انتزاعی بین منطق برنامه و دیتابیس.
//! - منطق برنامه نمیدونه داده کجا ذخیره میشه
//! - تست کردن راحت‌تر میشه (میشه mock کرد)
//! - تغییر دیتابیس آسون‌تر میشه
//!
//! ## مفاهیم Rust:
//! - **Traits**: تعریف interface
//! - **async_trait**: امکان async در traits
//! - **Generic Repository**: کار با هر نوع entity
//! - **Associated Types**: نوع‌های مرتبط با trait
//! - **Marker Traits**: traits بدون متد

use async_trait::async_trait;
use sqlx::FromRow;
use crate::error::Result;

// =====================================
// Base Repository Trait
// =====================================
/// Trait پایه برای همه Repository‌ها
///
/// # مفاهیم:
/// - `#[async_trait]`: macro برای async در traits
/// - `Send + Sync`: امکان ارسال بین threads
/// - Associated Types: `type Entity` و `type Id`
///
/// # چرا async_trait؟
/// قبل از Rust 1.75، async fn در trait مستقیم ممکن نبود.
/// این macro مشکل رو حل میکنه.
#[async_trait]
pub trait Repository: Send + Sync {
    /// نوع Entity که این repository باهاش کار میکنه
    type Entity: Send + Sync;
    
    /// نوع شناسه (ID)
    type Id: Send + Sync;
    
    /// پیدا کردن با ID
    async fn find_by_id(&self, id: &Self::Id) -> Result<Option<Self::Entity>>;
    
    /// پیدا کردن همه
    async fn find_all(&self) -> Result<Vec<Self::Entity>>;
    
    /// ذخیره کردن (insert)
    async fn save(&self, entity: &Self::Entity) -> Result<Self::Entity>;
    
    /// حذف با ID
    async fn delete(&self, id: &Self::Id) -> Result<bool>;
    
    /// شمارش کل
    async fn count(&self) -> Result<i64>;
}

// =====================================
// URL Repository
// =====================================
use super::Database;
use crate::models::{Url, CreateUrl};
use chrono::Utc;

/// Repository برای مدیریت URL‌ها
///
/// # مفاهیم:
/// - Struct با dependency injection
/// - Database به عنوان dependency تزریق میشه
#[derive(Debug, Clone)]
pub struct UrlRepository {
    db: Database,
}

impl UrlRepository {
    /// ساخت repository جدید
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
    
    /// پیدا کردن با short_code
    ///
    /// # مفاهیم:
    /// - `sqlx::query_as`: اجرای query و map به struct
    /// - `.fetch_optional()`: برگردوندن Option (0 یا 1 نتیجه)
    pub async fn find_by_short_code(&self, short_code: &str) -> Result<Option<Url>> {
        let url = sqlx::query_as::<_, Url>(
            r#"
            SELECT id, short_code, original_url, title, clicks, 
                   user_id, expires_at, created_at, updated_at
            FROM urls 
            WHERE short_code = ?
            "#
        )
        .bind(short_code)
        .fetch_optional(self.db.pool())
        .await?;
        
        Ok(url)
    }
    
    /// ایجاد URL جدید
    pub async fn create(&self, create_url: &CreateUrl) -> Result<Url> {
        let now = Utc::now();
        
        sqlx::query(
            r#"
            INSERT INTO urls (id, short_code, original_url, title, user_id, expires_at, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&create_url.id)
        .bind(&create_url.short_code)
        .bind(&create_url.original_url)
        .bind(&create_url.title)
        .bind(&create_url.user_id)
        .bind(&create_url.expires_at)
        .bind(now)
        .bind(now)
        .execute(self.db.pool())
        .await?;
        
        // خوندن URL ساخته شده
        self.find_by_id(&create_url.id)
            .await?
            .ok_or_else(|| crate::error::AppError::Internal("Failed to create URL".to_string()))
    }
    
    /// افزایش شمارنده کلیک
    ///
    /// # مفاهیم:
    /// - SQL UPDATE
    /// - Returning the updated value
    pub async fn increment_clicks(&self, short_code: &str) -> Result<Option<Url>> {
        let now = Utc::now();
        
        sqlx::query(
            r#"
            UPDATE urls 
            SET clicks = clicks + 1, updated_at = ?
            WHERE short_code = ?
            "#
        )
        .bind(now)
        .bind(short_code)
        .execute(self.db.pool())
        .await?;
        
        self.find_by_short_code(short_code).await
    }
    
    /// پیدا کردن URL‌های یک کاربر
    pub async fn find_by_user(&self, user_id: &str) -> Result<Vec<Url>> {
        let urls = sqlx::query_as::<_, Url>(
            r#"
            SELECT id, short_code, original_url, title, clicks,
                   user_id, expires_at, created_at, updated_at
            FROM urls 
            WHERE user_id = ?
            ORDER BY created_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(self.db.pool())
        .await?;
        
        Ok(urls)
    }
    
    /// چک کردن وجود short_code
    pub async fn exists(&self, short_code: &str) -> Result<bool> {
        let result = sqlx::query_scalar::<_, i32>(
            "SELECT COUNT(*) FROM urls WHERE short_code = ?"
        )
        .bind(short_code)
        .fetch_one(self.db.pool())
        .await?;
        
        Ok(result > 0)
    }
    
    /// حذف URL‌های منقضی شده
    pub async fn delete_expired(&self) -> Result<u64> {
        let now = Utc::now();
        
        let result = sqlx::query(
            "DELETE FROM urls WHERE expires_at IS NOT NULL AND expires_at < ?"
        )
        .bind(now)
        .execute(self.db.pool())
        .await?;
        
        Ok(result.rows_affected())
    }
    
    /// آمار کلی
    pub async fn get_stats(&self) -> Result<UrlStats> {
        let stats = sqlx::query_as::<_, UrlStats>(
            r#"
            SELECT 
                COUNT(*) as total_urls,
                COALESCE(SUM(clicks), 0) as total_clicks,
                COALESCE(AVG(clicks), 0) as avg_clicks
            FROM urls
            "#
        )
        .fetch_one(self.db.pool())
        .await?;
        
        Ok(stats)
    }
}

/// آمار URL‌ها
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct UrlStats {
    pub total_urls: i64,
    pub total_clicks: i64,
    pub avg_clicks: f64,
}

// پیاده‌سازی Repository trait برای UrlRepository
#[async_trait]
impl Repository for UrlRepository {
    type Entity = Url;
    type Id = String;
    
    async fn find_by_id(&self, id: &String) -> Result<Option<Url>> {
        let url = sqlx::query_as::<_, Url>(
            r#"
            SELECT id, short_code, original_url, title, clicks,
                   user_id, expires_at, created_at, updated_at
            FROM urls 
            WHERE id = ?
            "#
        )
        .bind(id)
        .fetch_optional(self.db.pool())
        .await?;
        
        Ok(url)
    }
    
    async fn find_all(&self) -> Result<Vec<Url>> {
        let urls = sqlx::query_as::<_, Url>(
            r#"
            SELECT id, short_code, original_url, title, clicks,
                   user_id, expires_at, created_at, updated_at
            FROM urls 
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(self.db.pool())
        .await?;
        
        Ok(urls)
    }
    
    async fn save(&self, entity: &Url) -> Result<Url> {
        let create_url = CreateUrl {
            id: entity.id.clone(),
            short_code: entity.short_code.clone(),
            original_url: entity.original_url.clone(),
            title: entity.title.clone(),
            user_id: entity.user_id.clone(),
            expires_at: entity.expires_at,
        };
        self.create(&create_url).await
    }
    
    async fn delete(&self, id: &String) -> Result<bool> {
        let result = sqlx::query("DELETE FROM urls WHERE id = ?")
            .bind(id)
            .execute(self.db.pool())
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn count(&self) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM urls")
            .fetch_one(self.db.pool())
            .await?;
        
        Ok(count)
    }
}

// =====================================
// User Repository
// =====================================
use crate::models::{User, CreateUser};

/// Repository برای مدیریت کاربران
#[derive(Debug, Clone)]
pub struct UserRepository {
    db: Database,
}

impl UserRepository {
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
    
    /// پیدا کردن با email
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, name, is_active, created_at, updated_at
            FROM users 
            WHERE email = ?
            "#
        )
        .bind(email)
        .fetch_optional(self.db.pool())
        .await?;
        
        Ok(user)
    }
    
    /// ایجاد کاربر جدید
    pub async fn create(&self, create_user: &CreateUser) -> Result<User> {
        let now = Utc::now();
        
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, name, is_active, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&create_user.id)
        .bind(&create_user.email)
        .bind(&create_user.password_hash)
        .bind(&create_user.name)
        .bind(true)
        .bind(now)
        .bind(now)
        .execute(self.db.pool())
        .await?;
        
        self.find_by_id(&create_user.id)
            .await?
            .ok_or_else(|| crate::error::AppError::Internal("Failed to create user".to_string()))
    }
    
    /// بررسی وجود email
    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let count = sqlx::query_scalar::<_, i32>(
            "SELECT COUNT(*) FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_one(self.db.pool())
        .await?;
        
        Ok(count > 0)
    }
}

#[async_trait]
impl Repository for UserRepository {
    type Entity = User;
    type Id = String;
    
    async fn find_by_id(&self, id: &String) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, name, is_active, created_at, updated_at
            FROM users 
            WHERE id = ?
            "#
        )
        .bind(id)
        .fetch_optional(self.db.pool())
        .await?;
        
        Ok(user)
    }
    
    async fn find_all(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, name, is_active, created_at, updated_at
            FROM users 
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(self.db.pool())
        .await?;
        
        Ok(users)
    }
    
    async fn save(&self, entity: &User) -> Result<User> {
        let create_user = CreateUser {
            id: entity.id.clone(),
            email: entity.email.clone(),
            password_hash: entity.password_hash.clone(),
            name: entity.name.clone(),
        };
        self.create(&create_user).await
    }
    
    async fn delete(&self, id: &String) -> Result<bool> {
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(self.db.pool())
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn count(&self) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(self.db.pool())
            .await?;
        
        Ok(count)
    }
}

