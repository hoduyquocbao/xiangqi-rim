// ============================================================================
// MODULE STATS: THỐNG KÊ CHỈ SỐ HIỆU NĂNG TỰ ĐẤU (SELF-PLAY METRICS TRACKER)
// ============================================================================
// Struct `Stats` ghi nhận các chỉ số thực thi trong suốt ván đấu:
// - Tổng số nút cây cờ đã duyệt (`nodes`).
// - Tổng thời gian tính toán (`time` ms).
// - Tốc độ duyệt nút trên giây (`nps`).
// - Tổng số nước đi đã thực hiện (`moves`).
// - Căn lề bộ nhớ 64-byte (`#[repr(C, align(64))]`) loại bỏ False Sharing.
// ============================================================================

/// Struct `Stats` lưu trữ thống kê hiệu năng ván tự đấu Cờ Tướng.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Stats {
    /// Tổng số nút đã duyệt trên cây cờ (8 bytes)
    pub nodes: u64,
    /// Tổng thời gian tìm kiếm tính bằng miligiây (8 bytes)
    pub time: u64,
    /// Tốc độ tìm kiếm tính bằng Nút / Giây (NPS) (8 bytes)
    pub nps: u64,
    /// Tổng số nước đi đã thực hiện trong ván (4 bytes)
    pub moves: u32,
    /// Trường đệm đệm căn lề bộ nhớ 64-byte (36 bytes)
    _pad: [u8; 36],
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}


impl Stats {
    /// Khởi tạo một đối tượng Stats mới hoàn toàn rỗng.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            nodes: 0,
            time: 0,
            nps: 0,
            moves: 0,
            _pad: [0; 36],
        }
    }

    /// Tính toán và cập nhật lại tốc độ duyệt nút trên giây NPS.
    #[inline(always)]
    pub fn rate(&mut self) -> u64 {
        if self.time > 0 {
            self.nps = self.nodes.saturating_mul(1000) / self.time;
        } else {
            self.nps = self.nodes;
        }
        self.nps
    }

    /// Tính trung bình số nút đã duyệt trên mỗi nước đi (Nodes / Move).
    #[inline(always)]
    pub fn mean(&self) -> u64 {
        if self.moves > 0 {
            self.nodes / (self.moves as u64)
        } else {
            0
        }
    }

    /// Tính trung bình thời gian tính toán trên mỗi nước đi (Time / Move) tính bằng miligiây.
    #[inline(always)]
    pub fn span(&self) -> u64 {
        if self.moves > 0 {
            self.time / (self.moves as u64)
        } else {
            0
        }
    }
}

// ----------------------------------------------------------------------------
// KHU VỰC BÀI KIỂM THỬ ĐƠN VỊ (UNIT TESTS) CHO STATS MODULE
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};

    /// Kiểm thử căn lề bộ nhớ 64-byte và kích thước vật lý của struct Stats.
    #[test]
    fn alignments() {
        assert_eq!(size_of::<Stats>(), 64);
        assert_eq!(align_of::<Stats>(), 64);
    }

    /// Kiểm thử khởi tạo mặc định và tính toán các chỉ số trung bình (NPS, mean nodes, time span).
    #[test]
    fn calculate() {
        let mut stats = Stats::new();
        assert_eq!(stats.nodes, 0);
        assert_eq!(stats.time, 0);
        assert_eq!(stats.moves, 0);
        assert_eq!(stats.mean(), 0);
        assert_eq!(stats.span(), 0);

        stats.nodes = 100000;
        stats.time = 500; // 0.5 giây
        stats.moves = 10;

        let rate = stats.rate();
        assert_eq!(rate, 200000); // 100,000 * 1000 / 500 = 200,000 NPS
        assert_eq!(stats.mean(), 10000); // 100,000 / 10 = 10,000 nodes/move
        assert_eq!(stats.span(), 50); // 500 / 10 = 50 ms/move
    }
}
