// ============================================================================
// MODULE LEARN REPLAY: BỘ ĐỆM KINH NGHIỆM XOAY VÒNG (EXPERIENCE REPLAY BUFFER)
// ============================================================================
// Module `replay` triển khai bộ đệm xoay vòng FIFO (Circular Buffer) chứa các
// mẫu chuyển dịch kinh nghiệm (Sample) thu được từ các ván tự đấu (Self-Play).
// Thiết kế 100% Clean Room std-only, căn lề bộ nhớ 64-byte loại bỏ False Sharing,
// và tuân thủ tuyệt đối Quy tắc Định danh Đơn Từ Tiếng Anh (Single-Word Principle).
// ============================================================================

/// Struct `Sample` lưu trữ 1 bản ghi chuyển dịch kinh nghiệm (s, a, r, s', done).
/// Căn lề bộ nhớ 16-byte (`#[repr(C, align(16))]`), dung lượng cố định 32 bytes.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    /// Mã băm Zobrist của thế cờ hiện tại s_t
    pub hash: u64,
    /// Mã nước đi đã chọn a_t (đóng gói 16-bit)
    pub mv: u16,
    /// Phần thưởng tức thì r_{t+1} nhận được (-1.0 đến +1.0)
    pub reward: f32,
    /// Mã băm Zobrist của thế cờ kế tiếp s_{t+1}
    pub next: u64,
    /// Cờ báo hiệu ván cờ kết thúc (1: đã kết thúc, 0: chưa kết thúc)
    pub done: u8,
    /// Đệm căn lề 7-byte cho đủ 32 bytes vật lý (32 mod 16 = 0)
    pub pad: [u8; 7],
}

impl Sample {
    /// Khởi tạo một mẫu kinh nghiệm mới với đầy đủ thông số.
    #[inline(always)]
    pub fn new(hash: u64, mv: u16, reward: f32, next: u64, done: u8) -> Self {
        Self {
            hash,
            mv,
            reward,
            next,
            done,
            pad: [0u8; 7],
        }
    }

    /// Khởi tạo mẫu kinh nghiệm mặc định rỗng.
    #[inline(always)]
    pub fn empty() -> Self {
        Self {
            hash: 0,
            mv: 0,
            reward: 0.0,
            next: 0,
            done: 0,
            pad: [0u8; 7],
        }
    }
}

impl Default for Sample {
    fn default() -> Self {
        Self::empty()
    }
}

/// Sức chứa mặc định của Bộ đệm Kinh nghiệm (10,000 phần tử mẫu)
pub const CAPACITY: usize = 10000;

/// Struct `Replay` quản lý bộ đệm kinh nghiệm xoay vòng FIFO dung lượng 10,000 mẫu.
/// Căn lề bộ nhớ 64-byte (`#[repr(C, align(64))]`) triệt tiêu False Sharing trên CPU Cache Line.
#[repr(C, align(64))]
pub struct Replay {
    /// Mảng chứa các mẫu kinh nghiệm cấp phát trên Heap (320 KB)
    pub samples: Box<[Sample]>,
    /// Vị trí con trỏ ghi xoay vòng hiện tại (0 .. capacity - 1)
    pub head: usize,
    /// Số lượng mẫu hợp lệ hiện tại có trong bộ đệm (0 .. capacity)
    pub count: usize,
    /// Giới hạn sức chứa tối đa của bộ đệm (mặc định 10,000)
    pub capacity: usize,
    /// Hạt giống sinh số ngẫu nhiên Xorshift64 cho thuật toán lấy mẫu std-only
    pub seed: u64,
    /// Mảng đệm căn lề 16-byte đảm bảo header đạt chuẩn 64 bytes
    pub pad: [u8; 16],
}

impl Replay {
    /// Khởi tạo Bộ đệm Kinh nghiệm `Replay` mới với sức chứa mặc định 10,000 mẫu.
    pub fn new() -> Self {
        Self::capacity(CAPACITY)
    }

    /// Khởi tạo Bộ đệm Kinh nghiệm `Replay` với sức chứa tùy chọn `capacity`.
    pub fn capacity(capacity: usize) -> Self {
        let samples = vec![Sample::empty(); capacity].into_boxed_slice();
        Self {
            samples,
            head: 0,
            count: 0,
            capacity,
            seed: 0x853c_42e6_48a6_50d2,
            pad: [0u8; 16],
        }
    }

