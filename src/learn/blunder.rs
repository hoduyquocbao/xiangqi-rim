// ============================================================================
// MODULE LEARN BLUNDER: PHÂN TÍCH NƯỚC ĐỊ SAI LẦM VÀ TÍCH LŨY ĐIỂM PHẠT (BLUNDER ANALYSIS)
// ============================================================================
// Module `blunder` phát hiện các nước đi làm sụt giảm điểm số đánh giá nghiêm trọng
// (Blunder >= 200 centipawns) và ghi nhận điểm phạt (Penalty Bias) để điều chỉnh
// thứ tự ưu tiên nước đi (Move Ordering) trong các phiên tìm kiếm PVS tương lai.
// Thiết kế 100% Clean Room std-only, căn lề bộ nhớ 64-byte loại bỏ False Sharing,
// và tuân thủ tuyệt đối Quy tắc Định danh Đơn Từ Tiếng Anh (Single-Word Principle).
// ============================================================================

/// Struct `Fault` lưu trữ 1 bản ghi nước đi sai lầm (Blunder Fault Record).
/// Căn lề bộ nhớ 16-byte (`#[repr(C, align(16))]`), dung lượng cố định 32 bytes.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fault {
    /// Mã băm Zobrist của thế cờ s
    pub hash: u64,
    /// Nước đi bị coi là sai lầm a (đóng gói 16-bit)
    pub mv: u16,
    /// Điểm phạt tích lũy (Penalty centipawns)
    pub penalty: i32,
    /// Đệm căn lề 12-byte cho đủ 32 bytes vật lý (32 mod 16 = 0)
    pub pad: [u8; 12],
}

impl Fault {
    /// Khởi tạo 1 bản ghi lỗi mới với đầy đủ thông số.
    #[inline(always)]
    pub fn new(hash: u64, mv: u16, penalty: i32) -> Self {
        Self {
            hash,
            mv,
            penalty,
            pad: [0u8; 12],
        }
    }

    /// Khởi tạo bản ghi lỗi mặc định rỗng.
    #[inline(always)]
    pub fn empty() -> Self {
        Self {
            hash: 0,
            mv: 0,
            penalty: 0,
            pad: [0u8; 12],
        }
    }
}

impl Default for Fault {
    fn default() -> Self {
        Self::empty()
    }
}

/// Sức chứa mặc định của Bảng Nước Lỗi Blunder (4,096 bản ghi)
pub const CAPACITY: usize = 4096;

/// Struct `Blunder` quản lý quá trình phát hiện nước sai và lưu tích lũy điểm phạt.
/// Căn lề bộ nhớ 64-byte (`#[repr(C, align(64))]`) loại bỏ False Sharing trên CPU Cache Line.
#[repr(C, align(64))]
pub struct Blunder {
    /// Mảng chứa các bản ghi lỗi cấp phát trên Heap (128 KB)
    pub faults: Box<[Fault]>,
    /// Số lượng bản ghi lỗi hiện đang được lưu trữ
    pub count: usize,
    /// Sức chứa tối đa của bảng lỗi (4,096)
    pub capacity: usize,
    /// Ngưỡng chênh lệch điểm số bị coi là blunder (mặc định 200 centipawns)
    pub threshold: i32,
    /// Biên độ điểm phạt cộng dồn cho mỗi lần phạm lỗi (mặc định 100 centipawns)
    pub bias: i32,
    /// Mảng đệm căn lề 24-byte đảm bảo header đạt chuẩn 64 bytes
    pub pad: [u8; 24],
}

impl Blunder {
    /// Khởi tạo Bộ quản lý Nước sai `Blunder` mới với sức chứa mặc định 4,096 bản ghi.
    pub fn new() -> Self {
        Self::capacity(CAPACITY)
    }

    /// Khởi tạo `Blunder` với sức chứa tùy chọn `capacity`.
    pub fn capacity(capacity: usize) -> Self {
        let faults = vec![Fault::empty(); capacity].into_boxed_slice();
        Self {
            faults,
            count: 0,
            capacity,
            threshold: 150,
            bias: 500,
            pad: [0u8; 24],
        }
    }

