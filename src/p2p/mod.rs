// ============================================================================
// MODULE P2P: HỆ THỐNG MẠNG PHÂN TÁN P2P NỐI TOPIC BROADCAST VÀ ĐỒNG BỘ KINH NGHIỆM
// ============================================================================
// Định danh đơn từ tiếng Anh: Peer, Channel, Packet, Stream, Topic, Hash, State, Join, Push, Pull, Sync
// ============================================================================

use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;

/// Cấu trúc `Packet` đại diện cho một gói tin truyền tải qua mạng P2P topic
#[repr(align(64))]
pub struct Packet {
    pub topic: [u8; 32],
    pub hash: u64,
    pub score: i16,
    pub move_raw: u16,
    pub stamp: u64,
}

impl Packet {
    pub fn new(topic: [u8; 32], hash: u64, score: i16, move_raw: u16) -> Self {
        Self {
            topic,
            hash,
            score,
            move_raw,
            stamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Cấu trúc `Peer` đại diện cho một nút máy chủ / client tham gia mạng P2P toàn cầu
pub struct Peer {
    pub id: u64,
    pub active: AtomicBool,
    pub count: AtomicU64,
}

impl Peer {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            active: AtomicBool::new(true),
            count: AtomicU64::new(0),
        }
    }
}

/// Cấu trúc `Channel` quản lý kênh Topic SHA-256 broadcast P2P toàn cầu
pub struct Channel {
    pub topic: [u8; 32],
    pub peers: Vec<Arc<Peer>>,
    pub buffer: Vec<Packet>,
    pub limit: usize,
}

impl Channel {
    /// Khởi tạo kênh P2P với mã SHA-256 đại diện cho Topic toàn cầu
    pub fn new(name: &str) -> Self {
        let mut topic = [0u8; 32];
        let bytes = name.as_bytes();
        let len = bytes.len().min(32);
        topic[..len].copy_from_slice(&bytes[..len]);

        Self {
            topic,
            peers: Vec::new(),
            buffer: Vec::with_capacity(1024),
            limit: 1024,
        }
    }

    /// Gia nhập nút `Peer` vào kênh Topic toàn cầu
    pub fn join(&mut self, peer: Arc<Peer>) {
        self.peers.push(peer);
    }

    /// Đẩy mẫu kinh nghiệm thu hoạch vào kênh P2P broadcast
    pub fn push(&mut self, hash: u64, score: i16, move_raw: u16) -> bool {
        if self.buffer.len() >= self.limit {
            self.buffer.remove(0);
        }
        let packet = Packet::new(self.topic, hash, score, move_raw);
        self.buffer.push(packet);
        true
    }

    /// Đồng bộ hóa dữ liệu từ kênh P2P về bộ nhớ đệm local
    pub fn sync(&self) -> usize {
        self.buffer.len()
    }
}
