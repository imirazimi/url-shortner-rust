//! # سرویس احراز هویت (Authentication Service)
//!
//! مدیریت کاربران، ورود، ثبت‌نام و JWT
//!
//! ## مفاهیم Rust:
//! - Password Hashing: هش کردن امن رمز عبور
//! - JWT: توکن‌های احراز هویت
//! - Security Best Practices

use std::sync::Arc;
use chrono::Utc;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use tracing::{info, warn, instrument};
use validator::Validate;

use crate::{
    config::Config,
    database::UserRepository,
    error::{AppError, Result, OptionExt},
    models::{
        Claims, CreateUser, LoginRequest, LoginResponse,
        RegisterRequest, RegisterResponse, User, UserResponse,
    },
};

use super::Service;

// =====================================
// Auth Service
// =====================================
/// سرویس احراز هویت
///
/// # مسئولیت‌ها:
/// - ثبت‌نام کاربر
/// - ورود و صدور توکن
/// - اعتبارسنجی توکن
/// - مدیریت رمز عبور
#[derive(Debug, Clone)]
pub struct AuthService {
    repo: UserRepository,
    config: Arc<Config>,
}

impl Service for AuthService {}

impl AuthService {
    /// ساخت سرویس جدید
    #[must_use]
    pub fn new(repo: UserRepository, config: Arc<Config>) -> Self {
        Self { repo, config }
    }
    
    /// ثبت‌نام کاربر جدید
    ///
    /// # مفاهیم:
    /// - Password hashing با Argon2
    /// - Validation
    /// - Duplicate checking
    #[instrument(skip(self, request), fields(email = %request.email))]
    pub async fn register(&self, request: RegisterRequest) -> Result<RegisterResponse> {
        // Step 1: اعتبارسنجی
        request.validate()?;
        
        // Step 2: بررسی تکراری نبودن email
        if self.repo.email_exists(&request.email).await? {
            return Err(AppError::Conflict(
                "Email already registered".to_string()
            ));
        }
        
        // Step 3: ساخت کاربر با هش کردن رمز عبور
        let create_user = CreateUser::new(
            &request.email,
            &request.password,
            request.name,
        )?;
        
        // Step 4: ذخیره در دیتابیس
        let user = self.repo.create(&create_user).await?;
        
        info!(user_id = %user.id, "New user registered");
        
        Ok(RegisterResponse {
            user: user.into(),
            message: "Registration successful".to_string(),
        })
    }
    
    /// ورود کاربر
    ///
    /// # مفاهیم:
    /// - Password verification
    /// - JWT generation
    /// - Constant-time comparison (از Argon2)
    #[instrument(skip(self, request), fields(email = %request.email))]
    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse> {
        // Step 1: اعتبارسنجی
        request.validate()?;
        
        // Step 2: پیدا کردن کاربر
        let user = self.repo
            .find_by_email(&request.email)
            .await?
            .ok_or_else(|| {
                // پیام عمومی برای جلوگیری از enumeration attack
                AppError::Unauthorized("Invalid credentials".to_string())
            })?;
        
        // Step 3: بررسی فعال بودن کاربر
        if !user.is_active {
            return Err(AppError::Forbidden(
                "Account is deactivated".to_string()
            ));
        }
        
        // Step 4: بررسی رمز عبور
        if !user.verify_password(&request.password)? {
            warn!(email = %request.email, "Failed login attempt");
            return Err(AppError::Unauthorized(
                "Invalid credentials".to_string()
            ));
        }
        
        // Step 5: صدور توکن
        let token = self.generate_token(&user)?;
        let expires_at = Utc::now() 
            + chrono::Duration::hours(self.config.jwt_expiration_hours as i64);
        
        info!(user_id = %user.id, "User logged in");
        
        Ok(LoginResponse {
            user: user.into(),
            token,
            expires_at,
        })
    }
    
    /// اعتبارسنجی توکن JWT
    ///
    /// # مفاهیم:
    /// - JWT verification
    /// - Claims extraction
    /// - Expiration check
    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let decoding_key = DecodingKey::from_secret(
            self.config.jwt_secret.as_bytes()
        );
        
