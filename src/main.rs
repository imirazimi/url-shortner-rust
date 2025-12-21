//! # URL Shortener - نقطه ورود برنامه
//!
//! این فایل نقطه شروع اجرای برنامه است.
//! در Rust، `main.rs` برای باینری‌ها و `lib.rs` برای کتابخانه‌ها استفاده میشه.
//!
//! ## مفاهیم Rust در این فایل:
//! - `#![doc]`: مستندسازی در سطح ماژول
//! - `use`: وارد کردن آیتم‌ها از ماژول‌های دیگه
//! - `async fn main()`: تابع اصلی غیرهمزمان با tokio
//! - `Result<T, E>`: مدیریت خطا
//! - `?` operator: انتشار خطا به بالا

use std::net::SocketAddr;

use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// وارد کردن ماژول‌ها از کتابخانه‌مون
use url_shortener::{
    api::create_router,
    config::Config,
    database::Database,
    error::Result,
};

/// نقطه ورود اصلی برنامه
///
/// # مفاهیم مهم:
/// - `#[tokio::main]`: این macro تابع async رو به یک runtime تبدیل میکنه
/// - `async fn`: تابع غیرهمزمان که میتونه await داشته باشه
/// - `Result<()>`: برگردوندن Result بدون مقدار موفقیت (unit type)
///
/// # Errors
/// خطا برمیگردونه اگه:
/// - تنظیمات لود نشن
/// - دیتابیس متصل نشه
/// - سرور استارت نشه
#[tokio::main]
async fn main() -> Result<()> {
    // لود کردن متغیرهای محیطی از فایل .env
    // در Rust خطاها رو باید handle کنیم، اینجا اگه فایل نباشه اوکیه
    dotenvy::dotenv().ok();

    // راه‌اندازی سیستم لاگینگ
    // این یه نمونه از Builder Pattern هست
    init_tracing();

    info!("🚀 Starting URL Shortener Service...");

    // لود کردن تنظیمات
    // `?` یعنی اگه خطا بود، همینجا return کن
    let config = Config::from_env()?;
    info!("✅ Configuration loaded successfully");

    // اتصال به دیتابیس
    // `Arc<T>` برای share کردن ownership بین thread‌ها
    let database = Database::connect(&config.database_url).await?;
    info!("✅ Database connected successfully");

    // اجرای migration‌ها
    database.migrate().await?;
    info!("✅ Database migrations applied");

    // ساخت router با تمام route‌ها و middleware‌ها
    // این یه نمونه از Dependency Injection هست
    let app = create_router(database, config.clone());

    // آدرس سرور
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("🌐 Server listening on http://{}", addr);

    // ساخت listener و اجرای سرور
    let listener = TcpListener::bind(addr).await?;

    // اجرای سرور - این بلاک تا ابد اجرا میشه
    axum::serve(listener, app)
        .await
        .map_err(|e| url_shortener::error::AppError::Server(e.to_string()))?;

    Ok(())
}

/// راه‌اندازی سیستم tracing برای لاگینگ
///
/// # مفاهیم:
/// - Structured Logging: لاگ‌ها به صورت ساختاریافته ذخیره میشن
/// - Layers: لایه‌های مختلف برای فرمت و فیلتر کردن
/// - EnvFilter: فیلتر کردن لاگ‌ها بر اساس متغیر محیطی
fn init_tracing() {
    // EnvFilter از متغیر RUST_LOG میخونه
    // اگه نبود، default استفاده میکنه
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("url_shortener=debug,tower_http=debug"));

    // ترکیب لایه‌ها با هم
    // این یه نمونه از Decorator Pattern هست
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)           // نمایش نام ماژول
                .with_thread_ids(true)       // نمایش ID ترد
                .with_file(true)             // نمایش نام فایل
                .with_line_number(true)      // نمایش شماره خط
                .with_level(true)            // نمایش سطح لاگ
                .pretty(),                   // فرمت زیبا
        )
        .init();
}

