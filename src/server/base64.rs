// ============================================================================
// MODULE BASE64: MÃ HÓA MẢNG BYTE SANG CHUỖI VĂN BẢN BASE64 (RFC 4648)
// ============================================================================
// Triển khai mã hóa Base64 thuần Rust std-only không phụ thuộc crate ngoài.
// Phục vụ tạo chuỗi `Sec-WebSocket-Accept` từ băm SHA-1 20 bytes cho WebSocket.
// Tuân thủ 100% quy tắc từ đơn tiếng Anh cho định danh và chú thích tiếng Việt.
// ============================================================================

/// Hằng số TABLE chứa 64 ký tự MIME chuẩn cho mã hóa Base64
const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Struct `Base64` cung cấp các phương thức mã hóa Base64 std-only
pub struct Base64;

impl Base64 {
    /// Mã hóa dải byte đầu vào `data` thành chuỗi văn bản Base64
    pub fn encode(data: &[u8]) -> String {
        // Trả về chuỗi rỗng nếu dải dữ liệu đầu vào data rỗng
        if data.is_empty() {
            return String::new();
        }

        // Khai báo mảng byte kết quả out với dung lượng ước tính
        let len = data.len();
        let mut out = Vec::with_capacity((len + 2) / 3 * 4);

        // Duyệt qua từng nhóm 3 byte trong dải dữ liệu data
        let mut i = 0;
        while i + 2 < len {
            // Đọc 3 byte b0, b1, b2
            let b0 = data[i] as u32;
            let b1 = data[i + 1] as u32;
            let b2 = data[i + 2] as u32;

            // Gộp 3 byte thành giá trị 24-bit val
            let val = (b0 << 16) | (b1 << 8) | b2;

            // Trích xuất 4 chỉ số 6-bit c0, c1, c2, c3
            let c0 = ((val >> 18) & 0x3F) as usize;
            let c1 = ((val >> 12) & 0x3F) as usize;
            let c2 = ((val >> 6) & 0x3F) as usize;
            let c3 = (val & 0x3F) as usize;

            // Tra bảng TABLE và thêm 4 ký tự vào danh sách kết quả out
            out.push(TABLE[c0]);
            out.push(TABLE[c1]);
            out.push(TABLE[c2]);
            out.push(TABLE[c3]);

            i += 3;
        }

        // Xử lý các byte dư rem còn lại ở cuối dữ liệu
        let rem = len - i;
        if rem == 1 {
            let b0 = data[i] as u32;
            let val = b0 << 16;
            let c0 = ((val >> 18) & 0x3F) as usize;
            let c1 = ((val >> 12) & 0x3F) as usize;

            out.push(TABLE[c0]);
            out.push(TABLE[c1]);
            out.push(b'=');
            out.push(b'=');
        } else if rem == 2 {
            let b0 = data[i] as u32;
            let b1 = data[i + 1] as u32;
            let val = (b0 << 16) | (b1 << 8);
            let c0 = ((val >> 18) & 0x3F) as usize;
            let c1 = ((val >> 12) & 0x3F) as usize;
            let c2 = ((val >> 6) & 0x3F) as usize;

            out.push(TABLE[c0]);
            out.push(TABLE[c1]);
            out.push(TABLE[c2]);
            out.push(b'=');
        }

        // Chuyển đổi mảng byte out thành chuỗi String hợp lệ
        String::from_utf8(out).unwrap_or_default()
    }
}
