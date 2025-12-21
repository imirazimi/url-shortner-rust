//! # تست‌های Integration
//!
//! این فایل تست‌های end-to-end رو شامل میشه.
//!
//! ## مفاهیم Rust در تست‌ها:
//! - `#[cfg(test)]`: کامپایل فقط در حالت تست
//! - `#[tokio::test]`: تست‌های async
//! - `assert!`, `assert_eq!`: ماکروهای assertion
//! - `#[should_panic]`: انتظار panic
//! - `#[ignore]`: نادیده گرفتن تست
//!
//! ## اجرای تست‌ها:
//! ```bash
//! cargo test                    # همه تست‌ها
//! cargo test --lib              # فقط تست‌های unit
//! cargo test --test integration_tests  # فقط این فایل
//! cargo test url_               # تست‌هایی که با url_ شروع میشن
//! ```

use std::time::Duration;

// =====================================
// تست‌های Utils
// =====================================
mod utils_tests {
    use url_shortener::utils;
    
    /// تست تولید short code
    #[test]
    fn test_generate_short_code_length() {
        let code = utils::generate_short_code();
        assert_eq!(code.len(), utils::DEFAULT_SHORT_CODE_LENGTH);
    }
    
    /// تست یکتا بودن short code‌ها
    #[test]
    fn test_short_codes_are_unique() {
        let codes: Vec<String> = (0..100)
            .map(|_| utils::generate_short_code())
            .collect();
        
        // چک کردن یکتا بودن با HashSet
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), codes.len());
    }
    
    /// تست اعتبارسنجی URL
    #[test]
    fn test_url_validation() {
        // URL‌های معتبر
        assert!(utils::is_valid_url("https://example.com"));
        assert!(utils::is_valid_url("http://example.com/path"));
        assert!(utils::is_valid_url("https://sub.example.com/path?query=1"));
        
        // URL‌های نامعتبر
        assert!(!utils::is_valid_url("ftp://example.com"));
        assert!(!utils::is_valid_url("not a url"));
        assert!(!utils::is_valid_url(""));
    }
    
    /// تست اعتبارسنجی short code
    #[test]
    fn test_short_code_validation() {
        // معتبر
        assert!(utils::is_valid_short_code("abc123"));
        assert!(utils::is_valid_short_code("ABC-xyz_123"));
        
        // نامعتبر
        assert!(!utils::is_valid_short_code("ab")); // خیلی کوتاه
        assert!(!utils::is_valid_short_code("abc 123")); // space
        assert!(!utils::is_valid_short_code("abc@123")); // کاراکتر خاص
    }
    
    /// تست truncate
    #[test]
    fn test_truncate_function() {
        assert_eq!(utils::truncate("short", Some(10)), "short");
        assert_eq!(utils::truncate("this is long", Some(7)), "this...");
    }
    
    /// تست format_duration
    #[test]
    fn test_format_duration() {
        assert_eq!(utils::format_duration(30), "30s");
        assert_eq!(utils::format_duration(90), "1m 30s");
        assert_eq!(utils::format_duration(3661), "1h 1m 1s");
    }
    
    /// تست base62 encoding
    #[test]
    fn test_base62_roundtrip() {
        for id in [1u64, 42, 123456789, u64::MAX] {
            let encoded = utils::encode_id_to_short_code(id);
            let decoded = utils::decode_short_code_to_id(&encoded).unwrap();
            assert_eq!(id, decoded);
        }
    }
}

// =====================================
// تست‌های Config
// =====================================
mod config_tests {
    use url_shortener::config::{Config, ConfigBuilder, Environment};
    
    /// تست مقادیر پیش‌فرض
    #[test]
    fn test_default_config() {
        let config = Config::default();
        
        assert_eq!(config.port, 3000);
        assert_eq!(config.host, "127.0.0.1");
        assert!(config.environment.is_development());
    }
    
    /// تست Builder Pattern
    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .port(8080)
            .host("0.0.0.0")
            .base_url("https://short.io")
            .environment(Environment::Production)
            .build();
        
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.base_url, "https://short.io");
        assert!(config.environment.is_production());
    }
    
    /// تست تبدیل Environment
    #[test]
    fn test_environment_conversion() {
        assert_eq!(
            Environment::from("production".to_string()),
            Environment::Production
        );
        assert_eq!(
            Environment::from("PROD".to_string()),
            Environment::Production
        );
        assert_eq!(
            Environment::from("development".to_string()),
            Environment::Development
        );
        assert_eq!(
            Environment::from("unknown".to_string()),
            Environment::Development  // default
        );
    }
    
    /// تست اعتبارسنجی در production
    #[test]
    fn test_validation_production_secret() {
        let config = ConfigBuilder::new()
            .environment(Environment::Production)
            .jwt_secret("change-me-in-production")  // default secret
            .build();
        
        // باید fail بشه
        assert!(config.validate().is_err());
        
        // با secret جدید باید pass بشه
        let config = ConfigBuilder::new()
            .environment(Environment::Production)
            .jwt_secret("my-super-secret-key-123")
            .build();
        
        assert!(config.validate().is_ok());
    }
}

// =====================================
// تست‌های Error
// =====================================
mod error_tests {
    use url_shortener::error::{AppError, OptionExt, ResultExt};
    use axum::http::StatusCode;
    