    /// Khởi tạo `Replay` trực tiếp trên Heap thông qua `Box` tránh quá tải Stack.
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }

    /// Đẩy 1 mẫu kinh nghiệm `sample` mới vào bộ đệm xoay vòng theo cơ chế FIFO.
    pub fn push(&mut self, sample: Sample) {
        self.samples[self.head] = sample;
        self.head = (self.head + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
    }

    /// Trích xuất ngẫu nhiên `batch.len()` mẫu từ bộ đệm vào mảng `batch`.
    /// Trả về số lượng mẫu thực tế được trích xuất thành công.
    pub fn sample(&mut self, batch: &mut [Sample]) -> usize {
        if self.count == 0 || batch.is_empty() {
            return 0;
        }

        let k = batch.len().min(self.count);
        for i in 0..k {
            // Thuật toán sinh số ngẫu nhiên Xorshift64 siêu nhanh O(1) std-only
            let mut x = self.seed;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.seed = x;

            let idx = (x as usize) % self.count;
            batch[i] = self.samples[idx];
        }

        k
    }

    /// Làm rỗng toàn bộ dữ liệu trong bộ đệm kinh nghiệm.
    pub fn clear(&mut self) {
        self.head = 0;
        self.count = 0;
    }

    /// Trả về số lượng mẫu hợp lệ hiện có trong bộ đệm.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Alias trả về số lượng mẫu hợp lệ hiện có trong bộ đệm.
    #[inline(always)]
    pub fn count(&self) -> usize {
        self.count
    }

    /// Kiểm tra bộ đệm có đang rỗng hay không.
    #[inline(always)]
    pub fn empty(&self) -> bool {
        self.count == 0
    }

    /// Lấy tham chiếu an toàn tới phần tử tại chỉ số `index`.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<&Sample> {
        if index < self.count {
            Some(&self.samples[index])
        } else {
            None
        }
    }
}

impl Default for Replay {
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO REPLAY BUFFER
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Kiểm thử căn lề bộ nhớ và dung lượng struct
    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<Sample>(), 16);
        assert_eq!(std::mem::size_of::<Sample>(), 32);
        assert_eq!(std::mem::align_of::<Replay>(), 64);
    }

    /// Kiểm thử đẩy phần tử vào bộ đệm xoay vòng FIFO
    #[test]
    fn push() {
        let mut replay = Replay::capacity(3);
        assert!(replay.empty());
        assert_eq!(replay.len(), 0);

        let s1 = Sample::new(100, 10, 0.5, 101, 0);
        let s2 = Sample::new(200, 20, -0.5, 201, 0);
        let s3 = Sample::new(300, 30, 1.0, 301, 1);
        let s4 = Sample::new(400, 40, -1.0, 401, 1);

        replay.push(s1);
        replay.push(s2);
        replay.push(s3);
        assert_eq!(replay.len(), 3);
        assert_eq!(replay.get(0).unwrap().hash, 100);

        // Đẩy phần tử thứ 4 ghi đè vị trí xoay vòng 0
        replay.push(s4);
        assert_eq!(replay.len(), 3);
        assert_eq!(replay.samples[0].hash, 400);
    }

    /// Kiểm thử thuật toán lấy mẫu ngẫu nhiên std-only
    #[test]
    fn sample() {
        let mut replay = Replay::capacity(10);
        for i in 0..10 {
            replay.push(Sample::new(i as u64, i as u16, 0.0, (i + 1) as u64, 0));
        }

        let mut batch = [Sample::empty(); 4];
        let k = replay.sample(&mut batch);
        assert_eq!(k, 4);
    }

    /// Kiểm thử dọn dẹp bộ đệm
    #[test]
    fn clear() {
        let mut replay = Replay::capacity(5);
        replay.push(Sample::new(1, 1, 0.0, 2, 0));
        assert_eq!(replay.len(), 1);
        replay.clear();
        assert_eq!(replay.len(), 0);
        assert!(replay.empty());
    }
}
