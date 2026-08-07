// ============================================================================
// MODULE JSON: PHÂN TÍCH VÀ TRÍCH XUẤT CHUỖI JSON ĐƠN GIẢN CHUẨN STD-ONLY
// ============================================================================
// Triển khai bộ phân tích JSON siêu nhẹ thuần Rust std-only không phụ thuộc serde.
// Phục vụ trích xuất giá trị trường chuỗi và số nguyên cho REST và WebSocket.
// Tuân thủ 100% quy tắc từ đơn tiếng Anh cho định danh và chú thích tiếng Việt.
// ============================================================================

/// Trích xuất giá trị chuỗi &str từ chuỗi JSON text cho khóa key
pub fn str<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let pattern = format!("\"{}\"", key);
    if let Some(pos) = text.find(&pattern) {
        let rest = &text[pos + pattern.len()..];
        if let Some(colon) = rest.find(':') {
            let val = rest[colon + 1..].trim();
            if val.starts_with('"') {
                if let Some(end) = val[1..].find('"') {
                    return Some(&val[1..1 + end]);
                }
            }
        }
    }
    None
}

/// Trích xuất giá trị số nguyên usize từ chuỗi JSON text cho khóa key
pub fn num(text: &str, key: &str) -> Option<usize> {
    let pattern = format!("\"{}\"", key);
    if let Some(pos) = text.find(&pattern) {
        let rest = &text[pos + pattern.len()..];
        if let Some(colon) = rest.find(':') {
            let val = rest[colon + 1..].trim();
            let digits: String = val.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !digits.is_empty() {
                return digits.parse::<usize>().ok();
            }
        }
    }
    None
}

/// Trích xuất danh sách các chuỗi từ mảng JSON text cho khóa key
pub fn list(text: &str, key: &str) -> Vec<String> {
    let mut items = Vec::new();
    let pattern = format!("\"{}\"", key);
    if let Some(pos) = text.find(&pattern) {
        let rest = &text[pos + pattern.len()..];
        if let Some(bracket_start) = rest.find('[') {
            if let Some(bracket_end) = rest[bracket_start..].find(']') {
                let array_str = &rest[bracket_start + 1..bracket_start + bracket_end];
                for part in array_str.split(',') {
                    let s = part.trim().trim_matches('"').trim();
                    if !s.is_empty() {
                        items.push(s.to_string());
                    }
                }
            }
        }
    }
    items
}
