// ============================================================================
// MODULE SYSTEM: HỆ SINH THÁI DATA-ORIENTED ECS SYSTEMS & DATA-DRIVEN PROFILES
// ============================================================================
// `system` đóng gói toàn bộ các hệ thống đánh giá và quyết định độc lập:
// - `weights`: Bảng siêu tham số căn lề 64-byte.
// - `king`: Hệ thống an toàn Cung Tướng.
// - `knight`: Hệ thống vận Mã chiến thuật.
// - `rook`: Hệ thống kiểm soát lộ Xe.
// - `pawn`: Hệ thống Tốt qua sông & tàn cuộc.
// - `trap`: Hệ thống bẫy cờ & phong tỏa không gian.
// - `endgame`: Hệ thống tàn cuộc lý thuyết.
// - `order`: Hệ thống sắp xếp nước đi Move Ordering.
// - `prune`: Hệ thống cắt tỉa tìm kiếm PVS.
// - `time`: Hệ thống cấp phát thời gian thích ứng động.
// ============================================================================

pub mod combo;
pub mod counter;
pub mod crdt;
pub mod endgame;
pub mod king;
pub mod knight;
pub mod order;
pub mod pawn;
pub mod prune;
pub mod rook;
pub mod skill;
pub mod time;
pub mod trap;
pub mod vault;
pub mod weights;

pub use combo::Combo;
pub use counter::Counter;
pub use crdt::{PnCounter, Record as CrdtRecord};
pub use endgame::EndgameSystem;
pub use king::KingSystem;
pub use knight::KnightSystem;
pub use order::OrderSystem;
pub use pawn::PawnSystem;
pub use prune::PruneSystem;
pub use rook::RookSystem;
pub use skill::Skill;
pub use time::TimeSystem;
pub use trap::TrapSystem;
pub use vault::{Entry, Shard, Vault};
pub use weights::{
    EndgameWeights, KingWeights, KnightWeights, OrderWeights, PawnWeights, PruneWeights,
    RookWeights, TimeWeights, TrapWeights, Weights,
};
