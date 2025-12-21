# 🔗 URL Shortener - یادگیری Rust Backend

یک پروژه کامل URL Shortener که برای آموزش مفاهیم Rust و توسعه Backend طراحی شده.

## 📚 هدف این پروژه

این پروژه به عنوان یک منبع آموزشی طراحی شده تا:

1. **مفاهیم Rust** رو در context واقعی یاد بگیرید
2. **ساختار پروژه بکند** رو درک کنید
3. **الگوهای طراحی** رو ببینید
4. **Best Practices** رو یاد بگیرید

---

## 🏗️ ساختار پروژه

```
url-shortener/
├── Cargo.toml              # مدیریت وابستگی‌ها
├── .gitignore
├── README.md
│
├── migrations/             # تغییرات دیتابیس
│   └── 20241221_001_initial.sql
│
├── src/
│   ├── main.rs            # نقطه ورود برنامه
│   ├── lib.rs             # نقطه ورود کتابخانه
│   │
│   ├── config/            # 📝 مدیریت تنظیمات
│   │   └── mod.rs
│   │
│   ├── error/             # ⚠️ مدیریت خطاها
│   │   └── mod.rs
│   │
│   ├── database/          # 🗄️ لایه دیتابیس
│   │   ├── mod.rs
│   │   └── repository.rs
│   │
│   ├── models/            # 📦 مدل‌های داده
│   │   ├── mod.rs
│   │   ├── url.rs
│   │   ├── user.rs
│   │   └── dto.rs
│   │
│   ├── services/          # 🔧 منطق کسب‌وکار
│   │   ├── mod.rs
│   │   ├── url_service.rs
│   │   └── auth_service.rs
│   │
│   ├── api/               # 🌐 لایه HTTP
│   │   ├── mod.rs
│   │   ├── extractors.rs
│   │   ├── middleware.rs
│   │   └── handlers/
│   │       ├── mod.rs
│   │       ├── url.rs
│   │       ├── auth.rs
│   │       ├── user.rs
│   │       ├── health.rs
│   │       └── stats.rs
│   │
│   └── utils/             # 🛠️ توابع کمکی
│       └── mod.rs
│
└── tests/                 # 🧪 تست‌ها
    └── integration_tests.rs
```

---

## 🦀 مفاهیم Rust در این پروژه

### 1. Ownership و Borrowing

```rust
// ❌ This code won't work - ownership moved
fn take_ownership(s: String) {
    println!("{}", s);
}
let s = String::from("hello");
take_ownership(s);
// println!("{}", s);  // Error: value moved

// ✅ With borrowing
fn borrow(s: &str) {
    println!("{}", s);
}
let s = String::from("hello");
borrow(&s);
println!("{}", s);  // OK!
```

**کجا در پروژه:**
- `&self` در متدها
- `impl AsRef<str>` برای پارامترها
- `String` vs `&str`

### 2. Result و Error Handling

```rust
// Result type
pub type Result<T, E = AppError> = std::result::Result<T, E>;

// Using ? operator
async fn create_url(&self, request: CreateUrlRequest) -> Result<Url> {
    request.validate()?;  // Returns early if error
    let url = self.repo.create(&create_url).await?;
    Ok(url)
}
```

**کجا در پروژه:**
- `src/error/mod.rs` - تعریف خطاها
- همه جا با `?` operator

### 3. Traits

```rust
// Trait definition
pub trait Repository: Send + Sync {
    type Entity: Send + Sync;
    type Id: Send + Sync;
    
    async fn find_by_id(&self, id: &Self::Id) -> Result<Option<Self::Entity>>;
    async fn save(&self, entity: &Self::Entity) -> Result<Self::Entity>;
}

// Implementation
impl Repository for UrlRepository {
    type Entity = Url;
    type Id = String;
    
    async fn find_by_id(&self, id: &String) -> Result<Option<Url>> {
        // implementation
    }
}
```

**کجا در پروژه:**
- `src/database/repository.rs`
- `Default`, `Clone`, `Debug` traits
- `From`, `Into` برای تبدیل

### 4. Enums و Pattern Matching

```rust
// Enum with data
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Database(#[from] sqlx::Error),
}

// Pattern matching
match self {
    Self::NotFound(_) => StatusCode::NOT_FOUND,
    Self::BadRequest(_) => StatusCode::BAD_REQUEST,
    Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
}
```

**کجا در پروژه:**
- `src/error/mod.rs` - AppError
- `src/config/mod.rs` - Environment

### 5. Generics

```rust
// Generic function
pub fn truncate<T: AsRef<str>>(text: T, max_len: usize) -> String {
    let text = text.as_ref();
    // ...
}

// Generic struct
pub struct PaginatedResult<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}
```

**کجا در پروژه:**
- `src/models/dto.rs` - ApiResponse<T>
- `src/database/repository.rs` - Repository trait

### 6. Lifetimes

```rust
// Explicit lifetime
fn get_first_word<'a>(s: &'a str) -> &'a str {
    s.split_whitespace().next().unwrap_or("")
}

// In structs
pub struct Claims {
    pub sub: String,  // owned - doesn't need lifetime
}
```

**کجا در پروژه:**
- Transaction: `Transaction<'_, Sqlite>`
- معمولا Rust خودش infer میکنه

### 7. Async/Await

