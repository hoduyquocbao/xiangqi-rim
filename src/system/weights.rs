// ============================================================================
// MODULE WEIGHTS: BẢNG THÔNG SỐ ĐÁNH GIÁ & CẮT TỈA CĂN LỀ 64-BYTE (WEIGHTS SYSTEM)
// ============================================================================
// `Weights` lưu trữ toàn bộ các siêu tham số định lượng của động cơ cờ Tướng:
// - Triệt tiêu 100% hardcoded magic numbers trong mã nguồn.
// - Hỗ trợ nạp động từ tệp cấu hình JSON hoặc dùng bản mặc định `Weights::grandmaster()`.
// - Căn lề 64-byte `#[repr(C, align(64))]` tối ưu L1 Cache Line không bị False Sharing.
// ============================================================================

/// Struct `KingWeights` chứa các trọng số đánh giá an toàn Cung Tướng
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct KingWeights {
    /// Phạt Tướng lên tầng 2 trung cuộc
    pub floor_two_mg: i32,
    /// Phạt Tướng lên tầng 2 tàn cuộc
    pub floor_two_eg: i32,
    /// Phạt Tướng lên tầng 3 trung cuộc
    pub floor_three_mg: i32,
    /// Phạt Tướng lên tầng 3 tàn cuộc
    pub floor_three_eg: i32,
    /// Giá trị bảo tồn Sĩ trung cuộc
    pub advisor_mg: i32,
    /// Giá trị bảo tồn Sĩ tàn cuộc
    pub advisor_eg: i32,
    /// Giá trị bảo tồn Tượng trung cuộc
    pub bishop_mg: i32,
    /// Giá trị bảo tồn Tượng tàn cuộc
    pub bishop_eg: i32,
    /// Thưởng khi đối phương khuyết 1 Sĩ trung cuộc
    pub advisor_lost_one_mg: i32,
    /// Thưởng khi đối phương khuyết 1 Sĩ tàn cuộc
    pub advisor_lost_one_eg: i32,
    /// Thưởng khi đối phương mất hết 2 Sĩ trung cuộc
    pub advisor_lost_all_mg: i32,
    /// Thưởng khi đối phương mất hết 2 Sĩ tàn cuộc
    pub advisor_lost_all_eg: i32,
    /// Thưởng khi đối phương khuyết 1 Tượng trung cuộc
    pub bishop_lost_one_mg: i32,
    /// Thưởng khi đối phương khuyết 1 Tượng tàn cuộc
    pub bishop_lost_one_eg: i32,
    /// Thưởng khi đối phương mất hết 2 Tượng trung cuộc
    pub bishop_lost_all_mg: i32,
    /// Thưởng khi đối phương mất hết 2 Tượng tàn cuộc
    pub bishop_lost_all_eg: i32,
    /// Thưởng khi Tướng đối phương bị giam cầm (Confinement) trung cuộc
    pub confinement_mg: i32,
    /// Thưởng khi Tướng đối phương bị giam cầm (Confinement) tàn cuộc
    pub confinement_eg: i32,
    /// Thưởng hình cờ sát cục Mating Net trung cuộc
    pub mating_net_mg: i32,
    /// Thưởng hình cờ sát cục Mating Net tàn cuộc
    pub mating_net_eg: i32,
}

/// Struct `KnightWeights` chứa các trọng số đánh giá Mã
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct KnightWeights {
    /// Phạt Mã dạt biên trung cuộc
    pub edge_mg: i32,
    /// Phạt Mã dạt biên tàn cuộc
    pub edge_eg: i32,
    /// Phạt Mã bị cản chân hoàn toàn trung cuộc
    pub pinned_mg: i32,
    /// Phạt Mã bị cản chân hoàn toàn tàn cuộc
    pub pinned_eg: i32,
    /// Phạt Mã bị hạn chế đường đi trung cuộc
    pub restricted_mg: i32,
    /// Phạt Mã bị hạn chế đường đi tàn cuộc
    pub restricted_eg: i32,
    /// Thưởng Mã qua sông trung cuộc
    pub river_mg: i32,
    /// Thưởng Mã qua sông tàn cuộc
    pub river_eg: i32,
    /// Thưởng Mã tiền đồn Ngọa Tào trung cuộc
    pub outpost_mg: i32,
    /// Thưởng Mã tiền đồn Ngọa Tào tàn cuộc
    pub outpost_eg: i32,
}

