//! # ماژول دیتابیس (Database Layer)
//!
//! این ماژول لایه ارتباط با دیتابیس رو مدیریت میکنه.
//!
//! ## مفاهیم Rust:
//! - **Arc<T>**: Reference Counting برای اشتراک داده بین threads
//! - **async/await**: برنامه‌نویسی غیرهمزمان
//! - **Traits**: تعریف interface
//! - **async_trait**: پشتیبانی از async در traits
//! - **Generic Bounds**: محدودیت روی generic‌ها
//! - **Lifetimes**: مدیریت طول عمر references
//!
//! ## الگوهای طراحی:
//! - Repository Pattern: جداسازی لایه داده از منطق
//! - Connection Pool: مدیریت اتصالات دیتابیس

mod repository;

pub use repository::*;

use std::sync::Arc;
use sqlx::{sqlite::{SqlitePool, SqlitePoolOptions}, migrate::Migrator};
use crate::error::Result;

// مسیر migration‌ها
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

// =====================================
// Database Connection
// =====================================
/// اتصال به دیتابیس با Connection Pool
///
/// # مفاهیم:
/// - `#[derive(Clone)]`: Clone implementation
/// - `Arc<SqlitePool>`: Reference counting برای thread-safe sharing
///
/// ## چرا Arc؟
/// - `Arc` (Atomic Reference Counted) اجازه میده یک داده رو بین چند thread share کنیم
/// - هر clone فقط counter رو زیاد میکنه، داده کپی نمیشه
/// - وقتی همه reference‌ها drop شن، داده آزاد میشه
#[derive(Debug, Clone)]
pub struct Database {
    /// Connection pool
    /// Arc برای share کردن بین handlers مختلف
    pool: Arc<SqlitePool>,
}

impl Database {
    /// اتصال به دیتابیس
    ///
    /// # مفاهیم:
    /// - `async fn`: تابع غیرهمزمان
    /// - `impl AsRef<str>`: هر نوعی که بتونه به &str تبدیل بشه
    /// - `.await`: صبر کردن برای نتیجه async
    ///
    /// # Arguments
    /// * `database_url` - آدرس دیتابیس (مثلا `sqlite://data/urls.db`)
    ///
    /// # Errors
    /// خطا برمیگردونه اگه اتصال موفق نباشه
    pub async fn connect(database_url: impl AsRef<str>) -> Result<Self> {
        // ساخت پوشه data اگه وجود نداره
        let url = database_url.as_ref();
        if url.starts_with("sqlite://") {
            // استخراج مسیر فایل
            if let Some(path) = url.strip_prefix("sqlite://") {
                // حذف query parameters
                let path = path.split('?').next().unwrap_or(path);
                if let Some(parent) = std::path::Path::new(path).parent() {
                    // ایجاد پوشه اگه وجود نداره
                    // `?` خطا رو propagate میکنه
                    std::fs::create_dir_all(parent)?;
                }
            }
        }
        
        // ساخت connection pool
        // Builder pattern برای تنظیمات
        let pool = SqlitePoolOptions::new()
            .max_connections(10)           // حداکثر 10 اتصال همزمان
            .min_connections(1)            // حداقل 1 اتصال
            .acquire_timeout(std::time::Duration::from_secs(5))
            .idle_timeout(std::time::Duration::from_secs(600))
            .connect(url)
            .await?;
        
        Ok(Self {
            pool: Arc::new(pool),
        })
    }
    
    /// اجرای migration‌ها
    ///
    /// Migration‌ها تغییرات schema دیتابیس رو مدیریت میکنن
    pub async fn migrate(&self) -> Result<()> {
        MIGRATOR.run(&*self.pool).await?;
        Ok(())
    }
    
    /// دسترسی به pool
    ///
    /// # مفاهیم:
    /// - `&self`: borrowing - فقط reference میگیره
    /// - `&SqlitePool`: reference به pool برمیگردونه
    #[must_use]
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
    
    /// بررسی سلامت دیتابیس
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&*self.pool)
            .await?;
        Ok(())
    }
}

// =====================================
// Transaction Support
// =====================================
/// Wrapper برای تراکنش‌ها
///
/// # مفاهیم:
/// - Transactions: گروه‌بندی عملیات - همه موفق یا همه لغو
/// - RAII (Resource Acquisition Is Initialization):
///   وقتی Transaction drop بشه، rollback میشه مگه commit شده باشه
impl Database {
    /// شروع یک تراکنش
    ///
    /// # مفاهیم:
    /// - تراکنش ACID properties رو تضمین میکنه
    /// - اگه commit نشه، خودکار rollback میشه
    ///
    /// # مثال
    /// ```rust,ignore
    /// let mut tx = db.begin().await?;
    /// // عملیات‌ها
    /// tx.commit().await?;
    /// ```
    pub async fn begin(&self) -> Result<sqlx::Transaction<'_, sqlx::Sqlite>> {
        Ok(self.pool.begin().await?)
    }
    
    /// اجرای یک closure در تراکنش
    ///
    /// # مفاهیم:
    /// - Higher-order function: تابع که تابع میگیره
    /// - Closure: تابع ناشناس
    /// - `async move`: منتقل کردن ownership به closure
    /// - `Fn` trait: نوع closure
    ///
    /// # مثال
    /// ```rust,ignore
    /// db.transaction(|tx| async move {
    ///     // عملیات‌ها
    ///     Ok(())
    /// }).await?;
    /// ```
    pub async fn transaction<F, T, Fut>(&self, f: F) -> Result<T>
    where
        F: FnOnce(sqlx::Transaction<'_, sqlx::Sqlite>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let tx = self.begin().await?;
        f(tx).await
    }
}

// =====================================
// Test Utilities
// =====================================
#[cfg(test)]
impl Database {
    /// ساخت دیتابیس in-memory برای تست
    pub async fn in_memory() -> Result<Self> {
        // :memory: یه دیتابیس موقت در RAM میسازه
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await?;
        
        let db = Self {
            pool: Arc::new(pool),
        };
        
        db.migrate().await?;
        Ok(db)
    }
}

