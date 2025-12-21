//! # ماژول توابع کمکی (Utilities)
//!
//! این ماژول توابع و ثابت‌های کمکی رو ارائه میده.
//!
//! ## مفاهیم Rust:
//! - **const fn**: توابعی که در compile-time اجرا میشن
//! - **static**: متغیرهای با عمر 'static
//! - **lazy_static / once_cell**: مقداردهی اولیه تنبل
//! - **Regex**: عبارات منظم

use once_cell::sync::Lazy;
use rand::Rng;
use regex::Regex;

// =====================================
// Constants
// =====================================
/// کاراکترهای مجاز برای short code
/// فقط حروف و اعداد برای جلوگیری از مشکلات URL
pub const SHORT_CODE_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// طول پیش‌فرض short code
pub const DEFAULT_SHORT_CODE_LENGTH: usize = 7;

/// حداکثر طول URL اصلی
pub const MAX_URL_LENGTH: usize = 2048;

/// حداقل طول short code سفارشی
pub const MIN_CUSTOM_CODE_LENGTH: usize = 3;

/// حداکثر طول short code سفارشی
pub const MAX_CUSTOM_CODE_LENGTH: usize = 20;

// =====================================
// Lazy Statics (Regex patterns)
// =====================================
/// الگوی معتبر برای short code
///
/// # مفاهیم:
/// - `Lazy`: مقداردهی اولیه در اولین استفاده
/// - این بهینه‌تر از ساخت Regex هر بار هست
/// - `pub static`: متغیر استاتیک عمومی با عمر 'static
pub static VALID_SHORT_CODE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9_-]+$").expect("Invalid regex pattern")
});

/// الگوی URL معتبر
pub static VALID_URL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^https?://[a-zA-Z0-9][-a-zA-Z0-9]*(\.[a-zA-Z0-9][-a-zA-Z0-9]*)+(/.*)?$"
    ).expect("Invalid regex pattern")
});

// =====================================
// Short Code Generation
// =====================================
/// تولید short code تصادفی
///
/// # مفاهیم:
/// - `rand::thread_rng()`: تولیدکننده اعداد تصادفی برای این thread
/// - `gen_range`: تولید عدد تصادفی در بازه
/// - Iterator: استفاده از iterator برای ساخت string
///
/// # مثال
/// ```rust
/// use url_shortener::utils::generate_short_code;
///
/// let code = generate_short_code();
/// assert_eq!(code.len(), 7);
/// ```
#[must_use]
pub fn generate_short_code() -> String {
    generate_short_code_with_length(DEFAULT_SHORT_CODE_LENGTH)
}

/// تولید short code با طول مشخص
///
/// # Arguments
/// * `length` - طول کد مورد نظر
///
/// # مثال
/// ```rust
/// use url_shortener::utils::generate_short_code_with_length;
///
/// let code = generate_short_code_with_length(10);
/// assert_eq!(code.len(), 10);
/// ```
#[must_use]
pub fn generate_short_code_with_length(length: usize) -> String {
    let mut rng = rand::thread_rng();
    
    // ساخت String با iterator
    // این یه pattern رایج در Rust هست
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..SHORT_CODE_CHARS.len());
            SHORT_CODE_CHARS[idx] as char
        })
        .collect()
}

/// تولید short code با استفاده از base62
///
/// # مفاهیم:
/// - Base62 encoding: 0-9, a-z, A-Z
/// - این روش از ID عددی یک کد کوتاه میسازه
#[must_use]
pub fn encode_id_to_short_code(id: u64) -> String {
    base62::encode(id)
}

/// دیکود کردن short code به ID
///
/// # Errors
/// خطا برمیگردونه اگه فرمت نامعتبر باشه
pub fn decode_short_code_to_id(code: &str) -> Result<u64, base62::DecodeError> {
    base62::decode(code)
}

// =====================================
// Validation Functions
// =====================================
/// اعتبارسنجی short code
///
/// # مفاهیم:
/// - `&str`: رفرنس به string (borrowing)
/// - `bool`: نوع بولین
/// - Regex matching
///
/// # مثال
/// ```rust
/// use url_shortener::utils::is_valid_short_code;
///
/// assert!(is_valid_short_code("abc123"));
/// assert!(!is_valid_short_code("abc 123")); // space نامعتبره
/// ```
#[must_use]
pub fn is_valid_short_code(code: &str) -> bool {
    let len = code.len();
    
    // چک کردن طول
    if len < MIN_CUSTOM_CODE_LENGTH || len > MAX_CUSTOM_CODE_LENGTH {
        return false;
    }
    
    // چک کردن کاراکترها
    VALID_SHORT_CODE.is_match(code)
}