/// Struct `RookWeights` chứa các trọng số đánh giá Xe
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct RookWeights {
    /// Thưởng Xe chiếm lộ mở trung cuộc
    pub open_mg: i32,
    /// Thưởng Xe chiếm lộ mở tàn cuộc
    pub open_eg: i32,
    /// Thưởng Xe tuần hà trung cuộc
    pub patrol_mg: i32,
    /// Thưởng Xe tuần hà tàn cuộc
    pub patrol_eg: i32,
    /// Thưởng Xe chiếm sườn Cung trung cuộc
    pub flank_mg: i32,
    /// Thưởng Xe chiếm sườn Cung tàn cuộc
    pub flank_eg: i32,
    /// Thưởng sắp xếp nước đi Xe chiếm trung lộ
    pub order_open: i32,
    /// Thưởng sắp xếp nước đi Xe chiếm sườn
    pub order_flank: i32,
    /// Phạt sắp xếp nước đi Xe dạt biên ăn Tốt rác
    pub order_edge_penalty: i32,
    /// Phạt đổi Xe khi đang có ưu thế thế trận trung cuộc
    pub trade_penalty_mg: i32,
    /// Phạt đổi Xe khi đang có ưu thế thế trận tàn cuộc
    pub trade_penalty_eg: i32,
    /// Thưởng Xe kết hợp Tốt đè Cung Tướng trung cuộc
    pub palace_press_mg: i32,
    /// Thưởng Xe kết hợp Tốt đè Cung Tướng tàn cuộc
    pub palace_press_eg: i32,
}

/// Struct `PawnWeights` chứa các trọng số đánh giá Tốt
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct PawnWeights {
    /// Thưởng Tốt qua sông trung cuộc
    pub river_mg: i32,
    /// Thưởng Tốt qua sông tàn cuộc
    pub river_eg: i32,
    /// Thưởng Tốt tiến sâu vào hàng 3/4 trung cuộc
    pub deep_mg: i32,
    /// Thưởng Tốt tiến sâu vào hàng 3/4 tàn cuộc
    pub deep_eg: i32,
    /// Thưởng Tốt qua sông có Xe/Pháo/Mã yểm trợ trung cuộc
    pub escorted_mg: i32,
    /// Thưởng Tốt qua sông có Xe/Pháo/Mã yểm trợ tàn cuộc
    pub escorted_eg: i32,
    /// Phạt để Tốt đối phương tự do tiến sâu không có quân chốt chặn trung cuộc
    pub enemy_unblocked_mg: i32,
    /// Phạt để Tốt đối phương tự do tiến sâu không có quân chốt chặn tàn cuộc
    pub enemy_unblocked_eg: i32,
    /// Thưởng Tốt lọt vào Cung Tướng (d2/e2/f2 hoặc d7/e7/f7) trung cuộc
    pub palace_breach_mg: i32,
    /// Thưởng Tốt lọt vào Cung Tướng (d2/e2/f2 hoặc d7/e7/f7) tàn cuộc
    pub palace_breach_eg: i32,
}

/// Struct `TrapWeights` chứa các trọng số đánh giá bẫy và phong tỏa không gian
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct TrapWeights {
    /// Phạt Mã bị nghẽn chân nhốt trong góc trung cuộc
    pub trapped_knight_mg: i32,
    /// Phạt Mã bị nghẽn chân nhốt trong góc tàn cuộc
    pub trapped_knight_eg: i32,
    /// Phạt Xe bị nhốt góc không đường ra trung cuộc
    pub trapped_rook_mg: i32,
    /// Phạt Xe bị nhốt góc không đường ra tàn cuộc
    pub trapped_rook_eg: i32,
    /// Phạt Pháo mất ngòi cơ động trung cuộc
    pub trapped_cannon_mg: i32,
    /// Phạt Pháo mất ngòi cơ động tàn cuộc
    pub trapped_cannon_eg: i32,
    /// Hệ số phạt thắt chặt không gian
    pub constriction_penalty: i32,
}

/// Struct `EndgameWeights` chứa các trọng số đánh giá tàn cuộc lý thuyết
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct EndgameWeights {
    /// Điểm số thắng thế tàn cuộc áp đảo
    pub win_score: i32,
    /// Điểm số hòa cân bằng tàn cuộc
    pub draw_score: i32,
    /// Điểm số thua thế tàn cuộc
    pub loss_score: i32,
    /// Thưởng Đơn Mã thắng Đơn Sĩ
    pub knight_vs_advisor: i32,
    /// Thưởng Xe Mã thắng Xe Sĩ Tượng
    pub rook_knight_vs_rook_guards: i32,
    /// Thưởng Song Pháo thắng Khuyết Sĩ Tượng
    pub double_cannons_vs_broken: i32,
}

/// Struct `OrderWeights` chứa các siêu tham số sắp xếp nước đi Move Ordering
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct OrderWeights {
    /// Điểm thưởng nước đi sát thủ Killer Move 1
    pub killer_one: i32,
    /// Điểm thưởng nước đi sát thủ Killer Move 2
    pub killer_two: i32,
    /// Điểm thưởng nước đi phản kích Counter Move
    pub counter_move: i32,
    /// Hệ số nhân MVV-LVA cho nước ăn quân
    pub mvv_lva_scale: i32,
}

/// Struct `PruneWeights` chứa các siêu tham số cắt tỉa tìm kiếm PVS
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct PruneWeights {
    /// Biên độ Reverse Futility Pruning cơ bản
    pub rfp_base: i32,
    /// Biên độ Null Move Pruning reduction cơ bản
    pub nmp_base: i32,
    /// Biên độ Singular Extensions beta
    pub singular_margin: i32,
    /// Biên độ ProbCut beta cutoff
    pub probcut_margin: i32,
}

