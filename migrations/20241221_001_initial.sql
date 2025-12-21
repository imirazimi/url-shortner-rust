-- =====================================
-- Migration اولیه - ساخت جداول
-- =====================================
-- در SQLx، migration‌ها به ترتیب اجرا میشن
-- نام فایل باید با timestamp یا شماره شروع بشه

-- جدول کاربران
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    name TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ایندکس روی email برای جستجوی سریع
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- جدول URL‌ها
CREATE TABLE IF NOT EXISTS urls (
    id TEXT PRIMARY KEY NOT NULL,
    short_code TEXT UNIQUE NOT NULL,
    original_url TEXT NOT NULL,
    title TEXT,
    clicks INTEGER NOT NULL DEFAULT 0,
    user_id TEXT,
    expires_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign Key به جدول کاربران
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- ایندکس روی short_code برای redirect سریع
CREATE INDEX IF NOT EXISTS idx_urls_short_code ON urls(short_code);

-- ایندکس روی user_id برای لیست URL‌های کاربر
CREATE INDEX IF NOT EXISTS idx_urls_user_id ON urls(user_id);

-- ایندکس روی expires_at برای cleanup
CREATE INDEX IF NOT EXISTS idx_urls_expires_at ON urls(expires_at);

-- جدول آمار کلیک‌ها (اختیاری - برای analytics پیشرفته)
CREATE TABLE IF NOT EXISTS click_events (
    id TEXT PRIMARY KEY NOT NULL,
    url_id TEXT NOT NULL,
    ip_address TEXT,
    user_agent TEXT,
    referer TEXT,
    country TEXT,
    clicked_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (url_id) REFERENCES urls(id) ON DELETE CASCADE
);

-- ایندکس برای گزارش‌گیری
CREATE INDEX IF NOT EXISTS idx_click_events_url_id ON click_events(url_id);
CREATE INDEX IF NOT EXISTS idx_click_events_clicked_at ON click_events(clicked_at);

