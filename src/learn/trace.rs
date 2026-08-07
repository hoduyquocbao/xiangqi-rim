// ============================================================================
// MODULE LEARN TRACE: THUẬT TOÁN HỌC VẾT ĐIỀU KIỆN ELIGIBILITY TRACE & TD(LAMBDA)
// ============================================================================
// Module `trace` triển khai thuật toán học tăng cường Temporal Difference TD(lambda)
// duy trì vết điều kiện (Eligibility Trace) e(s) và cập nhật giá trị trạng thái V(s).
// Thiết kế 100% Clean Room std-only, căn lề bộ nhớ 64-byte loại bỏ False Sharing,
// và tuân thủ tuyệt đối Quy tắc Định danh Đơn Từ Tiếng Anh (Single-Word Principle).
// ============================================================================

/// Struct `Entry` lưu vết eligibility trace và giá trị đánh giá V(s) của thế cờ.
/// Căn lề bộ nhớ 16-byte (`#[repr(C, align(16))]`), dung lượng 32 bytes.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entry {
    /// Mã băm Zobrist của thế cờ s
    pub hash: u64,
    /// Giá trị dự báo thế cờ V(s) (-1.0 đến +1.0)
    pub value: f32,
    /// Biên độ vết eligibility trace hiện tại e(s) >= 0.0
    pub trace: f32,
    /// Mốc thời điểm bước học (step) cập nhật gần nhất
    pub updated: u32,
    /// Đệm căn lề 12-byte cho đủ 32 bytes vật lý (32 mod 16 = 0)
    pub pad: [u8; 12],
}

impl Entry {
    /// Khởi tạo 1 bản ghi vết mới cho thế cờ `hash` với giá trị ban đầu `value`.
    #[inline(always)]
    pub fn new(hash: u64, value: f32, updated: u32) -> Self {
        Self {
            hash,
            value,
            trace: 1.0,
            updated,
            pad: [0u8; 12],
        }
    }

    /// Khởi tạo bản ghi vết rỗng.
    #[inline(always)]
    pub fn empty() -> Self {
        Self {
            hash: 0,
            value: 0.0,
            trace: 0.0,
            updated: 0,
            pad: [0u8; 12],
        }
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self::empty()
    }
}

/// Sức chứa mặc định của Bảng Vết Eligibility Trace (4,096 vị trí)
pub const CAPACITY: usize = 4096;

/// Struct `Trace` quản lý quá trình cập nhật sai số TD(lambda) và suy giảm vết e(s).
/// Căn lề bộ nhớ 64-byte (`#[repr(C, align(64))]`) loại bỏ False Sharing trên CPU L1/L2 Cache Line.
#[repr(C, align(64))]
pub struct Trace {
    /// Mảng chứa các bản ghi vết cấp phát trên Heap (128 KB)
    pub entries: Box<[Entry]>,
    /// Hệ số chiết khấu phần thưởng gamma (mặc định 0.99)
    pub gamma: f32,
    /// Hệ số suy giảm vết lambda (mặc định 0.75)
    pub lambda: f32,
    /// Tốc độ học alpha (mặc định 0.02)
    pub alpha: f32,
    /// Số lượng bản ghi vết đang hoạt động
    pub count: usize,
    /// Sức chứa tối đa của bảng vết (4,096)
    pub capacity: usize,
    /// Bộ đếm mốc thời gian tiến trình học (step counter)
    pub step: u32,
    /// Mảng đệm căn lề 16-byte đảm bảo header đạt chuẩn 64 bytes
    pub pad: [u8; 16],
}

impl Trace {
    /// Khởi tạo Bộ máy Cập nhật Vết `Trace` với các tham số mặc định (gamma=0.99, lambda=0.75, alpha=0.02).
    pub fn new() -> Self {
        Self::capacity(CAPACITY)
    }

    /// Khởi tạo `Trace` với sức chứa tùy chọn `capacity`.
    pub fn capacity(capacity: usize) -> Self {
        let entries = vec![Entry::empty(); capacity].into_boxed_slice();
        Self {
            entries,
            gamma: 0.99,
            lambda: 0.75,
            alpha: 0.02,
            count: 0,
            capacity,
            step: 0,
            pad: [0u8; 16],
        }
    }

