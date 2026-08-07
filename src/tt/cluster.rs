// ============================================================================
// MODULE CLUSTER: CỤM 4 ENTRY NẰM GỌN TRONG 1 DÒNG BỘ NHỚ L1 CACHE (CACHE LINE CLUSTER)
// ============================================================================
// Cấu trúc `Cluster` chứa 4 `Entry` 16-byte ($4 \times 16 = 64$ bytes).
// Sử dụng chỉ thị căn lề `#[repr(C, align(64))]` giúp cả 4 phần tử trong cụm
// nằm khít trong 1 L1 Data Cache Line (64-byte).
// Khi CPU nạp 1 địa chỉ băm, cả 4 ô nhớ được nạp vào Cache cùng 1 lúc với độ trễ 0 cycle!
// ============================================================================

use crate::tt::entry::Entry;

/// Struct `Cluster` chứa mảng 4 khe cắm Entry, căn lề 64-byte.
#[repr(C, align(64))]
pub struct Cluster {
    /// Mảng chứa 4 phần tử Entry nguyên tử 16-byte [64 bytes total]
    pub slots: [Entry; 4],
}

impl Default for Cluster {
    /// Khởi tạo cụm mặc định.
    fn default() -> Self {
        Self::new()
    }
}

impl Cluster {
    /// Khởi tạo một cụm `Cluster` rỗng gồm 4 khe `Entry::empty()`.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            slots: [Entry::empty(), Entry::empty(), Entry::empty(), Entry::empty()],
        }
    }
}

