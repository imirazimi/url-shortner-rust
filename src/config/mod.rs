//! # ماژول تنظیمات (Configuration)
//!
//! این ماژول مسئول خوندن و مدیریت تنظیمات برنامه هست.
//!
//! ## مفاهیم Rust:
//! - **Structs**: ساختار داده‌ای برای نگهداری تنظیمات
//! - **Derive Macros**: تولید خودکار کد با `#[derive(...)]`
//! - **Default Trait**: مقادیر پیش‌فرض
//! - **Clone**: کپی کردن داده‌ها
//! - **Debug**: نمایش برای دیباگ
//! - **Serde**: سریالایز/دسریالایز
//! - **Builder Pattern**: ساخت تدریجی آبجکت

use std::env;
use serde::{Deserialize, Serialize};
use crate::error::{AppError, Result};

/// تنظیمات اصلی برنامه
///
/// # مفاهیم:
/// - `#[derive(...)]`: macro برای تولید خودکار implementation
/// - `Clone`: اجازه کپی کردن (deep copy)
/// - `Debug`: اجازه پرینت با `{:?}`
/// - `Serialize/Deserialize`: تبدیل به/از JSON و فرمت‌های دیگه
///
/// # مثال
/// ```rust
/// use url_shortener::config::Config;
///
/// let config = Config::default();
/// println!("Port: {}", config.port);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// آدرس هاست سرور
    pub host: String,
    
    /// پورت سرور
    pub port: u16,
    
    /// آدرس پایه برای لینک‌های کوتاه
    pub base_url: String,
    
    /// آدرس اتصال به دیتابیس
    pub database_url: String,
    
    /// کلید مخفی JWT
    pub jwt_secret: String,
    
    /// مدت اعتبار توکن JWT (ساعت)
    pub jwt_expiration_hours: u64,
    
    /// تعداد درخواست مجاز در ثانیه
    pub rate_limit_per_second: u32,
    
    /// حداکثر burst در rate limiting
    pub rate_limit_burst: u32,
    
    /// محیط اجرا (development, production)
    pub environment: Environment,
}

/// محیط اجرای برنامه
///
/// # مفاهیم:
/// - `enum`: نوع داده شمارشی
/// - `#[serde(rename_all = "lowercase")]`: تغییر نام‌گذاری در سریالایز
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    /// محیط توسعه - با قابلیت‌های دیباگ
    #[default]
    Development,
    
    /// محیط تست
    Testing,
    
    /// محیط تولید - بهینه‌سازی شده
    Production,
}

impl Environment {
    /// آیا در محیط توسعه هستیم؟
    ///
    /// # مفاهیم:
    /// - `&self`: رفرنس به خودش (borrowing)
    /// - `bool`: نوع بولین
    #[must_use]
    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Development)
    }
    
    /// آیا در محیط تولید هستیم؟
    #[must_use]
    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }
}

/// تبدیل String به Environment
///
/// # مفاهیم:
/// - `impl From<T>`: پیاده‌سازی trait تبدیل
/// - `match`: pattern matching
/// - `_`: wildcard برای بقیه حالت‌ها
impl From<String> for Environment {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "production" | "prod" => Environment::Production,
            "testing" | "test" => Environment::Testing,
            _ => Environment::Development,
        }
    }
}

/// مقادیر پیش‌فرض برای Config
///
/// # مفاهیم:
/// - `impl Default`: پیاده‌سازی trait Default
/// - این trait اجازه میده با `Config::default()` یه instance بسازیم
impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            base_url: "http://localhost:3000".to_string(),
            database_url: "sqlite://data/urls.db?mode=rwc".to_string(),
            jwt_secret: "change-me-in-production".to_string(),
            jwt_expiration_hours: 24,
            rate_limit_per_second: 10,
            rate_limit_burst: 30,
            environment: Environment::Development,
        }
    }
}

impl Config {
    /// ساخت تنظیمات از متغیرهای محیطی
    ///
    /// # مفاهیم:
    /// - `Result<Self>`: برگردوندن خودش یا خطا
    /// - `env::var()`: خوندن متغیر محیطی
    /// - `unwrap_or_else`: مقدار پیش‌فرض با closure
    /// - `?` operator: انتشار خطا
    /// - `parse()`: تبدیل String به نوع‌های دیگه
    ///
    /// # Errors
    /// خطا برمیگردونه اگه متغیرهای اجباری موجود نباشن
    ///
    /// # مثال
    /// ```rust,no_run
    /// use url_shortener::config::Config;
    ///
    /// let config = Config::from_env().expect("Failed to load config");
    /// ```
    pub fn from_env() -> Result<Self> {
        // helper function برای خوندن متغیر محیطی با default
        // این یه closure هست که به عنوان متغیر ذخیره شده
        let get_env = |key: &str, default: &str| -> String {
            env::var(key).unwrap_or_else(|_| default.to_string())
        };
        
        // helper برای parse کردن عدد
        let parse_env = |key: &str, default: u32| -> u32 {
            env::var(key)
                .ok()                           // تبدیل Result به Option
                .and_then(|v| v.parse().ok())   // parse و تبدیل به Option
                .unwrap_or(default)             // مقدار پیش‌فرض
        };
        
        Ok(Self {
            host: get_env("HOST", "127.0.0.1"),
            port: parse_env("PORT", 3000) as u16,
            base_url: get_env("BASE_URL", "http://localhost:3000"),
            database_url: get_env("DATABASE_URL", "sqlite://data/urls.db?mode=rwc"),
            jwt_secret: get_env("JWT_SECRET", "change-me-in-production"),
            jwt_expiration_hours: parse_env("JWT_EXPIRATION_HOURS", 24) as u64,
            rate_limit_per_second: parse_env("RATE_LIMIT_PER_SECOND", 10),
            rate_limit_burst: parse_env("RATE_LIMIT_BURST", 30),
            environment: get_env("ENVIRONMENT", "development").into(),
        })
    }
    
