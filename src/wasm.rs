// ============================================================================
// MODULE WASM: GIAO DIỆN BINDINGS WEBASSEMBLY C-ABI PRESERVED STD-ONLY
// ============================================================================
// Module `wasm` cung cấp các cổng FFI C-ABI (C-compatible Application Binary Interface)
// xuất khẩu lõi Engine XiangRust cho môi trường trình duyệt WebAssembly (WASM):
// - 100% Clean Room Design std-only (0 external crates, không dùng wasm-bindgen).
// - Single-Word English Identifiers theo tiêu chuẩn hệ thống.
// - Căn lề bộ nhớ phần cứng `#[repr(C, align(64))]` cho WasmBuffer và WasmEngine.
// - 100% chú thích Tiếng Việt siêu chi tiết tới từng dòng mã nguồn.
// ============================================================================

use crate::board::Position;
use crate::eval::Eval;
use crate::search::Search;

#[cfg(target_arch = "wasm32")]
use std::slice;
#[cfg(target_arch = "wasm32")]
use std::str;
#[cfg(target_arch = "wasm32")]
use crate::board::{Parser, Serializer};
#[cfg(target_arch = "wasm32")]
use crate::movegen::perft as count;
#[cfg(target_arch = "wasm32")]
use crate::search::Limits;
#[cfg(target_arch = "wasm32")]
use crate::uci::Format;

/// Cấu trúc `WasmBuffer` quản lý vùng đệm dữ liệu đầu ra FFI, căn lề 64-byte.
#[repr(C, align(64))]
pub struct WasmBuffer {
    /// Mảng chứa dữ liệu chuỗi UTF-8 kết quả (dung lượng 4096 bytes)
    pub data: [u8; 4096],
    /// Số lượng byte thực tế đang lưu trữ trong đệm
    pub size: usize,
}

impl WasmBuffer {
    /// Khởi tạo đệm rỗng với dung lượng 4096 bytes.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            data: [0; 4096],
            size: 0,
        }
    }

    /// Xóa toàn bộ dữ liệu hiện có trong đệm về trạng thái rỗng.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.size = 0;
    }

    /// Ghi chuỗi văn bản UTF-8 vào đệm kết quả.
    #[inline(always)]
    pub fn write(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let limit = bytes.len().min(4096);
        self.data[..limit].copy_from_slice(&bytes[..limit]);
        self.size = limit;
    }
}

/// Cấu trúc `WasmEngine` quản lý trạng thái toàn cục của Engine trong WASM, căn lề 64-byte.
#[repr(C, align(64))]
pub struct WasmEngine {
    /// Vị trí bàn cờ hiện tại
    pub pos: Position,
    /// Bộ máy tìm kiếm PVS
    pub search: Box<Search>,
    /// Bộ đánh giá thế cờ NNUE + HCE
    pub eval: Eval,
    /// Vùng đệm kết quả FFI
    pub buffer: WasmBuffer,
}

/// Thể hiện toàn cục tĩnh của WasmEngine trong bộ nhớ WASM (chỉ biên dịch cho target WASM)
#[cfg(target_arch = "wasm32")]
static mut ENGINE: Option<Box<WasmEngine>> = None;

/// Khởi tạo trạng thái Engine toàn cục trong WASM.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn init() -> i32 {
    let engine = Box::new(WasmEngine {
        pos: Parser::parse(Parser::DEFAULT),
        search: Search::new_boxed(2),
        eval: Eval::new(),
        buffer: WasmBuffer::new(),
    });
    unsafe {
        ENGINE = Some(engine);
    }
    1
}

/// Phân tích chuỗi FEN từ JavaScript và thiết lập vị trí bàn cờ hiện tại.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn set_position(ptr: *const u8, len: usize) -> i32 {
    if ptr.is_null() || len == 0 {
        return 0;
    }
    let bytes = unsafe { slice::from_raw_parts(ptr, len) };
    let text = match str::from_utf8(bytes) {
        Ok(valid) => valid,
        Err(_) => return 0,
    };
    let parsed = Parser::parse(text);
    unsafe {
        if let Some(ref mut engine) = ENGINE {
            engine.pos = parsed;
            engine.eval.reset(&engine.pos);
            1
        } else {
            0
        }
    }
}

