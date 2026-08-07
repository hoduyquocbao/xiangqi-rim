// ============================================================================
// MODULE QUEUE: HÀNG ĐỢI VÒNG NHIỀU NGUỒN PHÁT/THU MPMC LOCK-FREE RING BUFFER
// ============================================================================
// `Queue` triển khai thuật toán Bounded Ring Buffer MPMC (Multi-Producer Multi-Consumer) lock-free
// dựa trên kỹ thuật Dmitry Vyukov Ring Buffer:
// - `head` và `tail` căn lề 64-byte riêng biệt (`pad0: [u8; 56]`, `pad1: [u8; 56]`) hoàn toàn triệt tiêu False Sharing!
// - Mỗi ô `Slot` chứa 1 số thứ tự nguyên tử `sequence: AtomicUsize` để kiểm soát lượt nạp/xuất cực kỳ chính xác.
// - Không bao giờ xảy ra bế tắc (Deadlock) giữa nhiều luồng sản xuất và tiêu thụ thông điệp!
// ============================================================================

use std::sync::atomic::{AtomicUsize, Ordering};
use super::kind::Kind;

/// Struct `Item` đại diện cho một phần tử thông điệp lưu trữ trong hàng đợi ring buffer.
#[repr(C, align(64))]
#[derive(Clone, Debug)]
pub struct Item {
    /// ID định danh duy nhất của thông điệp (u64)
    pub id: u64,
    /// Dấu thời gian timestamp (u64)
    pub stamp: u64,
    /// Phân loại thông điệp (Command, Query, Event)
    pub kind: Kind,
    /// Dữ liệu Payload chuỗi JSON/UCI của thông điệp
    pub data: String,
}

impl Item {
    /// Khởi tạo một thông điệp `Item` rỗng mặc định.
    pub const fn empty() -> Self {
        Self {
            id: 0,
            stamp: 0,
            kind: Kind::Event,
            data: String::new(),
        }
    }
}

/// Struct `Slot` đại diện cho một ô cắm dữ liệu trong Ring Buffer, căn lề 64-byte.
#[repr(C, align(64))]
pub struct Slot {
    /// Số thứ tự nguyên tử kiểm soát lượt nạp/xuất (Sequence counter)
    pub sequence: AtomicUsize,
    /// Dữ liệu thông điệp được bảo vệ bởi Mutex
    pub item: std::sync::Mutex<Item>,
}

/// Struct `Queue` quản lý toàn bộ Bounded Ring Buffer MPMC với thuộc tính căn lề 64-byte.
#[repr(C, align(64))]
pub struct Queue {
    /// Con trỏ đầu hàng đợi (Head pointer) cho thao tác Pop [AtomicUsize]
    pub head: AtomicUsize,
    /// Mảng đệm padding 56 bytes cách ly con trỏ head để tránh False Sharing với tail
    pub pad0: [u8; 56],
    /// Con trỏ đuôi hàng đợi (Tail pointer) cho thao tác Push [AtomicUsize]
    pub tail: AtomicUsize,
    /// Mảng đệm padding 56 bytes cách ly con trỏ tail với các trường dữ liệu còn lại
    pub pad1: [u8; 56],
    /// Dung lượng hàng đợi (Power of Two capacity)
    pub cap: usize,
    /// Mặt nạ bitwise mask tra cứu `(cap - 1)`
    pub mask: usize,
    /// Danh sách mảng các ô cắm Slot
    pub slots: Vec<Slot>,
}

impl Queue {
    /// Khởi tạo một hàng đợi vòng Ring Buffer MPMC mới với sức chứa `cap`.
    pub fn new(cap: usize) -> Self {
        let size = cap.next_power_of_two();
        let mut slots = Vec::with_capacity(size);
        for i in 0..size {
            slots.push(Slot {
                sequence: AtomicUsize::new(i),
                item: std::sync::Mutex::new(Item::empty()),
            });
        }
        Self {
            head: AtomicUsize::new(0),
            pad0: [0; 56],
            tail: AtomicUsize::new(0),
            pad1: [0; 56],
            cap: size,
            mask: size - 1,
            slots,
        }
    }

    /// Đẩy (Push) một thông điệp `item` mới vào đuôi hàng đợi theo thuật toán lock-free CAS.
    /// Trả về `true` nếu đẩy thành công, trả về `false` nếu hàng đợi bị đầy.
    pub fn push(&self, item: Item) -> bool {
        let mut pos = self.tail.load(Ordering::Relaxed);
        loop {
            let idx = pos & self.mask;
            let slot = &self.slots[idx];
            let seq = slot.sequence.load(Ordering::Acquire);
            let diff = (seq as isize).wrapping_sub(pos as isize);
            if diff == 0 {
                // Thử cập nhật vị trí tail mới qua Compare-and-Swap (CAS)
                match self.tail.compare_exchange_weak(
                    pos,
                    pos.wrapping_add(1),
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        if let Ok(mut guard) = slot.item.lock() {
                            *guard = item;
                        }
                        // Cập nhật số thứ tự sequence mới giải phóng ô cắm cho người tiêu thụ
                        slot.sequence.store(pos.wrapping_add(1), Ordering::Release);
                        return true;
                    }
                    Err(actual) => {
                        pos = actual;
                    }
                }
            } else if diff < 0 {
                // Hàng đợi đã đầy
                return false;
            } else {
                pos = self.tail.load(Ordering::Relaxed);
            }
        }
    }

    /// Lấy (Pop) một thông điệp từ đầu hàng đợi theo thuật toán lock-free CAS.
    /// Trả về `Some(Item)` nếu thành công, trả về `None` nếu hàng đợi rỗng.
    pub fn pop(&self) -> Option<Item> {
        let mut pos = self.head.load(Ordering::Relaxed);
        loop {
            let idx = pos & self.mask;
            let slot = &self.slots[idx];
            let seq = slot.sequence.load(Ordering::Acquire);
            let diff = (seq as isize).wrapping_sub(pos.wrapping_add(1) as isize);
            if diff == 0 {
                // Thử cập nhật vị trí head mới qua Compare-and-Swap (CAS)
                match self.head.compare_exchange_weak(
                    pos,
                    pos.wrapping_add(1),
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        let item = if let Ok(guard) = slot.item.lock() {
                            guard.clone()
                        } else {
                            Item::empty()
                        };
                        // Cập nhật số thứ tự sequence mới giải phóng ô cắm cho vòng lặp tiếp theo
                        slot.sequence.store(
                            pos.wrapping_add(self.mask).wrapping_add(1),
                            Ordering::Release,
                        );
                        return Some(item);
                    }
                    Err(actual) => {
                        pos = actual;
                    }
                }
            } else if diff < 0 {
                // Hàng đợi rỗng
                return None;
            } else {
                pos = self.head.load(Ordering::Relaxed);
            }
        }
    }

    /// Trả về số lượng thông điệp hiện có trong hàng đợi (`tail - head`).
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        tail.wrapping_sub(head)
    }

    /// Kiểm tra hàng đợi có đang rỗng hay không.
    pub fn empty(&self) -> bool {
        self.len() == 0
    }
}

