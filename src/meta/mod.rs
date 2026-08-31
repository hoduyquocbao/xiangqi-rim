// ============================================================================
// MÔ-ĐUN META: QUẢN LÝ THÔNG TIN PHIÊN BẢN ĐỘNG (ZERO-RECOMPILE SOTA)
// ============================================================================
// Cung cấp giao thức Dynamic Runtime Injection đọc thông tin phiên bản từ:
// 1. Biến môi trường hệ thống (APP_VERSION, APP_BUILD_STAMP).
// 2. Tệp cấu hình metadata.json (Zero Rebuild khi thay đổi phiên bản).
// 3. Fallback mặc định an toàn khi chạy offline.
// 100% tuân thủ Quy tắc Từ Đơn (Single-Word Principle) và chú thích tiếng Việt.
// ============================================================================

use std::fs::File;
use std::io::Read;

/// Struct `Meta` đại diện cho đối tượng quản trị siêu dữ liệu hệ thống.
#[derive(Debug, Clone, Copy)]
pub struct Meta;

impl Meta {
    /// Đọc thông tin phiên bản hệ thống hiện tại.
    pub fn version() -> String {
        // 1. Ưu tiên đọc từ biến môi trường runtime APP_VERSION
        if let Ok(v) = std::env::var("APP_VERSION") {
            if !v.trim().is_empty() {
                return v;
            }
        }

        // 2. Đọc từ tệp metadata.json nếu tồn tại
        if let Ok(mut file) = File::open("metadata.json") {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                if let Some(val) = Self::parse(&content, "version") {
                    return val;
                }
            }
        }

        // 3. Fallback mặc định theo hằng số
        "v11.2.0-dynamic-sota-smp".to_string()
    }

    /// Đọc dấu thời gian đóng gói hệ thống.
    pub fn stamp() -> String {
        // 1. Ưu tiên đọc từ biến môi trường runtime APP_BUILD_STAMP
        if let Ok(s) = std::env::var("APP_BUILD_STAMP") {
            if !s.trim().is_empty() {
                return s;
            }
        }

        // 2. Đọc từ tệp metadata.json nếu tồn tại
        if let Ok(mut file) = File::open("metadata.json") {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                if let Some(val) = Self::parse(&content, "stamp") {
                    return val;
                }
            }
        }

        // 3. Fallback mặc định
        "2026-08-25 22:56:00 ICT".to_string()
    }

    /// Đọc ghi chú phát hành của phiên bản hiện tại.
    pub fn notes() -> String {
        if let Ok(mut file) = File::open("metadata.json") {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                if let Some(val) = Self::parse(&content, "notes") {
                    return val;
                }
            }
        }
        "Dynamic Runtime Injection & Multi-Thread FFI SOTA".to_string()
    }

    /// Hàm phân tích chuỗi JSON đơn giản (Zero Crate JSON Parser).
    fn parse(json: &str, key: &str) -> Option<String> {
        let pattern = format!("\"{}\"", key);
        let pos = json.find(&pattern)?;
        let rest = &json[pos + pattern.len()..];
        let colon = rest.find(':')?;
        let after = rest[colon + 1..].trim_start();
        if !after.starts_with('"') {
            return None;
        }
        let quote_end = after[1..].find('"')?;
        Some(after[1..=quote_end].to_string())
    }

    /// Xuất chuỗi định dạng thông tin tổng hợp.
    pub fn info() -> String {
        format!("{} ({})", Self::version(), Self::stamp())
    }
}
