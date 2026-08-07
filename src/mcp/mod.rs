// ============================================================================
// MODULE MCP: GIAO THỨC MODEL CONTEXT PROTOCOL (JSON-RPC 2.0 OVER STDIN/STDOUT)
// ============================================================================
// Module triển khai giao thức MCP Server thuần Rust std không phụ thuộc thư viện ngoài.
// Tuân thủ 100% nguyên tắc Clean Room Design 0đ và Định danh Từ Đơn (Single-Word).
// Hỗ trợ bộ phân tích & dựng JSON (Value, Kind, Parser, Builder) và 5 MCP Tools.
// ============================================================================

// Nhập các kiểu dữ liệu từ thư viện tiêu chuẩn std::fmt
use std::fmt;
// Nhập bộ đọc ghi văn bản từ std::io
use std::io::{self, BufRead, Write};

// Nhập module board từ xiangrust
use crate::board;
// Nhập module eval từ xiangrust
use crate::eval;
// Nhập module movegen từ xiangrust
use crate::movegen;
// Nhập module search từ xiangrust
use crate::search;
// Nhập module thread từ xiangrust
use crate::thread;
// Nhập module uci từ xiangrust
use crate::uci;

// ============================================================================
// ENUM KIND: PHÂN LOẠI CÁC KIỂU DỮ LIỆU JSON
// ============================================================================
/// Enum phân loại 6 kiểu dữ liệu chuẩn của cú pháp JSON
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Kiểu giá trị rỗng (null)
    Null,
    /// Kiểu giá trị luận lý (boolean: true / false)
    Bool,
    /// Kiểu giá trị số (number: số nguyên hoặc số thực)
    Number,
    /// Kiểu chuỗi văn bản (string)
    String,
    /// Kiểu mảng phần tử (array: danh sách các giá trị)
    Array,
    /// Kiểu đối tượng (object: tập hợp các cặp khóa - giá trị)
    Object,
}

// ============================================================================
// ENUM VALUE: BIỂU DIỄN NÚT DỮ LIỆU JSON TRONG BỘ NHỚ
// ============================================================================
/// Enum biểu diễn trực tiếp cây dữ liệu JSON không cần serde
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Giá trị rỗng
    Null,
    /// Giá trị luận lý true/false
    Bool(bool),
    /// Giá trị số thực 64-bit
    Number(f64),
    /// Giá trị chuỗi sỡ hữu String
    String(String),
    /// Mảng các nút Value
    Array(Vec<Value>),
    /// Đối tượng chứa các cặp (String, Value)
    Object(Vec<(String, Value)>),
}

impl Value {
    /// Khởi tạo nút rỗng Null
    pub fn null() -> Self {
        Value::Null
    }

    /// Khởi tạo nút luận lý Bool
    pub fn bool(flag: bool) -> Self {
        Value::Bool(flag)
    }

    /// Khởi tạo nút số Number từ float 64-bit
    pub fn number(val: f64) -> Self {
        Value::Number(val)
    }

    /// Khởi tạo nút chuỗi String từ con trỏ chuỗi
    pub fn string(text: &str) -> Self {
        Value::String(text.to_string())
    }

    /// Khởi tạo nút mảng Array rỗng
    pub fn array() -> Self {
        Value::Array(Vec::new())
    }

    /// Khởi tạo nút đối tượng Object rỗng
    pub fn object() -> Self {
        Value::Object(Vec::new())
    }

    /// Trả về loại kiểu dữ liệu Enum Kind của nút hiện tại
    pub fn kind(&self) -> Kind {
        match self {
            Value::Null => Kind::Null,
            Value::Bool(_) => Kind::Bool,
            Value::Number(_) => Kind::Number,
            Value::String(_) => Kind::String,
            Value::Array(_) => Kind::Array,
            Value::Object(_) => Kind::Object,
        }
    }

    /// Thêm một phần tử Value vào cuối mảng Array
    pub fn push(&mut self, item: Value) {
        if let Value::Array(list) = self {
            list.push(item);
        }
    }

    /// Chèn một cặp (key, val) vào đối tượng Object
    pub fn insert(&mut self, key: &str, val: Value) {
        if let Value::Object(pairs) = self {
            for pair in pairs.iter_mut() {
                if pair.0 == key {
                    pair.1 = val;
                    return;
                }
            }
            pairs.push((key.to_string(), val));
        }
    }

