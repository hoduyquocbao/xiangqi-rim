// ============================================================================
// MODULE BUS: TRUNG TÂM ĐIỀU PHỐI THÔNG ĐIỆP HỆ THỐNG CQRS (MESSAGE BUS FACADE)
// ============================================================================
// `Bus` đóng vai trò là Facade điều phối trung tâm tất cả các luồng dữ liệu CQRS:
// - Tiếp nhận Lệnh điều khiển `Command` từ bên ngoài qua phương thức `send()`.
// - Tiếp nhận Truy vấn `Query` từ bên ngoài qua phương thức `ask()`.
// - Bắn Sự kiện `Event` ra Event Store và Ring Buffer qua phương thức `emit()`.
// - Tự động ghi nhận mọi tác vụ vào `Store` (Event Sourcing) để lưu vết trước khi đẩy vào `Queue`.
// - Căn lề 64-byte `#[repr(C, align(64))]` đảm bảo tính tương thích phần cứng tối ưu.
// ============================================================================

use std::sync::Arc;
use super::command::Command;
use super::event::Event;
use super::kind::Kind;
use super::query::Query;
use super::queue::{Item, Queue};
use super::store::Store;

/// Struct `Bus` đóng vai trò là trung tâm điều phối thông điệp CQRS, căn lề 64-byte.
#[repr(C, align(64))]
#[derive(Clone)]
pub struct Bus {
    /// Con trỏ đếm tham chiếu Arc trỏ tới Ring Buffer MPMC Queue
    pub queue: Arc<Queue>,
    /// Con trỏ đếm tham chiếu Arc trỏ tới Event Store
    pub store: Arc<Store>,
    /// Mảng đệm padding 48 bytes tròn 64 bytes (1 L1 Cache Line)
    pub pad: [u8; 48],
}

impl Default for Bus {
    /// Khởi tạo Bus mặc định với hàng đợi 1,024 ô và Event Store 65,536 ô.
    fn default() -> Self {
        Self::new(1024, 65536)
    }
}

impl Bus {
    /// Khởi tạo một đối tượng `Bus` mới với dung lượng hàng đợi `qcap` và dung lượng kho `scap`.
    pub fn new(qcap: usize, scap: usize) -> Self {
        Self {
            queue: Arc::new(Queue::new(qcap)),
            store: Arc::new(Store::new(scap)),
            pad: [0; 48],
        }
    }

    /// Gửi một Lệnh điều khiển `Command` vào hệ thống CQRS.
    /// Ghi vết vào Event Store và đẩy vào Ring Buffer Queue.
    pub fn send(&self, cmd: Command) -> bool {
        let item = Item {
            id: 1,
            stamp: 0,
            kind: Kind::Command,
            data: format!("{:?}", cmd),
        };
        self.store.record(item.clone());
        self.queue.push(item)
    }

    /// Gửi một Truy vấn `Query` đọc dữ liệu vào hệ thống CQRS.
    pub fn ask(&self, query: Query) -> Option<Item> {
        let item = Item {
            id: 2,
            stamp: 0,
            kind: Kind::Query,
            data: format!("{:?}", query),
        };
        self.store.record(item.clone());
        Some(item)
    }

    /// Bắn (Emit) một Sự kiện `Event` mới ra hệ thống CQRS.
    pub fn emit(&self, event: Event) -> bool {
        let item = Item {
            id: 3,
            stamp: 0,
            kind: Kind::Event,
            data: format!("{:?}", event),
        };
        self.store.record(item.clone());
        self.queue.push(item)
    }

    /// Trích xuất (Poll) một thông điệp tiếp theo từ đầu hàng đợi Ring Buffer Queue.
    pub fn poll(&self) -> Option<Item> {
        self.queue.pop()
    }

    /// Xóa sạch nhật ký lưu vết trong Event Store.
    pub fn clear(&self) {
        self.store.clear();
    }
}

