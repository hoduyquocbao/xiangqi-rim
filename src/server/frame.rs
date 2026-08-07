// ============================================================================
// MODULE FRAME: PHÂN TÍCH VÀ ĐÓNG GÓI KHUNG TIN WEBSOCKET (RFC 6455)
// ============================================================================
// Triển khai cấu trúc và phương thức phân tích/đóng gói khung WebSocket.
// Giải mã 4-byte mask từ client và đóng gói text frame truyền về client.
// Tuân thủ 100% quy tắc từ đơn tiếng Anh cho định danh và chú thích tiếng Việt.
// ============================================================================

use std::io::Read;
use std::net::TcpStream;

/// Enum `Opcode` định nghĩa mã thao tác Opcode của khung tin WebSocket RFC 6455
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    /// Khung tin tiếp diễn Continuation 0x0
    Cont = 0x0,
    /// Khung tin văn bản Text 0x1
    Text = 0x1,
    /// Khung tin nhị phân Binary 0x2
    Binary = 0x2,
    /// Khung tin đóng kết nối Close 0x8
    Close = 0x8,
    /// Khung tin kiểm tra Ping 0x9
    Ping = 0x9,
    /// Khung tin phản hồi Pong 0xA
    Pong = 0xA,
}

impl Opcode {
    /// Chuyển đổi giá trị byte nguyên 4-bit thành Opcode
    pub fn parse(byte: u8) -> Option<Self> {
        match byte & 0x0F {
            0x0 => Some(Self::Cont),
            0x1 => Some(Self::Text),
            0x2 => Some(Self::Binary),
            0x8 => Some(Self::Close),
            0x9 => Some(Self::Ping),
            0xA => Some(Self::Pong),
            _ => None,
        }
    }
}

/// Struct `Frame` biểu diễn một khung tin WebSocket RFC 6455
#[derive(Debug, Clone)]
#[repr(C, align(64))]
pub struct Frame {
    /// Cờ fin đánh dấu khung cuối cùng (bit 7 byte 0)
    pub fin: bool,
    /// Mã thao tác opcode
    pub opcode: Opcode,
    /// Khóa giải mã mask 4 byte (nếu client gửi)
    pub mask: Option<[u8; 4]>,
    /// Dải byte dữ liệu payload đã được giải mã
    pub payload: Vec<u8>,
}

impl Frame {
    /// Đọc và phân tích một khung WebSocket từ luồng TCP stream
    pub fn parse(stream: &mut TcpStream) -> Result<Self, String> {
        // Đọc 2 byte header head đầu tiên từ stream
        let mut head = [0u8; 2];
        stream.read_exact(&mut head).map_err(|e| e.to_string())?;

        // Extract cờ fin bit 7 của byte 0
        let fin = (head[0] & 0x80) != 0;

        // Extract opcode từ 4 bit thấp của byte 0
        let opcode = Opcode::parse(head[0]).ok_or_else(|| "Opcode không hợp lệ".to_string())?;

        // Extract cờ masked bit 7 của byte 1
        let masked = (head[1] & 0x80) != 0;

        // Extract độ dài ban đầu len từ 7 bit thấp của byte 1
        let mut len = (head[1] & 0x7F) as u64;

        // Nếu len = 126, đọc tiếp 2 byte độ dài u16 Big-Endian
        if len == 126 {
            let mut buf = [0u8; 2];
            stream.read_exact(&mut buf).map_err(|e| e.to_string())?;
            len = u16::from_be_bytes(buf) as u64;
        } else if len == 127 {
            // Nếu len = 127, đọc tiếp 8 byte độ dài u64 Big-Endian
            let mut buf = [0u8; 8];
            stream.read_exact(&mut buf).map_err(|e| e.to_string())?;
            len = u64::from_be_bytes(buf);
        }

        // Đọc 4 byte Masking Key nếu cờ masked bật
        let mask = if masked {
            let mut key = [0u8; 4];
            stream.read_exact(&mut key).map_err(|e| e.to_string())?;
            Some(key)
        } else {
            None
        };

        // Đọc len byte dữ liệu payload
        let mut payload = vec![0u8; len as usize];
        if len > 0 {
            stream.read_exact(&mut payload).map_err(|e| e.to_string())?;
        }

        // Giải mã dữ liệu payload với mask key nếu có
        if let Some(key) = mask {
            for i in 0..payload.len() {
                payload[i] ^= key[i % 4];
            }
        }

        // Trả về Frame đã phân tích hoàn chỉnh
        Ok(Self {
            fin,
            opcode,
            mask,
            payload,
        })
    }

    /// Đóng gói một văn bản text chuỗi UTF-8 thành khung tin WebSocket Server-to-Client
    pub fn text(data: &str) -> Vec<u8> {
        let bytes = data.as_bytes();
        let len = bytes.len();
        let mut frame = Vec::new();

        // Byte 0: FIN = 1 (0x80) | Opcode::Text (0x01) = 0x81
        frame.push(0x81);

        // Byte 1 và bổ sung độ dài len (Server gửi không mask bit)
        if len <= 125 {
            frame.push(len as u8);
        } else if len <= 65535 {
            frame.push(126);
            let b = (len as u16).to_be_bytes();
            frame.extend_from_slice(&b);
        } else {
            frame.push(127);
            let b = (len as u64).to_be_bytes();
            frame.extend_from_slice(&b);
        }

        // Nối toàn bộ byte dữ liệu text payload
        frame.extend_from_slice(bytes);
        frame
    }

    /// Đóng gói khung tin ngắt kết nối Close frame
    pub fn close() -> Vec<u8> {
        // Byte 0: FIN = 1 | Opcode::Close (0x08) = 0x88, Byte 1: Len = 0
        vec![0x88, 0x00]
    }
}