/// Struct `TimeWeights` chứa các tham số cấp phát thời gian
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct TimeWeights {
    /// Thời gian cơ bản mỗi nước đi (mili-giây)
    pub base_ms: u64,
    /// Thời gian tối thiểu mỗi nước đi (mili-giây)
    pub min_ms: u64,
    /// Thời gian tối đa mỗi nước đi (mili-giây)
    pub max_ms: u64,
    /// Hệ số nhân thời gian khi đang bị chiếu
    pub check_mult: f64,
    /// Hệ số nhân thời gian khi có biến động chiến thuật lớn
    pub tactical_mult: f64,
    /// Hệ số nhân thời gian trong giai đoạn tàn cuộc sâu
    pub endgame_mult: f64,
}

/// Struct `Weights` tập hợp toàn bộ các bảng trọng số hệ thống, căn lề 64-byte
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct Weights {
    /// Trọng số An toàn Cung Tướng
    pub king: KingWeights,
    /// Trọng số Mã
    pub knight: KnightWeights,
    /// Trọng số Xe
    pub rook: RookWeights,
    /// Trọng số Tốt
    pub pawn: PawnWeights,
    /// Trọng số Bẫy cờ & Không gian
    pub trap: TrapWeights,
    /// Trọng số Tàn cuộc lý thuyết
    pub endgame: EndgameWeights,
    /// Trọng số Sắp xếp nước đi
    pub order: OrderWeights,
    /// Trọng số Cắt tỉa tìm kiếm
    pub prune: PruneWeights,
    /// Trọng số Quản lý thời gian
    pub time: TimeWeights,
}

impl Default for Weights {
    fn default() -> Self {
        Self::grandmaster()
    }
}

impl Weights {
    /// Khởi tạo bộ trọng số chuẩn Grandmaster tối ưu hóa sẵn trong RAM
    pub const fn grandmaster() -> Self {
        Self {
            king: KingWeights {
                floor_two_mg: -450,
                floor_two_eg: -900,
                floor_three_mg: -850,
                floor_three_eg: -1800,
                advisor_mg: 110,
                advisor_eg: 140,
                bishop_mg: 110,
                bishop_eg: 160,
                advisor_lost_one_mg: 140,
                advisor_lost_one_eg: 220,
                advisor_lost_all_mg: 300,
                advisor_lost_all_eg: 450,
                bishop_lost_one_mg: 120,
                bishop_lost_one_eg: 180,
                bishop_lost_all_mg: 260,
                bishop_lost_all_eg: 380,
                confinement_mg: 350,
                confinement_eg: 750,
                mating_net_mg: 550,
                mating_net_eg: 1100,
            },
            knight: KnightWeights {
                edge_mg: -150,
                edge_eg: -200,
                pinned_mg: -80,
                pinned_eg: -130,
                restricted_mg: -35,
                restricted_eg: -55,
                river_mg: 70,
                river_eg: 110,
                outpost_mg: 160,
                outpost_eg: 260,
            },
            rook: RookWeights {
                open_mg: 25,
                open_eg: 35,
                patrol_mg: 20,
                patrol_eg: 25,
                flank_mg: 15,
                flank_eg: 20,
                order_open: 25000,
                order_flank: 12000,
                order_edge_penalty: -30000,
                trade_penalty_mg: -250,
                trade_penalty_eg: -450,
                palace_press_mg: 180,
                palace_press_eg: 350,
            },
            pawn: PawnWeights {
                river_mg: 50,
                river_eg: 90,
                deep_mg: 90,
                deep_eg: 160,
                escorted_mg: 120,
                escorted_eg: 200,
                enemy_unblocked_mg: -100,
                enemy_unblocked_eg: -180,
                palace_breach_mg: 550,
                palace_breach_eg: 1200,
            },
            trap: TrapWeights {
                trapped_knight_mg: -250,
                trapped_knight_eg: -350,
                trapped_rook_mg: -300,
                trapped_rook_eg: -400,
                trapped_cannon_mg: -180,
                trapped_cannon_eg: -240,
                constriction_penalty: -35,
            },
            endgame: EndgameWeights {
                win_score: 4000,
                draw_score: 0,
                loss_score: -4000,
                knight_vs_advisor: 3800,
                rook_knight_vs_rook_guards: 3900,
                double_cannons_vs_broken: 4000,
            },
            order: OrderWeights {
                killer_one: 900_000,
                killer_two: 800_000,
                counter_move: 700_000,
                mvv_lva_scale: 100,
            },
            prune: PruneWeights {
                rfp_base: 150,
                nmp_base: 3,
                singular_margin: 200,
                probcut_margin: 200,
            },
            time: TimeWeights {
                base_ms: 3500,
                min_ms: 1500,
                max_ms: 12000,
                check_mult: 1.5,
                tactical_mult: 1.8,
                endgame_mult: 2.2,
            },
        }
    }
}
