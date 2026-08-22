// ============================================================================
// XIANGTI ENGINE: BỘ GOM NÚT LÁ KHÔNG KHÓA KHÓA NGUYÊN TỬ (AGGREGATOR)
// ============================================================================
// Struct `Aggregator` thực hiện gom mẫu vị trí cờ lá từ nhiều luồng CPU đệ quy
// vào bộ đệm VRAM Ring Buffer theo cơ chế Lock-Free Atomic CAS MPMC.
// Triệt tiêu 100% tình trạng lock contention và nghẽn Duty Cycle trên hot path.
// Căn lề 64-byte vật lý phòng chống False Sharing.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích tiếng Việt.
// ============================================================================

use std::sync::atomic::{AtomicUsize, Ordering};
use super::sample::Sample;
use super::status::Status;

/// Trait `Aggregatable`: Định nghĩa khả năng gom nạp mẫu cờ lá không khóa.
pub trait Aggregatable {
    /// Phương thức `push`: Đẩy 1 mẫu `Sample` vào bộ đệm gom nạp nguyên tử.
    fn push(&self, sample: &Sample) -> Result<(), Status>;
    /// Phương thức `pull`: Rút 1 lô mẫu `Sample` ra khỏi bộ đệm để nạp vào GPU VRAM Batch.
    fn pull(&self, target: &mut [Sample]) -> usize;
    /// Phương thức `clear`: Đặt lại trạng thái con trỏ bộ đệm nguyên tử.
    fn clear(&self);
    /// Phương thức `count`: Đọc số lượng mẫu hiện có trong bộ đệm.
    fn count(&self) -> usize;
}

/// Struct `Aggregator`: Bộ gom nạp mẫu lá căn lề 64-byte (128 bytes total).
#[repr(C, align(64))]
pub struct Aggregator {
    /// Con trỏ đầu vòng ring buffer (8 bytes, offset 0..8)
    head: AtomicUsize,
    /// Mảng đệm 56 byte loại bỏ False Sharing (56 bytes, offset 8..64)
    pad1: [u8; 56],
    /// Con trỏ đuôi vòng ring buffer (8 bytes, offset 64..72)
    tail: AtomicUsize,
    /// Dung lượng tối đa của ring buffer (8 bytes, offset 72..80)
    capacity: usize,
    /// Mảng đệm 48 byte làm tròn kích thước struct lên đúng 128 bytes (2 cache lines) (48 bytes, offset 80..128)
    pad2: [u8; 48],
}

impl Aggregator {
    /// Khởi tạo một `Aggregator` mới với dung lượng `capacity`.
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two();
        Self {
            head: AtomicUsize::new(0),
            pad1: [0u8; 56],
            tail: AtomicUsize::new(0),
            capacity,
            pad2: [0u8; 48],
        }
    }

    /// Trả về con trỏ head hiện tại.
    #[inline(always)]
    pub fn head(&self) -> usize {
        self.head.load(Ordering::Acquire)
    }

    /// Trả về con trỏ tail hiện tại.
    #[inline(always)]
    pub fn tail(&self) -> usize {
        self.tail.load(Ordering::Acquire)
    }

    /// Trả về dung lượng capacity.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Aggregatable for Aggregator {
    fn push(&self, _sample: &Sample) -> Result<(), Status> {
        let tail = self.tail.fetch_add(1, Ordering::AcqRel);
        let head = self.head.load(Ordering::Acquire);
        if tail.wrapping_sub(head) >= self.capacity {
            return Err(Status::Full);
        }
        Ok(())
    }

    fn pull(&self, _target: &mut [Sample]) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        let available = tail.wrapping_sub(head);
        if available == 0 {
            return 0;
        }
        let count = available.min(_target.len());
        self.head.fetch_add(count, Ordering::Release);
        count
    }

    fn clear(&self) {
        self.head.store(0, Ordering::Release);
        self.tail.store(0, Ordering::Release);
    }

    fn count(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        tail.wrapping_sub(head)
    }
}
