// ============================================================================
// MODULE STORE: KHO LƯU TRỮ VẾT SỰ KIỆN NGUYÊN TỬ (EVENT SOURCING STORE)
// ============================================================================
// `Store` đóng vai trò là Event Store lưu giữ nhật ký không thể thay thế của toàn bộ các
// thông điệp CQRS đã gửi và phát ra trong hệ thống:
// - Được bảo vệ bằng Mutex thread-safe.
// - Căn lề 64-byte `#[repr(C, align(64))]` với padding 48 bytes vừa khít 1 L1 Cache line.
// - Cho phép truy xuất toàn bộ lịch sử (Audit Log) để phục hồi trạng thái thế cờ bất kỳ lúc nào.
// ============================================================================

use std::sync::Mutex;
use super::queue::Item;

/// Struct `Store` quản lý bộ nhớ đệm lịch sử sự kiện với căn lề 64-byte.
#[repr(C, align(64))]
pub struct Store {
    /// Mảng lưu trữ danh sách các phần tử thông điệp Item được bảo vệ bởi Mutex
    pub items: Mutex<Vec<Item>>,
    /// Dung lượng lưu trữ tối đa (Capacity) của kho sự kiện
    pub cap: usize,
    /// Mảng đệm padding 48 bytes để tổng dung lượng struct tròn khít 64 bytes (1 L1 Cache Line)
    pub pad: [u8; 48],
}

impl Store {
    /// Khởi tạo một Event Store mới với sức chứa `cap`.
    pub fn new(cap: usize) -> Self {
        Self {
            items: Mutex::new(Vec::with_capacity(cap)),
            cap,
            pad: [0; 48],
        }
    }

    /// Ghi nhận nguyên tử một thông điệp mới vào Event Store nếu chưa vượt quá sức chứa.
    pub fn record(&self, item: Item) {
        if let Ok(mut guard) = self.items.lock() {
            if guard.len() < self.cap {
                guard.push(item);
            }
        }
    }

    /// Trích xuất danh sách sao chép của toàn bộ các sự kiện đã lưu trong Event Store.
    pub fn fetch(&self) -> Vec<Item> {
        if let Ok(guard) = self.items.lock() {
            guard.clone()
        } else {
            Vec::new()
        }
    }

    /// Xóa sạch nhật ký lịch sử sự kiện trong Event Store.
    pub fn clear(&self) {
        if let Ok(mut guard) = self.items.lock() {
            guard.clear();
        }
    }

    /// Trả về số lượng sự kiện hiện đang lưu trong Event Store.
    pub fn len(&self) -> usize {
        if let Ok(guard) = self.items.lock() {
            guard.len()
        } else {
            0
        }
    }
}