    /// Truy xuất tham chiếu đến giá trị thuộc khóa key trong Object
    pub fn get(&self, key: &str) -> Option<&Value> {
        if let Value::Object(pairs) = self {
            for pair in pairs {
                if pair.0 == key {
                    return Some(&pair.1);
                }
            }
        }
        None
    }

    /// Chuyển đổi giá trị sang chuỗi tham chiếu &str nếu thuộc kiểu String
    pub fn text(&self) -> Option<&str> {
        if let Value::String(text) = self {
            Some(text.as_str())
        } else {
            None
        }
    }

    /// Chuyển đổi giá trị sang số thực f64 nếu thuộc kiểu Number
    pub fn num(&self) -> Option<f64> {
        if let Value::Number(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    /// Chuyển đổi giá trị sang số nguyên i64 nếu thuộc kiểu Number
    pub fn integer(&self) -> Option<i64> {
        if let Value::Number(val) = self {
            Some(*val as i64)
        } else {
            None
        }
    }

    /// Chuyển đổi giá trị sang boolean nếu thuộc kiểu Bool
    pub fn boolean(&self) -> Option<bool> {
        if let Value::Bool(flag) = self {
            Some(*flag)
        } else {
            None
        }
    }

    /// Mã hóa đối tượng Value thành chuỗi văn bản JSON chuẩn
    pub fn encode(&self) -> String {
        Builder::build(self)
    }
}

// Trợ giúp hiển thị chuỗi JSON
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encode())
    }
}

// ============================================================================
// STRUCT BUILDER: XÂY DỰNG CHUỖI VĂN BẢN JSON CHUẨN
// ============================================================================
/// Struct hỗ trợ chuyển đổi cây dữ liệu Value thành chuỗi văn bản JSON
pub struct Builder;

impl Builder {
    /// Chuyển đổi một Value thành chuỗi JSON
    pub fn build(val: &Value) -> String {
        match val {
            Value::Null => "null".to_string(),
            Value::Bool(flag) => {
                if *flag {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Value::Number(num) => {
                if num.fract() == 0.0 && num.abs() < 1e15 {
                    (*num as i64).to_string()
                } else {
                    format!("{}", num)
                }
            }
            Value::String(text) => format!("\"{}\"", Self::escape(text)),
            Value::Array(items) => {
                let mut out = String::from("[");
                for (idx, item) in items.iter().enumerate() {
                    if idx > 0 {
                        out.push(',');
                    }
                    out.push_str(&Self::build(item));
                }
                out.push(']');
                out
            }
            Value::Object(pairs) => {
                let mut out = String::from("{");
                for (idx, (key, item)) in pairs.iter().enumerate() {
                    if idx > 0 {
                        out.push(',');
                    }
                    out.push('"');
                    out.push_str(&Self::escape(key));
                    out.push_str("\":");
                    out.push_str(&Self::build(item));
                }
                out.push('}');
                out
            }
        }
    }

    /// Mã hóa các ký tự đặc biệt trong chuỗi thành ký tự escape hợp lệ
    pub fn escape(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        for ch in text.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                _ => out.push(ch),
            }
        }
        out
    }
}

// ============================================================================
// STRUCT PARSER: BỘ PHÂN TÍCH CÚ PHÁP JSON ĐỆ QUY SÂU THUẦN RUST STD
// ============================================================================
/// Bộ phân tích cú pháp chuỗi JSON đệ quy sâu thuần Rust std (0đ crate)
pub struct Parser<'a> {
    /// Lát cắt văn bản nguồn JSON
    slice: &'a str,
    /// Con trỏ vị trí ký tự hiện tại
    index: usize,
}

impl<'a> Parser<'a> {
    /// Khởi tạo một Parser mới từ chuỗi văn bản nguồn
    pub fn new(slice: &'a str) -> Self {
        Parser { slice, index: 0 }
    }

    /// Phân tích toàn bộ chuỗi văn bản thành đối tượng Value
    pub fn parse(slice: &'a str) -> Result<Value, String> {
        let mut parser = Parser::new(slice);
        parser.space();
        let val = parser.value()?;
        parser.space();
        Ok(val)
    }

    /// Xem trước ký tự tiếp theo mà không di chuyển con trỏ index
    fn peek(&self) -> Option<char> {
        self.slice[self.index..].chars().next()
    }