    /// Khởi tạo `Blunder` trực tiếp trên Heap thông qua `Box` tránh tràn Stack.
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }

    /// Kiểm tra xem nước đi `mv` tại thế cờ `hash` có phạm sai lầm (best - played >= threshold) hay không.
    /// Nếu phạm sai lầm, tự động cộng điểm phạt `bias` và trả về `true`.
    pub fn check(&mut self, hash: u64, mv: u16, best: i32, played: i32) -> bool {
        let drop = best - played;
        if drop >= self.threshold {
            self.record(hash, mv, self.bias);
            true
        } else {
            false
        }
    }

    /// Đăng ký hoặc cộng dồn điểm phạt `penalty` cho nước đi `mv` tại thế cờ `hash`.
    pub fn record(&mut self, hash: u64, mv: u16, penalty: i32) {
        if let Some(idx) = self.find(hash, mv) {
            self.faults[idx].penalty = (self.faults[idx].penalty + penalty).min(28000);
        } else if self.count < self.capacity {
            let fault = Fault::new(hash, mv, penalty);
            self.faults[self.count] = fault;
            self.count += 1;
        }
    }

    /// Tra cứu điểm phạt tích lũy cho nước đi `mv` tại thế cờ `hash`. Trả về 0 nếu chưa phạm lỗi.
    pub fn penalty(&self, hash: u64, mv: u16) -> i32 {
        if let Some(idx) = self.find(hash, mv) {
            self.faults[idx].penalty
        } else {
            0
        }
    }

    /// Tìm vị trí chỉ số bản ghi nước lỗi của cặp (hash, mv).
    pub fn find(&self, hash: u64, mv: u16) -> Option<usize> {
        for i in 0..self.count {
            if self.faults[i].hash == hash && self.faults[i].mv == mv {
                return Some(i);
            }
        }
        None
    }

    /// Xóa sạch dữ liệu bảng nước đi sai lầm.
    pub fn clear(&mut self) {
        self.count = 0;
    }

    /// Trả về số lượng nước đi sai lầm đã lưu giữ.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Phân tích hồi quy ván thua (Post-Mortem Defeat Attribution):
    /// Duyệt ngược chuỗi nước đi ván thua, tìm tiết điểm làm sụt giảm điểm số lớn nhất,
    /// gán điểm phạt tích lũy để AI không bao giờ đi lại nước sai lầm đó trong tương lai.
    pub fn analyze_defeat(&mut self, trace: &[crate::learn::replay::Sample], final_reward: f32) -> usize {
        if final_reward >= 0.0 || trace.is_empty() {
            return 0;
        }

        let mut penalized = 0usize;
        let mut min_reward = 1.0f32;
        let mut blunder_sample = None;

        for sample in trace {
            if sample.reward < min_reward {
                min_reward = sample.reward;
                blunder_sample = Some(sample);
            }
        }

        if let Some(sample) = blunder_sample {
            self.record(sample.hash, sample.mv, self.bias * 2);
            penalized += 1;
        }

        penalized
    }

    /// Alias trả về số lượng nước đi sai lầm đã lưu giữ.
    #[inline(always)]
    pub fn count(&self) -> usize {
        self.count
    }
}

impl Default for Blunder {
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO BLUNDER ANALYSIS
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Kiểm thử căn lề bộ nhớ và dung lượng struct
    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<Fault>(), 16);
        assert_eq!(std::mem::size_of::<Fault>(), 32);
        assert_eq!(std::mem::align_of::<Blunder>(), 64);
    }

    /// Kiểm thử phát hiện và tích lũy điểm phạt blunder
    #[test]
    fn fault() {
        let mut blunder = Blunder::capacity(10);
        assert_eq!(blunder.len(), 0);

        // Best move score = 500, played move score = 250 -> drop = 250 >= 200 (Threshold)
        let ok = blunder.check(1001, 42, 500, 250);
        assert!(ok);
        assert_eq!(blunder.len(), 1);
        assert_eq!(blunder.penalty(1001, 42), 500);

        // Lần sai thứ 2 tích lũy điểm phạt
        blunder.check(1001, 42, 600, 300);
        assert_eq!(blunder.penalty(1001, 42), 1000);
    }

    /// Kiểm thử dọn dẹp bảng blunder
    #[test]
    fn clear() {
        let mut blunder = Blunder::capacity(5);
        blunder.record(1, 1, 100);
        assert_eq!(blunder.len(), 1);
        blunder.clear();
        assert_eq!(blunder.len(), 0);
    }
}