/// Thực thi phiên tìm kiếm PVS với độ sâu `depth` và thời gian `time_ms`.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn search(depth: u32, time_ms: u32) -> i32 {
    unsafe {
        if let Some(ref mut engine) = ENGINE {
            let mut limits = Limits::new();
            limits.depth = depth as u8;
            limits.exact = time_ms as u64;

            let result = engine.search.go(&engine.pos, &limits);
            let best = Format::encode(result.best);

            // Dựng chuỗi tuyến nước đi biến thể chính PV
            let mut pv = String::new();
            let mut i = 0;
            while i < result.pv.count {
                let item = result.pv.items[i];
                if item.valid() {
                    if !pv.is_empty() {
                        pv.push(' ');
                    }
                    pv.push_str(&Format::encode(item));
                }
                i += 1;
            }

            // Ghi kết quả dưới dạng chuỗi JSON vào WasmBuffer
            let json = format!(
                "{{\"best\":\"{}\",\"score\":{},\"depth\":{},\"nodes\":{},\"time\":{},\"pv\":\"{}\"}}",
                best, result.score, result.depth, result.nodes, result.time, pv
            );
            engine.buffer.write(&json);
            if result.best.valid() { 1 } else { 0 }
        } else {
            0
        }
    }
}

/// Đánh giá thế cờ hiện tại bằng bộ NNUE + HCE và trả về điểm centipawns.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn evaluate() -> i32 {
    unsafe {
        if let Some(ref mut engine) = ENGINE {
            engine.eval.score(&engine.pos)
        } else {
            0
        }
    }
}

/// Đếm tổng số nút cây nước đi Perft ở độ sâu `depth`.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn perft(depth: u32) -> u64 {
    unsafe {
        if let Some(ref mut engine) = ENGINE {
            count::perft(&mut engine.pos, depth as usize)
        } else {
            0
        }
    }
}

/// Cấp phát vùng nhớ đệm tĩnh dung lượng `size` bytes cho JavaScript ghi dữ liệu.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn allocate(size: usize) -> *mut u8 {
    if size == 0 {
        return std::ptr::null_mut();
    }
    let mut vec = vec![0u8; size];
    let ptr = vec.as_mut_ptr();
    std::mem::forget(vec);
    ptr
}

/// Giải phóng vùng nhớ đệm đã cấp phát từ con trỏ thô `ptr` (ngăn xung đột ký hiệu free của libc hệ thống).
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn free(ptr: *mut u8, size: usize) {
    if !ptr.is_null() && size > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(ptr, size, size);
        }
    }
}

/// Chép nội dung dữ liệu kết quả từ WasmBuffer ra con trỏ thô của JavaScript.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn fetch(ptr: *mut u8, limit: usize) -> usize {
    if ptr.is_null() || limit == 0 {
        return 0;
    }
    unsafe {
        if let Some(ref engine) = ENGINE {
            let count = engine.buffer.size.min(limit);
            let dest = slice::from_raw_parts_mut(ptr, count);
            dest.copy_from_slice(&engine.buffer.data[..count]);
            count
        } else {
            0
        }
    }
}

/// Trích xuất chuỗi FEN hiện tại của bàn cờ ra con trỏ thô của JavaScript.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn fen(ptr: *mut u8, limit: usize) -> usize {
    if ptr.is_null() || limit == 0 {
        return 0;
    }
    unsafe {
        if let Some(ref engine) = ENGINE {
            let text = Serializer::export(&engine.pos);
            let bytes = text.as_bytes();
            let count = bytes.len().min(limit);
            let dest = slice::from_raw_parts_mut(ptr, count);
            dest.copy_from_slice(&bytes[..count]);
            count
        } else {
            0
        }
    }
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO MODULE WASM FFI
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Kiểm thử căn lề bộ nhớ 64-byte phần cứng cho WasmBuffer và WasmEngine.
    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<WasmBuffer>(), 64);
        assert_eq!(std::mem::align_of::<WasmEngine>(), 64);
    }

    /// Kiểm thử quy trình khởi tạo, nạp FEN, đánh giá, tìm kiếm và giải phóng đệm WASM FFI (chỉ chạy trên WASM).
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn workflow() {
        assert_eq!(init(), 1);

        let fen_str = Parser::DEFAULT;
        let ptr = allocate(fen_str.len());
        assert!(!ptr.is_null());

        unsafe {
            let slice = slice::from_raw_parts_mut(ptr, fen_str.len());
            slice.copy_from_slice(fen_str.as_bytes());
        }

        assert_eq!(set_position(ptr, fen_str.len()), 1);
        free(ptr, fen_str.len());

        let eval_score = evaluate();
        assert!(eval_score.abs() < 2000);

        let perft_count = perft(1);
        assert_eq!(perft_count, 44);

        let search_res = search(2, 100);
        assert_eq!(search_res, 1);

        let out_ptr = allocate(4096);
        let fetched = fetch(out_ptr, 4096);
        assert!(fetched > 0);

        unsafe {
            let out_slice = slice::from_raw_parts(out_ptr, fetched);
            let out_str = str::from_utf8(out_slice).unwrap();
            assert!(out_str.contains("best"));
            assert!(out_str.contains("score"));
        }
        free(out_ptr, 4096);

        let fen_ptr = allocate(256);
        let fen_len = fen(fen_ptr, 256);
        assert!(fen_len > 0);
        free(fen_ptr, 256);
    }
}