    /// Đọc ký tự tiếp theo và di chuyển con trỏ index
    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.index += ch.len_utf8();
        Some(ch)
    }

    /// Bỏ qua toàn bộ ký tự khoảng trắng (space, tab, newline, return)
    fn space(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
                self.next();
            } else {
                break;
            }
        }
    }

    /// Phân tích một nút Value đệ quy từ vị trí hiện tại
    fn value(&mut self) -> Result<Value, String> {
        self.space();
        let ch = self.peek().ok_or_else(|| "Nguồn JSON rỗng".to_string())?;
        match ch {
            '{' => self.object(),
            '[' => self.array(),
            '"' => self.string().map(Value::String),
            't' | 'f' => self.boolean().map(Value::Bool),
            'n' => self.null().map(|_| Value::Null),
            '-' | '0'..='9' => self.number().map(Value::Number),
            _ => Err(format!("Ký tự không hợp lệ tại vị trí {}: {}", self.index, ch)),
        }
    }

    /// Phân tích chuỗi văn bản JSON (String)
    fn string(&mut self) -> Result<String, String> {
        if self.next() != Some('"') {
            return Err("Kỳ vọng ký tự mở kép '\"'".to_string());
        }
        let mut out = String::new();
        while let Some(ch) = self.next() {
            if ch == '"' {
                return Ok(out);
            }
            if ch == '\\' {
                let esc = self.next().ok_or_else(|| "Chuỗi kết thúc bất ngờ sau '\\'".to_string())?;
                match esc {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    '/' => out.push('/'),
                    'b' => out.push('\x08'),
                    'f' => out.push('\x0C'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'u' => {
                        let mut code = 0u32;
                        for _ in 0..4 {
                            let hex = self.next().ok_or_else(|| "Chuỗi unicode không đủ 4 ký tự hex".to_string())?;
                            let digit = hex.to_digit(16).ok_or_else(|| "Ký tự hex unicode không hợp lệ".to_string())?;
                            code = (code << 4) | digit;
                        }
                        if let Some(unicode) = char::from_u32(code) {
                            out.push(unicode);
                        }
                    }
                    _ => out.push(esc),
                }
            } else {
                out.push(ch);
            }
        }
        Err("Chuỗi chưa được đóng bằng '\"'".to_string())
    }

    /// Phân tích giá trị số (Number)
    fn number(&mut self) -> Result<f64, String> {
        let start = self.index;
        if self.peek() == Some('-') {
            self.next();
        }
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.next();
            } else {
                break;
            }
        }
        if self.peek() == Some('.') {
            self.next();
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    self.next();
                } else {
                    break;
                }
            }
        }
        if self.peek() == Some('e') || self.peek() == Some('E') {
            self.next();
            if self.peek() == Some('+') || self.peek() == Some('-') {
                self.next();
            }
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    self.next();
                } else {
                    break;
                }
            }
        }
        let text = &self.slice[start..self.index];
        text.parse::<f64>().map_err(|_| format!("Không thể parse số: {}", text))
    }

    /// Phân tích giá trị mảng (Array)
    fn array(&mut self) -> Result<Value, String> {
        if self.next() != Some('[') {
            return Err("Kỳ vọng '['".to_string());
        }
        let mut list = Vec::new();
        self.space();
        if self.peek() == Some(']') {
            self.next();
            return Ok(Value::Array(list));
        }
        loop {
            let elem = self.value()?;
            list.push(elem);
            self.space();
            if self.peek() == Some(',') {
                self.next();
                self.space();
            } else if self.peek() == Some(']') {
                self.next();
                break;
            } else {
                return Err("Kỳ vọng ',' hoặc ']' trong mảng".to_string());
            }
        }
        Ok(Value::Array(list))
    }

    /// Phân tích đối tượng (Object)
    fn object(&mut self) -> Result<Value, String> {
        if self.next() != Some('{') {
            return Err("Kỳ vọng '{'".to_string());
        }
        let mut pairs = Vec::new();
        self.space();
        if self.peek() == Some('}') {
            self.next();
            return Ok(Value::Object(pairs));
        }
        loop {
            self.space();
            if self.peek() != Some('"') {
                return Err("Kỳ vọng chuỗi khóa '\"'".to_string());
            }
            let key = self.string()?;
            self.space();
            if self.next() != Some(':') {
                return Err("Kỳ vọng dấu ':' sau khóa".to_string());
            }
            let val = self.value()?;
            pairs.push((key, val));
            self.space();
            if self.peek() == Some(',') {
                self.next();
                self.space();
            } else if self.peek() == Some('}') {
                self.next();
                break;
            } else {
                return Err("Kỳ vọng ',' hoặc '}' trong đối tượng".to_string());
            }
        }
        Ok(Value::Object(pairs))
    }

    /// Phân tích giá trị luận lý boolean (true / false)
    fn boolean(&mut self) -> Result<bool, String> {
        if self.slice[self.index..].starts_with("true") {
            self.index += 4;
            Ok(true)
        } else if self.slice[self.index..].starts_with("false") {
            self.index += 5;
            Ok(false)
        } else {
            Err("Kỳ vọng giá trị luận lý true hoặc false".to_string())
        }
    }

    /// Phân tích giá trị rỗng null
    fn null(&mut self) -> Result<(), String> {
        if self.slice[self.index..].starts_with("null") {
            self.index += 4;
            Ok(())
        } else {
            Err("Kỳ vọng giá trị rỗng null".to_string())
        }
    }
}

