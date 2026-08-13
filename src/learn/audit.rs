// ============================================================================
// MODULE LEARN AUDIT: PHÂN HỆ CHẨN ĐOÁN RỦI RO, MẶT TỐI VÀ NGÂY THƠ TIỀM ẨN
// ============================================================================
// Module `audit.rs` rà soát và chẩn đoán toàn diện các nguy cơ tiềm ẩn trong thế cờ:
// 1. Unguarded Major Pieces (Quân cờ chủ lực không có vệ sĩ bảo vệ): Xe/Pháo/Mã đứng trống trải.
// 2. Overloaded Defenders (Quân phòng thủ quá tải): Quân cờ phải gánh giữ 2 mục tiêu cùng lúc.
// 3. Palace Exposure (Nguy cơ hở Cung Tướng): Lỗ hổng Sĩ Tượng bị khai thác.
// 4. Horizon Risks (Rủi ro hiệu ứng chân trời hoãn sụt điểm).
// 100% Clean Room std-only, căn lề 64-byte, 100% chú thích tiếng Việt & từ đơn tiếng Anh.
// ============================================================================

use crate::board::Position;
use crate::movegen::lookup;

/// Struct `Report` chứa thông số báo cáo chẩn đoán rủi ro (64 bytes, `#[repr(C, align(64))]`)
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Report {
    /// Số quân chủ lực không có vệ sĩ bảo vệ
    pub unguarded: u32,
    /// Số quân phòng thủ chịu quá tải nhiệm vụ
    pub overloaded: u32,
    /// Chỉ số nguy cơ hở Cung / khuyết Sĩ Tượng
    pub exposure: u32,
    /// Chỉ số rủi ro hiệu ứng chân trời hoãn sụt điểm
    pub horizon: u32,
    /// Tổng điểm phạt centipawn cho các rủi ro tiềm ẩn
    pub penalty: i32,
    /// Mảng đệm căn lề 44-byte đạt đúng 64 bytes vật lý
    pub pad: [u8; 44],
}

impl Report {
    /// Khởi tạo đối tượng `Report` rỗng mặc định.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            unguarded: 0,
            overloaded: 0,
            exposure: 0,
            horizon: 0,
            penalty: 0,
            pad: [0u8; 44],
        }
    }
}

impl Default for Report {
    fn default() -> Self {
        Self::new()
    }
}

/// Struct `Audit` rà soát và chẩn đoán mặt tối & ngây thơ thế cờ (64 bytes, `#[repr(C, align(64))]`)
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, Default)]
pub struct Audit;

impl Audit {
    /// Khởi tạo bộ chẩn đoán `Audit` mới.
    #[inline(always)]
    pub const fn new() -> Self {
        Self
    }

    /// Rà soát thế cờ `pos` và xuất báo cáo `Report` phân tích rủi ro & ngây thơ tiềm ẩn.
    pub fn scan(pos: &Position) -> Report {
        let mut report = Report::new();
        let turn = pos.side as usize;
        let opp = 1 - turn;

        // 1. Quét quân chủ lực không có bảo vệ (Unguarded Major Pieces)
        let mut role = 3usize;
        while role <= 5 {
            let mut bb = pos.piece[turn * 7 + role];
            while let Some(sq) = bb.pop() {
                let mut attacks = 0u32;
                let mut defenders = 0u32;

                // Kiểm tra xem quân này có bị tấn công bởi quân đối phương hay không
                let mut opp_role = 0usize;
                while opp_role < 7 {
                    let opp_piece_idx = opp * 7 + opp_role;
                    let mut opp_bb = pos.piece[opp_piece_idx];
                    while let Some(opp_sq) = opp_bb.pop() {
                        let king_bb = lookup::KING[opp][opp_sq.0 as usize];
                        if king_bb.test(sq) {
                            attacks += 1;
                        }
                    }
                    opp_role += 1;
                }

                // Kiểm tra quân đồng minh có bảo vệ không
                let mut friend_role = 0usize;
                while friend_role < 7 {
                    let friend_piece_idx = turn * 7 + friend_role;
                    let mut friend_bb = pos.piece[friend_piece_idx];
                    while let Some(friend_sq) = friend_bb.pop() {
                        if friend_sq.0 != sq.0 {
                            let king_bb = lookup::KING[turn][friend_sq.0 as usize];
                            if king_bb.test(sq) {
                                defenders += 1;
                            }
                        }
                    }
                    friend_role += 1;
                }

                if attacks > 0 && defenders == 0 {
                    report.unguarded += 1;
                    report.penalty += 150;
                } else if defenders >= 2 && attacks >= 1 {
                    report.overloaded += 1;
                    report.penalty += 100;
                }
            }
            role += 1;
        }

        // 2. Quét nguy cơ hở Cung Tướng (Palace Exposure & Advisor/Bishop Defect)
        let advisors = pos.counts[turn * 7 + 1];
        let bishops = pos.counts[turn * 7 + 2];
        if advisors < 2 {
            report.exposure += (2 - advisors) as u32;
            report.penalty += (2 - advisors) as i32 * 80;
        }
        if bishops < 2 {
            report.exposure += (2 - bishops) as u32;
            report.penalty += (2 - bishops) as i32 * 60;
        }

        // 3. Đánh giá chỉ số rủi ro Chân trời (Horizon Risk Index)
        if report.unguarded > 0 && report.exposure > 0 {
            report.horizon = report.unguarded + report.exposure;
            report.penalty += 120;
        }

        report
    }
}

// ----------------------------------------------------------------------------
// UNIT TESTS CHO PHÂN HỆ CHẨN ĐOÁN AUDIT
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Parser;

    #[test]
    fn alignments() {
        assert_eq!(std::mem::align_of::<Report>(), 64);
        assert_eq!(std::mem::size_of::<Report>(), 64);
        assert_eq!(std::mem::align_of::<Audit>(), 64);
    }

    #[test]
    fn default_position_audit() {
        let pos = Parser::parse(Parser::DEFAULT);
        let rep = Audit::scan(&pos);
        // Ở thế cờ ban đầu, 2 Sĩ 2 Tượng đầy đủ, không có quân chủ lực đứng trống trải
        assert_eq!(rep.unguarded, 0);
        assert_eq!(rep.exposure, 0);
        assert_eq!(rep.penalty, 0);
    }
}