/// اعتبارسنجی URL
///
/// # مفاهیم:
/// - استفاده از کتابخانه url برای parsing
/// - Pattern matching روی Result
#[must_use]
pub fn is_valid_url(url_str: &str) -> bool {
    // چک کردن طول
    if url_str.len() > MAX_URL_LENGTH {
        return false;
    }
    
    // استفاده از url crate برای parsing
    match url::Url::parse(url_str) {
        Ok(url) => {
            // فقط http و https مجازه
            let scheme = url.scheme();
            scheme == "http" || scheme == "https"
        }
        Err(_) => false,
    }
}

/// نرمالایز کردن URL
///
/// حذف trailing slash، lowercase scheme و host
#[must_use]
pub fn normalize_url(url_str: &str) -> Option<String> {
    url::Url::parse(url_str).ok().map(|url| {
        url.to_string().trim_end_matches('/').to_string()
    })
}

// =====================================
// String Utilities
// =====================================
/// خلاصه کردن متن طولانی
///
/// # مفاهیم:
/// - `Option<usize>`: پارامتر اختیاری
/// - `unwrap_or`: مقدار پیش‌فرض
/// - Unicode-aware truncation با `char_indices`
///
/// # مثال
/// ```rust
/// use url_shortener::utils::truncate;
///
/// let text = "Hello, World!";
/// assert_eq!(truncate(text, Some(5)), "Hello...");
/// ```
#[must_use]
pub fn truncate(text: &str, max_len: Option<usize>) -> String {
    let max = max_len.unwrap_or(100);
    
    // اگه کوتاه‌تر از حد بود، همونو برگردون
    if text.len() <= max {
        return text.to_string();
    }
    
    // Unicode-safe truncation
    // char_indices کاراکترهای یونیکد رو درست میشکونه
    let truncated: String = text
        .char_indices()
        .take_while(|(i, _)| *i < max - 3)
        .map(|(_, c)| c)
        .collect();
    
    format!("{}...", truncated)
}

/// تمیز کردن whitespace‌های اضافی
#[must_use]
pub fn clean_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

// =====================================
// Time Utilities
// =====================================
use chrono::{DateTime, Duration, Utc};

/// محاسبه تاریخ انقضا
#[must_use]
pub fn expires_at_from_hours(hours: u32) -> DateTime<Utc> {
    Utc::now() + Duration::hours(hours as i64)
}

/// فرمت مدت زمان به صورت خوانا
///
/// # مثال
/// ```rust
/// use url_shortener::utils::format_duration;
///
/// assert_eq!(format_duration(3661), "1h 1m 1s");
/// assert_eq!(format_duration(60), "1m 0s");
/// ```
#[must_use]
pub fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

// =====================================
// Security Utilities
// =====================================
/// تولید کلید تصادفی برای API keys و غیره
///
/// # مفاهیم:
/// - Cryptographically secure random generation
/// - Hex encoding
#[must_use]
pub fn generate_secure_token(length: usize) -> String {
    use rand::RngCore;
    
    let mut bytes = vec![0u8; length / 2];
    rand::thread_rng().fill_bytes(&mut bytes);
    
    // تبدیل به hex
    bytes.iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

/// Mask کردن بخشی از متن (برای لاگ‌ها)
///
/// # مثال
/// ```rust
/// use url_shortener::utils::mask_string;
///
/// assert_eq!(mask_string("secret123", 3), "sec***");
/// ```
#[must_use]
pub fn mask_string(text: &str, visible_chars: usize) -> String {
    if text.len() <= visible_chars {
        return "*".repeat(text.len());
    }
    
    let visible: String = text.chars().take(visible_chars).collect();
    format!("{}***", visible)
}

// =====================================
// Tests
// =====================================
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_short_code() {
        let code = generate_short_code();
        assert_eq!(code.len(), DEFAULT_SHORT_CODE_LENGTH);
        assert!(is_valid_short_code(&code));
    }
    
    #[test]
    fn test_generate_short_code_with_length() {
        for len in [5, 10, 15] {
            let code = generate_short_code_with_length(len);
            assert_eq!(code.len(), len);
        }
    }
    
    #[test]
    fn test_valid_short_code() {
        assert!(is_valid_short_code("abc123"));
        assert!(is_valid_short_code("ABC-xyz_123"));
        assert!(!is_valid_short_code("ab")); // too short
        assert!(!is_valid_short_code("abc 123")); // space
        assert!(!is_valid_short_code("abc@123")); // special char
    }
    
    #[test]
    fn test_valid_url() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://example.com/path?q=1"));
        assert!(!is_valid_url("ftp://example.com"));
        assert!(!is_valid_url("not a url"));
    }
    
    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", Some(10)), "short");
        assert_eq!(truncate("this is a long text", Some(10)), "this is...");
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m 1s");
    }
    
    #[test]
    fn test_base62_encoding() {
        let id = 123456789u64;
        let encoded = encode_id_to_short_code(id);
        let decoded = decode_short_code_to_id(&encoded).unwrap();
        assert_eq!(id, decoded);
    }
    
    #[test]
    fn test_mask_string() {
        assert_eq!(mask_string("secret123", 3), "sec***");
        assert_eq!(mask_string("ab", 5), "**");
    }
}