// ============================================================================
// STRUCTS JSON-RPC 2.0 PROTOCOL: REQUEST, RESPONSE, ERROR
// ============================================================================
/// Cấu trúc yêu cầu JSON-RPC 2.0 Request
#[derive(Clone, Debug)]
pub struct Request {
    /// Định danh yêu cầu (ID dạng số, chuỗi hoặc null)
    pub id: Value,
    /// Tên phương thức RPC (ví dụ: "tools/list", "tools/call", "initialize")
    pub method: String,
    /// Tham số đầu vào dạng đối tượng hoặc mảng Value
    pub params: Value,
}

impl Request {
    /// Phân tích một yêu cầu từ chuỗi văn bản JSON
    pub fn parse(text: &str) -> Result<Self, String> {
        let root = Parser::parse(text)?;
        let id = root.get("id").cloned().unwrap_or(Value::Null);
        let method = root
            .get("method")
            .and_then(|m| m.text())
            .ok_or_else(|| "Thiếu trường 'method'".to_string())?
            .to_string();
        let params = root.get("params").cloned().unwrap_or(Value::object());
        Ok(Request { id, method, params })
    }
}

/// Cấu trúc phản hồi JSON-RPC 2.0 Response
#[derive(Clone, Debug)]
pub struct Response {
    /// Khóa id tương ứng với yêu cầu đầu vào
    pub id: Value,
    /// Kết quả trả về nếu thành công
    pub result: Value,
    /// Lỗi phát sinh nếu thất bại
    pub error: Option<Error>,
}

impl Response {
    /// Khởi tạo phản hồi kết quả thành công
    pub fn success(id: Value, result: Value) -> Self {
        Response {
            id,
            result,
            error: None,
        }
    }

    /// Khởi tạo phản hồi báo lỗi
    pub fn fail(id: Value, code: i32, message: &str) -> Self {
        Response {
            id,
            result: Value::Null,
            error: Some(Error {
                code,
                message: message.to_string(),
                data: Value::Null,
            }),
        }
    }

    /// Chuyển đổi cấu trúc Response thành đối tượng Value JSON
    pub fn value(&self) -> Value {
        let mut obj = Value::object();
        obj.insert("jsonrpc", Value::string("2.0"));
        obj.insert("id", self.id.clone());
        if let Some(err) = &self.error {
            obj.insert("error", err.value());
        } else {
            obj.insert("result", self.result.clone());
        }
        obj
    }

    /// Mã hóa phản hồi thành chuỗi văn bản JSON
    pub fn encode(&self) -> String {
        self.value().encode()
    }
}

/// Cấu trúc lỗi JSON-RPC 2.0 Error
#[derive(Clone, Debug)]
pub struct Error {
    /// Mã lỗi số nguyên (ví dụ: -32601)
    pub code: i32,
    /// Thông điệp mô tả lỗi bằng văn bản
    pub message: String,
    /// Dữ liệu bổ sung đi kèm lỗi
    pub data: Value,
}

impl Error {
    /// Chuyển đổi cấu trúc Error thành nút đối tượng Value
    pub fn value(&self) -> Value {
        let mut obj = Value::object();
        obj.insert("code", Value::number(self.code as f64));
        obj.insert("message", Value::string(&self.message));
        if self.data != Value::Null {
            obj.insert("data", self.data.clone());
        }
        obj
    }
}

