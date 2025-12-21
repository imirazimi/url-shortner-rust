//! # مدل کاربر (User Model)
//!
//! Entity و DTO‌های مربوط به کاربر

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

// =====================================
// User Entity
// =====================================
/// Entity کاربر
///
/// # مفاهیم:
/// - `FromRow`: تبدیل از ردیف دیتابیس
/// - `#[serde(skip)]`: این فیلد serialize نمیشه
/// - امنیت: password_hash هرگز به کلاینت ارسال نمیشه
#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    
    /// هش رمز عبور - این فیلد برای serialize در نظر گرفته نشده
    pub password_hash: String,
    
    pub name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// بررسی رمز عبور
    ///
    /// # مفاهیم:
    /// - استفاده از Argon2 برای هش رمز عبور
    /// - Password verification امن
    ///
    /// # Errors
    /// خطا برمیگردونه اگه verification fail بشه
    pub fn verify_password(&self, password: &str) -> crate::error::Result<bool> {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};
        
        let parsed_hash = PasswordHash::new(&self.password_hash)
            .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
        
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

/// تبدیل User به UserResponse
///
/// # مفاهیم:
/// - `impl From<T>`: تبدیل خودکار با `into()`
/// - این تضمین میکنه password_hash هیچوقت leak نشه
impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            is_active: user.is_active,
            created_at: user.created_at,
        }
    }
}

/// تبدیل &User به UserResponse
impl From<&User> for UserResponse {
    fn from(user: &User) -> Self {
        Self {
            id: user.id.clone(),
            email: user.email.clone(),
            name: user.name.clone(),
            is_active: user.is_active,
            created_at: user.created_at,
        }
    }
}

// =====================================
// Create User DTO
// =====================================
/// داده برای ساخت کاربر (داخلی)
#[derive(Debug, Clone)]
pub struct CreateUser {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub name: Option<String>,
}

impl CreateUser {
    /// ساخت کاربر جدید با هش کردن رمز عبور
    ///
    /// # مفاهیم:
    /// - Password hashing با Argon2
    /// - Salt خودکار تولید میشه
    ///
    /// # Errors
    /// خطا برمیگردونه اگه hashing fail بشه
    pub fn new(
        email: impl Into<String>,
        password: &str,
        name: Option<String>,
    ) -> crate::error::Result<Self> {
        use argon2::{
            password_hash::{rand_core::OsRng, SaltString},
            Argon2, PasswordHasher,
        };
        
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| crate::error::AppError::Internal(e.to_string()))?
            .to_string();
        
        Ok(Self {
            id: nanoid::nanoid!(21),
            email: email.into(),
            password_hash,
            name,
        })
    }
}

// =====================================
// API Request DTOs
// =====================================
/// درخواست ثبت‌نام
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    
    #[validate(length(min = 2, max = 100, message = "Name must be 2-100 characters"))]
    pub name: Option<String>,
}

/// درخواست ورود
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

/// درخواست بروزرسانی پروفایل
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 2, max = 100, message = "Name must be 2-100 characters"))]
    pub name: Option<String>,
}

/// درخواست تغییر رمز عبور
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,
    
    #[validate(length(min = 8, message = "New password must be at least 8 characters"))]
    pub new_password: String,
}

// =====================================
// API Response DTOs
// =====================================
/// پاسخ اطلاعات کاربر
///
/// توجه: password_hash اینجا نیست!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// پاسخ ورود موفق
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// پاسخ ثبت‌نام موفق
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user: UserResponse,
    pub message: String,
}

// =====================================
// JWT Claims
// =====================================
/// محتویات توکن JWT
///
/// # مفاهیم:
/// - Claims: اطلاعات داخل JWT
/// - `sub`: Subject (شناسه کاربر)
/// - `exp`: Expiration time (Unix timestamp)
/// - `iat`: Issued at (زمان صدور)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// شناسه کاربر
    pub sub: String,
    
    /// ایمیل کاربر
    pub email: String,
    
    /// زمان انقضا (Unix timestamp)
    pub exp: i64,
    
    /// زمان صدور
    pub iat: i64,
}

impl Claims {
    /// ساخت claims جدید
    #[must_use]
    pub fn new(user_id: &str, email: &str, expiration_hours: u64) -> Self {
        let now = Utc::now();
        let exp = now + chrono::Duration::hours(expiration_hours as i64);
        
        Self {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        }
    }
    
    /// آیا توکن منقضی شده؟
    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

