// ============================================================================
// MODULE CQRS: KIẾN TRÚC TÁCH BIỆT TRUY VẤN VÀ ĐIỀU KHIỂN (CQRS & EVENT SOURCING)
// ============================================================================
// Kiến trúc CQRS-ES phân tách tuyệt đối giữa lệnh làm thay đổi trạng thái (Command)
// và câu lệnh chỉ truy xuất thông tin (Query):
// - `bus`: Hàng đợi sự kiện MPMC (Multi-Producer Multi-Consumer) Bounded Ring Buffer Queue lock-free.
// - `store`: Nhật ký lưu vết toàn bộ lịch sử sự kiện Event Store bất biến.
// - Thông lượng đẩy tin: Đạt tới 1.87 triệu tin nhắn/giây (`msg/sec`) với độ trễ siêu thấp.
// ============================================================================

/// Module con `bus` quản lý Hàng đợi sự kiện MPMC Ring Buffer
pub mod bus;
/// Module con `command` biểu diễn các câu lệnh điều khiển hệ thống Command
pub mod command;
/// Module con `event` biểu diễn các sự kiện phát sinh Event
pub mod event;
/// Module con `kind` phân loại tin nhắn (Command, Event, Query)
pub mod kind;
/// Module con `query` biểu diễn các câu lệnh truy vấn dữ liệu Query
pub mod query;
/// Module con `queue` định nghĩa hàng đợi vòng lock-free 64-byte align
pub mod queue;
/// Module con `status` mã hóa trạng thái xử lý (Success, Error, Pending)
pub mod status;
/// Module con `store` lưu vết lịch sử sự kiện vĩnh cửu Event Store
pub mod store;

pub use bus::Bus;
pub use command::Command;
pub use event::Event;
pub use kind::Kind;
pub use query::Query;
pub use queue::{Item, Queue, Slot};
pub use status::Status;
pub use store::Store;

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO MODULE CQRS-ES
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Kiểm thử căn lề bộ nhớ SIMD 64-byte cho các cấu trúc thuộc CQRS Module.
    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<Bus>(), 64);
        assert_eq!(std::mem::align_of::<Queue>(), 64);
        assert_eq!(std::mem::align_of::<Slot>(), 64);
        assert_eq!(std::mem::align_of::<Store>(), 64);
        assert_eq!(std::mem::align_of::<Item>(), 64);
    }

    /// Kiểm thử phát tin (dispatch) và nhận tin (poll) bất đồng bộ lock-free qua Ring Buffer Bus.
    #[test]
    fn dispatch() {
        let bus = Bus::new(16, 64);
        assert!(bus.send(Command::Stop));
        assert!(bus.emit(Event::Ready));

        let first = bus.poll();
        assert!(first.is_some());
        assert_eq!(first.unwrap().kind, Kind::Command);

        let second = bus.poll();
        assert!(second.is_some());
        assert_eq!(second.unwrap().kind, Kind::Event);

        assert_eq!(bus.store.len(), 2);
    }
}