// ============================================================================
// STRUCT TOOL: KHAI BÁO THÔNG TIN CHI TIẾT CỦA MỘT MCP TOOL
// ============================================================================
/// Định nghĩa cấu trúc thông tin của một MCP Tool chuẩn
#[derive(Clone, Debug)]
pub struct Tool {
    /// Tên công cụ (ví dụ: "get_best_move")
    pub name: String,
    /// Mô tả chức năng bằng văn bản
    pub description: String,
    /// Đòn schema tham số đầu vào JSON Schema
    pub schema: Value,
}

impl Tool {
    /// Chuyển đổi định nghĩa Tool thành đối tượng JSON Value
    pub fn value(&self) -> Value {
        let mut obj = Value::object();
        obj.insert("name", Value::string(&self.name));
        obj.insert("description", Value::string(&self.description));
        obj.insert("inputSchema", self.schema.clone());
        obj
    }
}

// ============================================================================
// STRUCT SERVER: QUẢN LÝ MCP SERVER & ĐIỀU PHỐI CÁC MCP TOOLS
// ============================================================================
/// Máy chủ MCP Server giao tiếp JSON-RPC 2.0 qua STDIN/STDOUT
pub struct Server {
    /// Vị trí bàn cờ hiện tại của Engine
    pos: board::Position,
    /// Bộ đánh giá thế cờ NNUE & HCE
    eval: eval::Eval,
}

impl Server {
    /// Khởi tạo một đối tượng máy chủ MCP Server mới
    pub fn new() -> Self {
        Server {
            pos: board::fen::Parser::parse(board::fen::Parser::DEFAULT),
            eval: eval::Eval::new(),
        }
    }

    /// Truy xuất vị trí bàn cờ hiện tại của Server
    pub fn position(&self) -> &board::Position {
        &self.pos
    }