    /// اعتبارسنجی تنظیمات
    ///
    /// # مفاهیم:
    /// - `&self`: borrowing - فقط میخونه، تغییر نمیده
    /// - Early return: برگشت زودهنگام در صورت خطا
    pub fn validate(&self) -> Result<()> {
        // چک کردن که jwt_secret در production تغییر کرده باشه
        if self.environment.is_production() 
            && self.jwt_secret == "change-me-in-production" 
        {
            return Err(AppError::Config(
                "JWT_SECRET must be changed in production".to_string()
            ));
        }
        
        // چک کردن port
        if self.port == 0 {
            return Err(AppError::Config(
                "PORT cannot be 0".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// آدرس کامل سرور
    ///
    /// # مفاهیم:
    /// - `format!`: ماکرو برای ساخت String
    #[must_use]
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

// =====================================
// Builder Pattern
// =====================================
/// ساخت Config با Builder Pattern
///
/// # مفاهیم:
/// - Builder Pattern: ساخت تدریجی یک object
/// - Method Chaining: زنجیره‌ای کردن متدها
/// - Consuming self: گرفتن ownership در هر متد
///
/// # مثال
/// ```rust
/// use url_shortener::config::ConfigBuilder;
///
/// let config = ConfigBuilder::new()
///     .port(8080)
///     .host("0.0.0.0")
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    /// ساخت builder جدید
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }
    
    /// تنظیم پورت
    ///
    /// # مفاهیم:
    /// - `mut self`: گرفتن ownership و اجازه تغییر
    /// - `Self`: برگردوندن نوع خودش
    #[must_use]
    pub fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }
    
    /// تنظیم هاست
    #[must_use]
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.config.host = host.into();
        self
    }
    
    /// تنظیم base_url
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = url.into();
        self
    }
    
    /// تنظیم database_url
    #[must_use]
    pub fn database_url(mut self, url: impl Into<String>) -> Self {
        self.config.database_url = url.into();
        self
    }
    
    /// تنظیم jwt_secret
    #[must_use]
    pub fn jwt_secret(mut self, secret: impl Into<String>) -> Self {
        self.config.jwt_secret = secret.into();
        self
    }
    
    /// تنظیم محیط
    #[must_use]
    pub fn environment(mut self, env: Environment) -> Self {
        self.config.environment = env;
        self
    }
    
    /// ساخت Config نهایی
    ///
    /// این متد ownership رو میگیره و Config رو برمیگردونه
    #[must_use]
    pub fn build(self) -> Config {
        self.config
    }
    
    /// ساخت Config با اعتبارسنجی
    ///
    /// # Errors
    /// خطا برمیگردونه اگه اعتبارسنجی fail بشه
    pub fn build_validated(self) -> Result<Config> {
        let config = self.build();
        config.validate()?;
        Ok(config)
    }
}

// =====================================
// Tests
// =====================================
#[cfg(test)]
mod tests {
    use super::*;
    
    /// تست ساخت config با مقادیر پیش‌فرض
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.port, 3000);
        assert_eq!(config.host, "127.0.0.1");
    }
    
    /// تست Builder Pattern
    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .port(8080)
            .host("0.0.0.0")
            .build();
        
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0");
    }
    
    /// تست تبدیل Environment
    #[test]
    fn test_environment_from_string() {
        assert_eq!(Environment::from("production".to_string()), Environment::Production);
        assert_eq!(Environment::from("PROD".to_string()), Environment::Production);
        assert_eq!(Environment::from("development".to_string()), Environment::Development);
        assert_eq!(Environment::from("unknown".to_string()), Environment::Development);
    }
    
    /// تست اعتبارسنجی
    #[test]
    fn test_validation_fails_in_production_with_default_secret() {
        let config = ConfigBuilder::new()
            .environment(Environment::Production)
            .build();
        
        assert!(config.validate().is_err());
    }
}

