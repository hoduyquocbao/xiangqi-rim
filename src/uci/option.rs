// ============================================================================
// MODULE OPTION: CẤU HÌNH TÙY CHỌN CHUẨN GIAO THỨC UCI (ENGINE OPTIONS DEFINITION)
// ============================================================================
// `option.rs` chịu trách nhiệm định nghĩa và quản lý các loại tùy chọn cài đặt của Engine:
// - `Kind`: Phân loại tùy chọn (`Spin`, `Button`, `Combo`, `Check`, `String`).
// - `Option`: Struct bọc tên tùy chọn (`name`), giá trị mặc định (`def`), giá trị hiện tại (`val`),
//   khoảng giá trị số nguyên $[min, max]$, và danh sách các lựa chọn (`vars`).
// ============================================================================

/// Enum `Kind` phân loại các kiểu tùy chọn giao diện UCI hỗ trợ.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Tùy chọn kiểu Spin (Số nguyên nằm trong khoảng [min, max])
    Spin,
    /// Tùy chọn kiểu Button (Nút bấm thực thi lệnh đơn)
    Button,
    /// Tùy chọn kiểu Combo (Danh sách chọn từ các giá trị có sẵn)
    Combo,
    /// Tùy chọn kiểu Check (Cờ bật/tắt true/false)
    Check,
    /// Tùy chọn kiểu String (Chuỗi văn bản tự do)
    String,
}

/// Struct `Option` đại diện cho một thông số tùy chọn UCI của Engine.
#[derive(Clone, Debug)]
pub struct Option {
    /// Tên của tùy chọn (e.g. `"Hash"`, `"Threads"`)
    pub name: String,
    /// Loại tùy chọn (Spin, Button, Combo, Check, String)
    pub kind: Kind,
    /// Giá trị hiện tại dưới dạng chuỗi
    pub val: String,
    /// Giá trị mặc định dưới dạng chuỗi
    pub def: String,
    /// Giá trị nhỏ nhất (dành cho kiểu Spin)
    pub min: i32,
    /// Giá trị lớn nhất (dành cho kiểu Spin)
    pub max: i32,
    /// Danh sách các biến thể chuỗi hợp lệ (dành cho kiểu Combo)
    pub vars: Vec<String>,
}

impl Option {
    /// Khởi tạo tùy chọn kiểu Spin (số nguyên nằm trong khoảng $[min, max]$).
    pub fn spin(name: &str, def: i32, min: i32, max: i32) -> Self {
        Self {
            name: name.to_string(),
            kind: Kind::Spin,
            val: def.to_string(),
            def: def.to_string(),
            min,
            max,
            vars: Vec::new(),
        }
    }

    /// Khởi tạo tùy chọn kiểu Button (nút bấm thực thi lệnh).
    pub fn button(name: &str) -> Self {
        Self {
            name: name.to_string(),
            kind: Kind::Button,
            val: String::new(),
            def: String::new(),
            min: 0,
            max: 0,
            vars: Vec::new(),
        }
    }

    /// Khởi tạo tùy chọn kiểu Combo (danh sách lựa chọn từ các chuỗi `vars`).
    pub fn combo(name: &str, def: &str, vars: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            kind: Kind::Combo,
            val: def.to_string(),
            def: def.to_string(),
            min: 0,
            max: 0,
            vars: vars.iter().map(|item| item.to_string()).collect(),
        }
    }
}