    /// Lắng nghe vòng lặp STDIN và phản hồi JSON-RPC 2.0 ra STDOUT
    pub fn run(&mut self) {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let mut line = String::new();

        eprintln!("[XiangRust MCP Server] Đã khởi tạo thành công. Đang lắng nghe STDIN...");

        while let Ok(bytes) = reader.read_line(&mut line) {
            if bytes == 0 {
                break;
            }
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                if let Ok(req) = Request::parse(trimmed) {
                    if let Some(resp) = self.dispatch(&req) {
                        println!("{}", resp.encode());
                        let _ = io::stdout().flush();
                    }
                } else {
                    let fail = Response::fail(Value::Null, -32700, "Parse error: Cú pháp JSON không hợp lệ");
                    println!("{}", fail.encode());
                    let _ = io::stdout().flush();
                }
            }
            line.clear();
        }
    }

    /// Điều phối xử lý các phương thức JSON-RPC 2.0 từ Request
    pub fn dispatch(&mut self, req: &Request) -> Option<Response> {
        let method = req.method.as_str();
        match method {
            "initialize" => Some(self.initialize(req.id.clone())),
            "notifications/initialized" => None,
            "tools/list" => Some(self.list(req.id.clone())),
            "tools/call" => {
                let name = req.params.get("name").and_then(|n| n.text()).unwrap_or("");
                let args = req.params.get("arguments").cloned().unwrap_or(Value::object());
                Some(self.call(req.id.clone(), name, &args))
            }
            "get_best_move" => {
                let res = self.best(&req.params);
                Some(Response::success(req.id.clone(), res))
            }
            "evaluate_position" => {
                let res = self.evaluate(&req.params);
                Some(Response::success(req.id.clone(), res))
            }
            "perft_test" => {
                let res = self.perft(&req.params);
                Some(Response::success(req.id.clone(), res))
            }
            "parse_fen" => {
                let res = self.fen(&req.params);
                Some(Response::success(req.id.clone(), res))
            }
            "get_engine_info" => {
                let res = self.info();
                Some(Response::success(req.id.clone(), res))
            }
            _ => Some(Response::fail(
                req.id.clone(),
                -32601,
                &format!("Method not found: Phương thức '{}' không tồn tại", method),
            )),
        }
    }

    /// Phản hồi cho câu lệnh "initialize"
    pub fn initialize(&self, id: Value) -> Response {
        let mut result = Value::object();
        result.insert("protocolVersion", Value::string("2024-11-05"));

        let mut caps = Value::object();
        caps.insert("tools", Value::object());
        result.insert("capabilities", caps);

        let mut info = Value::object();
        info.insert("name", Value::string("XiangRust"));
        info.insert("version", Value::string("0.1.0"));
        result.insert("serverInfo", info);

        Response::success(id, result)
    }

    /// Phản hồi cho câu lệnh "tools/list"
    pub fn list(&self, id: Value) -> Response {
        let mut tools = Value::array();

        // Tool 1: get_best_move
        let mut tool1 = Value::object();
        tool1.insert("name", Value::string("get_best_move"));
        tool1.insert("description", Value::string("Tìm nước đi tối ưu nhất cho vị trí FEN bằng bộ tìm kiếm PVS Lazy SMP"));
        let mut schema1 = Value::object();
        schema1.insert("type", Value::string("object"));
        let mut props1 = Value::object();
        props1.insert("fen", Value::string("Chuỗi FEN vị trí bàn cờ"));
        props1.insert("depth", Value::string("Độ sâu tìm kiếm (mặc định 8)"));
        props1.insert("movetime", Value::string("Thời gian giới hạn tìm kiếm (ms)"));
        props1.insert("threads", Value::string("Số luồng tính toán song song"));
        props1.insert("hash", Value::string("Dung lượng Transposition Table MB"));
        schema1.insert("properties", props1);
        tool1.insert("inputSchema", schema1);
        tools.push(tool1);

        // Tool 2: evaluate_position
        let mut tool2 = Value::object();
        tool2.insert("name", Value::string("evaluate_position"));
        tool2.insert("description", Value::string("Đánh giá điểm số thế cờ bằng mạng nơ-ron NNUE và bộ luật HCE"));
        let mut schema2 = Value::object();
        schema2.insert("type", Value::string("object"));
        let mut props2 = Value::object();
        props2.insert("fen", Value::string("Chuỗi FEN bàn cờ"));
        props2.insert("mode", Value::string("Chế độ đánh giá: auto, nnue, hce"));
        schema2.insert("properties", props2);
        tool2.insert("inputSchema", schema2);
        tools.push(tool2);

        // Tool 3: perft_test
        let mut tool3 = Value::object();
        tool3.insert("name", Value::string("perft_test"));
        tool3.insert("description", Value::string("Chạy thuật toán kiểm thử sinh nước đi Perft đếm số nút lá"));
        let mut schema3 = Value::object();
        schema3.insert("type", Value::string("object"));
        let mut props3 = Value::object();
        props3.insert("fen", Value::string("Chuỗi FEN vị trí bàn cờ"));
        props3.insert("depth", Value::string("Độ sâu Perft (mặc định 3)"));
        schema3.insert("properties", props3);
        tool3.insert("inputSchema", schema3);
        tools.push(tool3);

        // Tool 4: parse_fen
        let mut tool4 = Value::object();
        tool4.insert("name", Value::string("parse_fen"));
        tool4.insert("description", Value::string("Phân tích FEN, kiểm tra tính hợp lệ, Zobrist Hash và mảng ma trận bàn cờ"));
        let mut schema4 = Value::object();
        schema4.insert("type", Value::string("object"));
        let mut props4 = Value::object();
        props4.insert("fen", Value::string("Chuỗi FEN bàn cờ"));
        schema4.insert("properties", props4);
        tool4.insert("inputSchema", schema4);
        tools.push(tool4);

        // Tool 5: get_engine_info
        let mut tool5 = Value::object();
        tool5.insert("name", Value::string("get_engine_info"));
        tool5.insert("description", Value::string("Lấy thông tin định danh, phiên bản, tác giả và danh sách MCP Tools hỗ trợ"));
        let mut schema5 = Value::object();
        schema5.insert("type", Value::string("object"));
        schema5.insert("properties", Value::object());
        tool5.insert("inputSchema", schema5);
        tools.push(tool5);

        let mut res = Value::object();
        res.insert("tools", tools);
        Response::success(id, res)
    }

    /// Phản hồi chuẩn cho câu lệnh "tools/call"
    pub fn call(&mut self, id: Value, name: &str, args: &Value) -> Response {
        let content = match name {
            "get_best_move" => self.best(args),
            "evaluate_position" => self.evaluate(args),
            "perft_test" => self.perft(args),
            "parse_fen" => self.fen(args),
            "get_engine_info" => self.info(),
            _ => {
                return Response::fail(
                    id,
                    -32601,
                    &format!("Tool not found: MCP Tool '{}' không tồn tại", name),
                );
            }
        };

        let mut item = Value::object();
        item.insert("type", Value::string("text"));
        item.insert("text", Value::string(&content.encode()));

        let mut array = Value::array();
        array.push(item);

        let mut res = Value::object();
        res.insert("content", array);
        Response::success(id, res)
    }

    /// Xử lý MCP Tool 1: get_best_move
    pub fn best(&mut self, args: &Value) -> Value {
        let text = args
            .get("fen")
            .and_then(|f| f.text())
            .unwrap_or(board::fen::Parser::DEFAULT);
        let depth = args
            .get("depth")
            .and_then(|d| d.integer())
            .unwrap_or(8) as u8;
        let movetime = args
            .get("movetime")
            .and_then(|t| t.integer())
            .unwrap_or(0) as u64;
        let threads = args
            .get("threads")
            .and_then(|t| t.integer())
            .unwrap_or(1) as usize;
        let hash = args
            .get("hash")
            .and_then(|h| h.integer())
            .unwrap_or(64) as usize;

        let pos = board::fen::Parser::parse(text);
        let mut limits = search::Limits::new();
        limits.depth = depth;
        if movetime > 0 {
            limits.exact = movetime;
        }

        let pool = thread::Pool::new(threads, hash);
        let res = pool.go(&pos, &limits);

        let best = uci::Format::encode(res.best);
        let ponder = if res.ponder.valid() {
            uci::Format::encode(res.ponder)
        } else {
            String::new()
        };

        let mut pvs = Value::array();
        for i in 0..res.pv.len() {
            let mv = res.pv.get(i);
            pvs.push(Value::string(&uci::Format::encode(mv)));
        }

        let span = res.time;
        let nps = if span > 0 {
            (res.nodes * 1000) / span
        } else {
            res.nodes
        };

        let mut out = Value::object();
        out.insert("best_move", Value::string(&best));
        out.insert("ponder_move", Value::string(&ponder));
        out.insert("score", Value::number(res.score as f64));
        out.insert("depth", Value::number(res.depth as f64));
        out.insert("nodes", Value::number(res.nodes as f64));
        out.insert("time_ms", Value::number(span as f64));
        out.insert("nps", Value::number(nps as f64));
        out.insert("pv", pvs);

        out
    }

    /// Xử lý MCP Tool 2: evaluate_position
    pub fn evaluate(&mut self, args: &Value) -> Value {
        let text = args
            .get("fen")
            .and_then(|f| f.text())
            .unwrap_or(board::fen::Parser::DEFAULT);
        let mode = args
            .get("mode")
            .and_then(|m| m.text())
            .unwrap_or("auto");

        let pos = board::fen::Parser::parse(text);
        self.eval.reset(&pos);

        let score = self.eval.score(&pos);
        let raw = self.eval.hce.evaluate(&pos);

        let side = if pos.side == 0 { "red" } else { "black" };

        let mut red = Value::object();
        red.insert("king", Value::number(pos.counts[0] as f64));
        red.insert("advisor", Value::number(pos.counts[1] as f64));
        red.insert("elephant", Value::number(pos.counts[2] as f64));
        red.insert("knight", Value::number(pos.counts[3] as f64));
        red.insert("rook", Value::number(pos.counts[4] as f64));
        red.insert("cannon", Value::number(pos.counts[5] as f64));
        red.insert("pawn", Value::number(pos.counts[6] as f64));

        let mut black = Value::object();
        black.insert("king", Value::number(pos.counts[7] as f64));
        black.insert("advisor", Value::number(pos.counts[8] as f64));
        black.insert("elephant", Value::number(pos.counts[9] as f64));
        black.insert("knight", Value::number(pos.counts[10] as f64));
        black.insert("rook", Value::number(pos.counts[11] as f64));
        black.insert("cannon", Value::number(pos.counts[12] as f64));
        black.insert("pawn", Value::number(pos.counts[13] as f64));

        let mut counts = Value::object();
        counts.insert("red", red);
        counts.insert("black", black);

        let mut out = Value::object();
        out.insert("score", Value::number(score as f64));
        out.insert("eval_mode", Value::string(mode));
        out.insert("hce_score", Value::number(raw as f64));
        out.insert("side_to_move", Value::string(side));
        out.insert("piece_counts", counts);

        out
    }

    /// Xử lý MCP Tool 3: perft_test
    pub fn perft(&mut self, args: &Value) -> Value {
        let text = args
            .get("fen")
            .and_then(|f| f.text())
            .unwrap_or(board::fen::Parser::DEFAULT);
        let depth = args
            .get("depth")
            .and_then(|d| d.integer())
            .unwrap_or(3) as usize;

        let mut pos = board::fen::Parser::parse(text);
        let mut list = movegen::types::List::new();
        movegen::legal::gen(&mut pos, &mut list);

        let mut branches = Value::array();
        let mut total = 0u64;

        let start = std::time::Instant::now();

        for i in 0..list.len() {
            let mv = list.get(i);
            let state = pos.apply(mv.from, mv.to);
            let count = if depth <= 1 {
                1u64
            } else {
                movegen::perft::perft(&mut pos, depth - 1)
            };
            pos.revert(mv.from, mv.to, &state);

            total += count;

            let mut branch = Value::object();
            branch.insert("move", Value::string(&uci::Format::encode(mv)));
            branch.insert("nodes", Value::number(count as f64));
            branches.push(branch);
        }

        let elapsed = start.elapsed().as_millis() as u64;
        let nps = if elapsed > 0 {
            (total * 1000) / elapsed
        } else {
            total
        };

        let mut out = Value::object();
        out.insert("nodes", Value::number(total as f64));
        out.insert("depth", Value::number(depth as f64));
        out.insert("time_ms", Value::number(elapsed as f64));
        out.insert("nps", Value::number(nps as f64));
        out.insert("divide", branches);

        out
    }

    /// Xử lý MCP Tool 4: parse_fen
    pub fn fen(&mut self, args: &Value) -> Value {
        let text = args
            .get("fen")
            .and_then(|f| f.text())
            .unwrap_or(board::fen::Parser::DEFAULT);

        let pos = board::fen::Parser::parse(text);
        let valid = pos.counts[0] == 1 && pos.counts[7] == 1;

        let side = if pos.side == 0 { "red" } else { "black" };
        let hash = format!("0x{:016X}", pos.hash);

        let chars = [
            "K", "A", "B", "N", "R", "C", "P", "k", "a", "b", "n", "r", "c", "p", ".",
        ];

        let mut grid = Value::array();
        for r in 0..10 {
            let mut row = Value::array();
            for c in 0..9 {
                let sq = r * 9 + c;
                let pc = pos.grid[sq] as usize;
                let label = if pc < 15 { chars[pc] } else { "." };
                row.push(Value::string(label));
            }
            grid.push(row);
        }

        let mut out = Value::object();
        out.insert("side", Value::string(side));
        out.insert("hash", Value::string(&hash));
        out.insert("valid", Value::bool(valid));
        out.insert("grid", grid);

        out
    }

    /// Xử lý MCP Tool 5: get_engine_info
    pub fn info(&self) -> Value {
        let mut caps = Value::array();
        caps.push(Value::string("pvs_search"));
        caps.push(Value::string("nnue_eval"));
        caps.push(Value::string("hce_eval"));
        caps.push(Value::string("perft_test"));
        caps.push(Value::string("fen_parser"));
        caps.push(Value::string("lazy_smp"));
        caps.push(Value::string("mcp_server"));

        let mut tools = Value::array();
        tools.push(Value::string("get_best_move"));
        tools.push(Value::string("evaluate_position"));
        tools.push(Value::string("perft_test"));
        tools.push(Value::string("parse_fen"));
        tools.push(Value::string("get_engine_info"));

        let mut out = Value::object();
        out.insert("name", Value::string("XiangRust"));
        out.insert("version", Value::string("0.1.0"));
        out.insert("author", Value::string("HDQB"));
        out.insert("protocol", Value::string("MCP (Model Context Protocol) JSON-RPC 2.0"));
        out.insert(
            "description",
            Value::string("High-performance Xiangqi AI Engine in pure Rust 2021 (Clean Room Design 0d)"),
        );
        out.insert("capabilities", caps);
        out.insert("tools", tools);
        out.insert("default_fen", Value::string(board::fen::Parser::DEFAULT));

        out
    }
}

// Cho phép khởi tạo Server bằng Default::default()
impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}