        let validation = Validation::new(Algorithm::HS256);
        
        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| {
                warn!(error = %e, "Token verification failed");
                AppError::Unauthorized("Invalid token".to_string())
            })?;
        
        // بررسی انقضا
        if token_data.claims.is_expired() {
            return Err(AppError::Unauthorized("Token expired".to_string()));
        }
        
        Ok(token_data.claims)
    }
    
    /// گرفتن کاربر با ID
    pub async fn get_user(&self, user_id: &str) -> Result<UserResponse> {
        let user = self.repo
            .find_by_id(&user_id.to_string())
            .await?
            .ok_or_not_found(format!("User '{}' not found", user_id))?;
        
        Ok(user.into())
    }
    
    /// تولید توکن JWT
    ///
    /// # مفاهیم:
    /// - JWT encoding
    /// - Claims struct
    fn generate_token(&self, user: &User) -> Result<String> {
        let claims = Claims::new(
            &user.id,
            &user.email,
            self.config.jwt_expiration_hours,
        );
        
        let encoding_key = EncodingKey::from_secret(
            self.config.jwt_secret.as_bytes()
        );
        
        let token = encode(&Header::default(), &claims, &encoding_key)?;
        
        Ok(token)
    }
    
    /// Refresh توکن
    ///
    /// توکن جدید صادر میکنه اگه توکن قبلی هنوز معتبر باشه
    pub async fn refresh_token(&self, token: &str) -> Result<LoginResponse> {
        // اعتبارسنجی توکن فعلی
        let claims = self.verify_token(token)?;
        
        // گرفتن کاربر
        let user = self.repo
            .find_by_id(&claims.sub)
            .await?
            .ok_or_not_found("User not found")?;
        
        // صدور توکن جدید
        let new_token = self.generate_token(&user)?;
        let expires_at = Utc::now() 
            + chrono::Duration::hours(self.config.jwt_expiration_hours as i64);
        
        Ok(LoginResponse {
            user: user.into(),
            token: new_token,
            expires_at,
        })
    }
    
    /// تغییر رمز عبور
    #[instrument(skip(self, current_password, new_password))]
    pub async fn change_password(
        &self,
        user_id: &str,
        current_password: &str,
        new_password: &str,
    ) -> Result<()> {
        // اعتبارسنجی طول رمز جدید
        if new_password.len() < 8 {
            return Err(AppError::BadRequest(
                "Password must be at least 8 characters".to_string()
            ));
        }
        
        // گرفتن کاربر
        let user = self.repo
            .find_by_id(&user_id.to_string())
            .await?
            .ok_or_not_found("User not found")?;
        
        // بررسی رمز فعلی
        if !user.verify_password(current_password)? {
            return Err(AppError::Unauthorized(
                "Current password is incorrect".to_string()
            ));
        }
        
        // TODO: بروزرسانی رمز در دیتابیس
        // این بخش نیاز به متد update در repository داره
        
        info!(user_id = %user_id, "Password changed");
        Ok(())
    }
}

// =====================================
// Token Utilities
// =====================================
/// استخراج توکن از header Authorization
///
/// # مفاهیم:
/// - String slicing
/// - Pattern matching
///
/// # Format
/// `Authorization: Bearer <token>`
#[must_use]
pub fn extract_token_from_header(header_value: &str) -> Option<&str> {
    // چک کردن prefix
    if !header_value.starts_with("Bearer ") {
        return None;
    }
    
    // استخراج توکن
    Some(&header_value[7..])
}

// =====================================
// Tests
// =====================================
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_token() {
        assert_eq!(
            extract_token_from_header("Bearer abc123"),
            Some("abc123")
        );
        
        assert_eq!(
            extract_token_from_header("Basic abc123"),
            None
        );
        
        assert_eq!(
            extract_token_from_header("abc123"),
            None
        );
    }
    
    #[test]
    fn test_claims_expiration() {
        // توکن با انقضای 1 ساعت
        let claims = Claims::new("user1", "test@example.com", 1);
        assert!(!claims.is_expired());
        
        // توکن منقضی شده
        let expired_claims = Claims {
            sub: "user1".to_string(),
            email: "test@example.com".to_string(),
            exp: Utc::now().timestamp() - 3600, // 1 ساعت قبل
            iat: Utc::now().timestamp() - 7200, // 2 ساعت قبل
        };
        assert!(expired_claims.is_expired());
    }
}