    /// Khởi tạo `Trace` trực tiếp trên Heap thông qua `Box` tránh tràn Stack.
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }

    /// Cập nhật sai số dự báo TD(0) delta = r + gamma * V(s') * (1 - done) - V(s),
    /// suy giảm vết e(s) = gamma * lambda * e(s), tăng cường vết e(s_t) += 1.0,
    /// và cập nhật giá trị V(s) += alpha * delta * e(s). Trả về giá trị delta.
    pub fn update(&mut self, hash: u64, next: u64, reward: f32, done: bool) -> f32 {
        self.step += 1;
        let present = self.value(hash);
        let future = if done { 0.0 } else { self.value(next) };

        // 1. Tra cứu hoặc chèn vết cho thế cờ hiện tại s_t VÀ tăng cường trace += 1.0 trước
        if let Some(idx) = self.find(hash) {
            self.entries[idx].trace += 1.0;
            self.entries[idx].updated = self.step;
        } else if self.count < self.capacity {
            let mut entry = Entry::new(hash, present, self.step);
            entry.trace = 1.0;
            self.entries[self.count] = entry;
            self.count += 1;
        }

        // 2. Tính sai số dự báo TD(0) Error
        let delta = reward + self.gamma * future - present;

        // 3. Cập nhật V(s) cho toàn bộ vết active bằng trace đã được tăng cường
        for i in 0..self.count {
            self.entries[i].value += self.alpha * delta * self.entries[i].trace;
        }

        // 4. Áp dụng hệ số suy giảm vết gamma * lambda cho bước kế tiếp
        let factor = self.gamma * self.lambda;
        for i in 0..self.count {
            self.entries[i].trace *= factor;
        }

        // 5. Tự động thu dọn vết quá nhỏ (< 0.0001)
        self.decay();

        delta
    }

    /// Tra cứu giá trị dự báo V(s) của thế cờ `hash`. Trả về 0.0 nếu chưa có.
    pub fn value(&self, hash: u64) -> f32 {
        if let Some(idx) = self.find(hash) {
            self.entries[idx].value
        } else {
            0.0
        }
    }

    /// Tìm vị trí chỉ số của thế cờ `hash` trong bảng vết.
    pub fn find(&self, hash: u64) -> Option<usize> {
        for i in 0..self.count {
            if self.entries[i].hash == hash {
                return Some(i);
            }
        }
        None
    }

    /// Tự động loại bỏ các phần tử vết có biên độ e(s) nhỏ hơn 0.0001.
    pub fn decay(&mut self) {
        let mut write = 0;
        for read in 0..self.count {
            if self.entries[read].trace >= 0.0001 {
                if write != read {
                    self.entries[write] = self.entries[read];
                }
                write += 1;
            }
        }
        self.count = write;
    }

    /// Đặt lại toàn bộ dữ liệu bảng vết.
    pub fn clear(&mut self) {
        self.count = 0;
        self.step = 0;
    }

    /// Trả về số lượng phần tử vết đang có hiệu lực.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Alias trả về số lượng phần tử vết đang có hiệu lực.
    #[inline(always)]
    pub fn count(&self) -> usize {
        self.count
    }
}

impl Default for Trace {
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO ELIGIBILITY TRACE & TD(LAMBDA)
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Kiểm thử căn lề bộ nhớ và dung lượng struct
    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<Entry>(), 16);
        assert_eq!(std::mem::size_of::<Entry>(), 32);
        assert_eq!(std::mem::align_of::<Trace>(), 64);
    }

    /// Kiểm thử cập nhật giá trị TD(0) và vết trace
    #[test]
    fn update() {
        let mut trace = Trace::capacity(10);

        let delta1 = trace.update(100, 200, 0.0, false);
        assert_eq!(trace.len(), 1);
        assert_eq!(delta1, 0.0);

        // Nước thứ 2 với reward +1.0
        let delta2 = trace.update(200, 300, 1.0, true);
        assert!(delta2 > 0.0);
        assert_eq!(trace.len(), 2);
    }

    /// Kiểm thử dọn dẹp vết yếu
    #[test]
    fn decay() {
        let mut trace = Trace::capacity(10);
        trace.update(100, 200, 0.0, false);
        assert_eq!(trace.len(), 1);

        // Cho suy giảm 50 lần để trace rơi xuống dưới 0.0001
        for _ in 0..50 {
            for i in 0..trace.count {
                trace.entries[i].trace *= 0.5;
            }
        }
        trace.decay();
        assert_eq!(trace.len(), 0);
    }
}
