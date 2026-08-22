// ============================================================================
// XIANGTI ENGINE: BỘ QUẢN LÝ CỜ HỆ THỐNG VÀ TỰ ĐỘNG ROLLBACK (FEATURE FLAGS & ROLLBACK)
// ============================================================================
// Struct `Flag` và `Manager` điều khiển cờ tính năng runtime/compile-time:
//   1. Quản lý cờ bật/tắt gia tốc GPU, Hàng đợi đệm kép, MVV-LVA, NMP.
//   2. Tự động chuyển trạng thái Rollback về CPU SIMD HCE khi xảy ra sự cố GPU driver.
// Căn lề 64-byte vật lý phòng chống False Sharing.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích Tiếng Việt trên từng dòng mã.
// ============================================================================

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering}; // Nhập các kiểu nguyên tử atomic từ thư viện chuẩn

/// Enum `Feature`: Định nghĩa các loại cờ tính năng trong hệ thống Engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)] // Derive các trait cơ bản
pub enum Feature { // Định nghĩa enum Feature
    /// Cờ bật/tắt gia tốc phần cứng GPU
    Gpu, // Cờ Gpu
    /// Cờ bật/tắt Hàng đợi đệm kép RingBuffer
    Queue, // Cờ Queue
    /// Cờ bật/tắt Sắp xếp nước đi MVV-LVA
    Ordering, // Cờ Ordering
    /// Cờ bật/tắt Cắt tỉa nước đi trống Null Move Pruning
    Pruning, // Cờ Pruning
    /// Cờ tự động Rollback dự phòng CPU SIMD
    Rollback, // Cờ Rollback
} // Kết thúc enum Feature

/// Struct `Manager`: Bộ quản lý cờ tính năng căn lề 64-byte (64 bytes total).
#[repr(C, align(64))] // Căn lề 64-byte phòng False Sharing trên CPU Cache Line
pub struct Manager { // Định nghĩa struct Manager
    /// Cờ gia tốc GPU nguyên tử (1 byte, offset 0)
    gpu: AtomicBool, // Trường cờ gpu
    /// Cờ hàng đợi đệm kép nguyên tử (1 byte, offset 1)
    queue: AtomicBool, // Trường cờ queue
    /// Cờ sắp xếp nước đi nguyên tử (1 byte, offset 2)
    ordering: AtomicBool, // Trường cờ ordering
    /// Cờ cắt tỉa nước đi trống nguyên tử (1 byte, offset 3)
    pruning: AtomicBool, // Trường cờ pruning
    /// Cờ tự động Rollback nguyên tử (1 byte, offset 4)
    rollback: AtomicBool, // Trường cờ rollback
    /// Bộ đếm số lần tự động Rollback ngắt mạch (8 bytes, offset 8..16)
    rollbacks: AtomicUsize, // Trường đếm rollbacks
    /// Mảng đệm 48 byte đảm bảo tổng kích thước struct đúng 64 bytes (48 bytes, offset 16..64)
    pad: [u8; 48], // Trường đệm pad căn lề 64 bytes
} // Kết thúc struct Manager

impl Manager { // Khối triển khai các phương thức cho Manager
    /// Khởi tạo một Manager mới với các cờ tính năng bật mặc định.
    pub fn new() -> Self { // Hàm khởi tạo new
        Self {
            gpu: AtomicBool::new(true), // Bật cờ GPU mặc định
            queue: AtomicBool::new(true), // Bật cờ Queue mặc định
            ordering: AtomicBool::new(true), // Bật cờ Ordering mặc định
            pruning: AtomicBool::new(true), // Bật cờ Pruning mặc định
            rollback: AtomicBool::new(false), // Tắt cờ Rollback ban đầu
            rollbacks: AtomicUsize::new(0), // Khởi tạo đếm rollbacks bằng 0
            pad: [0u8; 48], // Khởi tạo mảng đệm zero
        } // Kết thúc khởi tạo struct
    } // Kết thúc hàm new

    /// Lấy trạng thái của một cờ tính năng `Feature`.
    pub fn check(&self, feature: Feature) -> bool { // Hàm check kiểm tra cờ
        match feature { // Kiểm tra loại cờ
            Feature::Gpu => self.gpu.load(Ordering::Acquire), // Trả về cờ GPU
            Feature::Queue => self.queue.load(Ordering::Acquire), // Trả về cờ Queue
            Feature::Ordering => self.ordering.load(Ordering::Acquire), // Trả về cờ Ordering
            Feature::Pruning => self.pruning.load(Ordering::Acquire), // Trả về cờ Pruning
            Feature::Rollback => self.rollback.load(Ordering::Acquire), // Trả về cờ Rollback
        } // Kết thúc match feature
    } // Kết thúc hàm check

    /// Bật hoặc tắt một cờ tính năng `Feature`.
    pub fn toggle(&self, feature: Feature, enable: bool) { // Hàm toggle thay đổi cờ
        match feature { // Kiểm tra loại cờ
            Feature::Gpu => self.gpu.store(enable, Ordering::Release), // Đặt cờ GPU
            Feature::Queue => self.queue.store(enable, Ordering::Release), // Đặt cờ Queue
            Feature::Ordering => self.ordering.store(enable, Ordering::Release), // Đặt cờ Ordering
            Feature::Pruning => self.pruning.store(enable, Ordering::Release), // Đặt cờ Pruning
            Feature::Rollback => self.rollback.store(enable, Ordering::Release), // Đặt cờ Rollback
        } // Kết thúc match feature
    } // Kết thúc hàm toggle

    /// Kích hoạt tự động Rollback ngắt mạch hạ cấp về CPU SIMD HCE.
    pub fn trigger_rollback(&self) { // Hàm trigger_rollback
        self.rollback.store(true, Ordering::Release); // Đặt cờ Rollback true
        self.gpu.store(false, Ordering::Release); // Tắt cờ GPU
        self.queue.store(false, Ordering::Release); // Tắt cờ Queue
        self.rollbacks.fetch_add(1, Ordering::Relaxed); // Tăng đếm số lần rollback
    } // Kết thúc hàm trigger_rollback

    /// Lấy tổng số lần đã tự động Rollback ngắt mạch.
    pub fn count_rollbacks(&self) -> usize { // Hàm count_rollbacks
        self.rollbacks.load(Ordering::Relaxed) // Trả về số lần rollback
    } // Kết thúc hàm count_rollbacks
} // Kết thúc khối impl Manager