    /// تست status codes
    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            AppError::NotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::BadRequest("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::Unauthorized("test".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::Internal("test".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
    
    /// تست is_server_error
    #[test]
    fn test_is_server_error() {
        assert!(AppError::Internal("test".to_string()).is_server_error());
        assert!(!AppError::NotFound("test".to_string()).is_server_error());
    }
    
    /// تست OptionExt
    #[test]
    fn test_option_extension() {
        let some_value: Option<i32> = Some(42);
        let none_value: Option<i32> = None;
        
        assert!(some_value.ok_or_not_found("not found").is_ok());
        
        let err = none_value.ok_or_not_found("item not found");
        assert!(matches!(err, Err(AppError::NotFound(_))));
    }
    
    /// تست ResultExt
    #[test]
    fn test_result_extension() {
        let ok: Result<i32, &str> = Ok(42);
        let err: Result<i32, &str> = Err("original error");
        
        assert!(ok.map_internal().is_ok());
        
        let mapped = err.map_internal();
        assert!(matches!(mapped, Err(AppError::Internal(_))));
    }
}

// =====================================
// تست‌های Models
// =====================================
mod model_tests {
    use url_shortener::models::{Id, Pagination, PaginationInfo, Claims};
    use chrono::Utc;
    
    /// تست Id
    #[test]
    fn test_id_generation() {
        let id1 = Id::new();
        let id2 = Id::new();
        
        // باید یکتا باشن
        assert_ne!(id1.as_str(), id2.as_str());
        
        // طول ثابت
        assert_eq!(id1.as_str().len(), 21);
    }
    
    /// تست Pagination
    #[test]
    fn test_pagination_offset() {
        let page1 = Pagination { page: 1, per_page: 20 };
        let page2 = Pagination { page: 2, per_page: 20 };
        let page3 = Pagination { page: 3, per_page: 10 };
        
        assert_eq!(page1.offset(), 0);
        assert_eq!(page2.offset(), 20);
        assert_eq!(page3.offset(), 20);
    }
    
    /// تست pagination limit
    #[test]
    fn test_pagination_limit() {
        let normal = Pagination { page: 1, per_page: 20 };
        let large = Pagination { page: 1, per_page: 200 };
        
        assert_eq!(normal.limit(), 20);
        assert_eq!(large.limit(), 100);  // حداکثر 100
    }
    
    /// تست PaginationInfo
    #[test]
    fn test_pagination_info() {
        let pagination = Pagination { page: 2, per_page: 10 };
        let info = PaginationInfo::new(&pagination, 35);
        
        assert_eq!(info.total_pages, 4);  // ceil(35/10)
        assert!(info.has_next);           // page 2 < 4
        assert!(info.has_prev);           // page 2 > 1
    }
    
    /// تست Claims expiration
    #[test]
    fn test_claims_expiration() {
        // توکن با 1 ساعت انقضا
        let claims = Claims::new("user1", "test@test.com", 1);
        assert!(!claims.is_expired());
        
        // توکن منقضی شده (دستی)
        let expired = Claims {
            sub: "user1".to_string(),
            email: "test@test.com".to_string(),
            exp: Utc::now().timestamp() - 3600,
            iat: Utc::now().timestamp() - 7200,
        };
        assert!(expired.is_expired());
    }
}

// =====================================
// تست‌های Async (با Database)
// =====================================
#[cfg(test)]
mod async_tests {
    use super::*;
    
    /// تست اتصال به دیتابیس
    /// 
    /// # مفاهیم:
    /// - `#[tokio::test]`: تست async
    /// - در production از mock استفاده کنید
    #[tokio::test]
    #[ignore]  // نیاز به دیتابیس واقعی داره
    async fn test_database_connection() {
        // این تست نیاز به setup دیتابیس داره
        // در CI/CD معمولا از testcontainers استفاده میشه
    }
}

// =====================================
// Property-Based Tests
// =====================================
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use url_shortener::utils;
    
    proptest! {
        /// تست که هر short code تولید شده valid هست
        /// 
        /// # مفاهیم:
        /// - Property-based testing: تست با ورودی‌های تصادفی
        /// - proptest: کتابخانه PBT در Rust
        #[test]
        fn generated_codes_are_valid(len in 5usize..20) {
            let code = utils::generate_short_code_with_length(len);
            prop_assert_eq!(code.len(), len);
            prop_assert!(utils::is_valid_short_code(&code));
        }
        
        /// تست base62 encoding با هر عدد
        #[test]
        fn base62_roundtrip(id: u64) {
            let encoded = utils::encode_id_to_short_code(id);
            let decoded = utils::decode_short_code_to_id(&encoded);
            prop_assert_eq!(decoded.ok(), Some(id));
        }
    }
}

// =====================================
// Benchmark-style Tests
// =====================================
#[cfg(test)]
mod performance_tests {
    use url_shortener::utils;
    use std::time::Instant;
    
    /// تست سرعت تولید short code
    /// 
    /// این یه benchmark ساده‌ست، برای واقعی از criterion استفاده کنید
    #[test]
    fn benchmark_short_code_generation() {
        let iterations = 10_000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = utils::generate_short_code();
        }
        
        let duration = start.elapsed();
        let per_operation = duration / iterations;
        
        println!("Generated {} short codes in {:?}", iterations, duration);
        println!("Average: {:?} per code", per_operation);
        
        // باید کمتر از 1ms برای هر کد باشه
        assert!(per_operation.as_millis() < 1);
    }
}