```rust
// Async function
pub async fn create_url(&self, request: CreateUrlRequest) -> Result<Url> {
    // await to wait for result
    let url = self.repo.find_by_short_code(&code).await?;
    Ok(url)
}

// Spawn task
tokio::spawn(async move {
    // background work
});
```

**کجا در پروژه:**
- همه handlers
- همه service methods
- database operations

### 8. Smart Pointers

```rust
// Arc - Atomic Reference Counting
pub struct AppState {
    pub config: Arc<Config>,
    pub url_service: Arc<UrlService>,
}

// For cloning
let state = state.clone();  // only reference count increases
```

**کجا در پروژه:**
- `src/services/mod.rs` - AppState
- `src/database/mod.rs` - Database

### 9. Derive Macros

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Url {
    pub id: String,
    pub short_code: String,
}
```

| Macro | کاربرد |
|-------|--------|
| `Debug` | پرینت با `{:?}` |
| `Clone` | کپی کردن |
| `Serialize` | تبدیل به JSON |
| `Deserialize` | خوندن از JSON |
| `FromRow` | تبدیل از دیتابیس |
| `Validate` | اعتبارسنجی |
| `Default` | مقدار پیش‌فرض |

### 10. Modules و Visibility

```rust
// In lib.rs
pub mod config;      // public - accessible from outside
mod internal;        // private - internal only

// Re-export
pub use error::Result;

// In other files
use crate::config::Config;
use super::utils;
```

---

## 🎨 الگوهای طراحی (Design Patterns)

### 1. Builder Pattern

```rust
let config = ConfigBuilder::new()
    .port(8080)
    .host("0.0.0.0")
    .environment(Environment::Production)
    .build();
```

**فایل:** `src/config/mod.rs`, `src/models/url.rs`

### 2. Repository Pattern

```rust
pub trait Repository {
    async fn find_by_id(&self, id: &Self::Id) -> Result<Option<Self::Entity>>;
    async fn save(&self, entity: &Self::Entity) -> Result<Self::Entity>;
}
```

**فایل:** `src/database/repository.rs`

### 3. Newtype Pattern

```rust
pub struct Id(String);

impl Id {
    pub fn new() -> Self { Self(nanoid::nanoid!()) }
    pub fn as_str(&self) -> &str { &self.0 }
}
```

**فایل:** `src/models/mod.rs`

### 4. Dependency Injection

```rust
pub struct UrlService {
    repo: UrlRepository,      // injected
    config: Arc<Config>,      // injected
}

impl UrlService {
    pub fn new(repo: UrlRepository, config: Arc<Config>) -> Self {
        Self { repo, config }
    }
}
```

**فایل:** `src/services/`

### 5. Extension Traits

```rust
pub trait OptionExt<T> {
    fn ok_or_not_found(self, msg: impl Into<String>) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_not_found(self, msg: impl Into<String>) -> Result<T> {
        self.ok_or_else(|| AppError::NotFound(msg.into()))
    }
}

// Usage
let url = some_option.ok_or_not_found("URL not found")?;
```

**فایل:** `src/error/mod.rs`

---

## 🚀 اجرای پروژه

### پیش‌نیازها

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install sqlx-cli (optional)
cargo install sqlx-cli
```

### اجرا

```bash
# Clone
git clone <repo>
cd url-shortener

# Create data directory
mkdir -p data

# Run in development
cargo run

# Run in release mode
cargo run --release
```

### تست

```bash
# All tests
cargo test

# With output
cargo test -- --nocapture

# Specific test
cargo test test_generate_short_code

# Only unit tests
cargo test --lib
```

### بررسی کد

```bash
# Check errors without compiling
cargo check

# Linting with clippy
cargo clippy

# Format code
cargo fmt
```

---

## 📡 API Endpoints

### URL Shortening

```bash
# Create short URL
curl -X POST http://localhost:3000/api/urls \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com/long-url"}'

# Create with custom code
curl -X POST http://localhost:3000/api/urls \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "custom_code": "mylink"}'

# Redirect
curl -L http://localhost:3000/abc123

# Get URL info
curl http://localhost:3000/api/urls/abc123

# Delete URL
curl -X DELETE http://localhost:3000/api/urls/abc123
```

### Authentication

```bash
# Register
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "securepass123"}'

# Login
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "securepass123"}'

# Profile (requires token)
curl http://localhost:3000/api/me \
  -H "Authorization: Bearer <token>"
```

### Health & Stats

```bash
# Health check
curl http://localhost:3000/health

# Statistics
curl http://localhost:3000/api/stats
```

---

## 📖 منابع یادگیری

### کتاب‌ها
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Async Rust](https://rust-lang.github.io/async-book/)

### فریمورک‌ها
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [SQLx](https://github.com/launchbadge/sqlx)
- [Tokio](https://tokio.rs/)

### ابزارها
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)

---

## 🎯 تمرین‌های پیشنهادی

1. **اضافه کردن Rate Limiting واقعی** با Redis
2. **اضافه کردن Click Analytics** با جزئیات بیشتر
3. **پیاده‌سازی Custom Domain** برای هر کاربر
4. **اضافه کردن QR Code** برای هر URL
5. **پیاده‌سازی Bulk URL Creation**
6. **اضافه کردن OpenAPI/Swagger**
7. **Dockerize کردن پروژه**
8. **اضافه کردن CI/CD**

---

## 📝 License

MIT

---

ساخته شده با ❤️ برای یادگیری Rust

