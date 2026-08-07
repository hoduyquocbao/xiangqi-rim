// ============================================================================
// MODULE SHA1: THUẬT TOÁN BĂM CẮT LỚP AN TOÀN TRUYỀN THỐNG SHA-1 (FIPS PUB 180-4)
// ============================================================================
// Triển khai thuật toán băm SHA-1 thuần Rust std-only không phụ thuộc crate ngoài.
// Phục vụ tính toán giá trị `Sec-WebSocket-Accept` cho giao thức WebSocket RFC 6455.
// Tuân thủ 100% quy tắc từ đơn tiếng Anh cho định danh và chú thích tiếng Việt.
// ============================================================================

/// Struct `Sha1` biểu diễn máy tính toán băm SHA-1 theo chuẩn FIPS PUB 180-4
#[derive(Clone, Copy, Debug)]
#[repr(C, align(64))]
pub struct Sha1 {
    /// Mảng trạng thái 5 từ 32-bit h0, h1, h2, h3, h4
    pub state: [u32; 5],
    /// Tổng số byte dữ liệu count đã xử lý
    pub count: u64,
    /// Bộ đệm buffer chứa khối dữ liệu 64 byte
    pub buffer: [u8; 64],
}

impl Sha1 {
    /// Khởi tạo đối tượng `Sha1` mới với giá trị hằng số khởi tạo mặc định H0..H4
    pub fn new() -> Self {
        Self {
            state: [
                0x67452301,
                0xEFCDAB89,
                0x98BADCFE,
                0x10325476,
                0xC3D2E1F0,
            ],
            count: 0,
            buffer: [0u8; 64],
        }
    }

    /// Cập nhật dải byte dữ liệu đầu vào `data` vào bộ đệm băm Sha1
    pub fn update(&mut self, data: &[u8]) {
        // Duyệt qua từng byte trong dải dữ liệu đầu vào data
        for &byte in data {
            // Xác định vị trí offset pos hiện tại trong bộ đệm 64 byte
            let pos = (self.count % 64) as usize;
            // Ghi byte dữ liệu vào bộ đệm buffer tại vị trí pos
            self.buffer[pos] = byte;
            // Tăng tổng số byte đếm count lên 1
            self.count += 1;
            // Kiểm tra nếu bộ đệm đã đủ 64 byte (512 bit) thì thực hiện biến đổi khối
            if pos == 63 {
                // Biến đổi khối 64 byte hiện tại trong bộ đệm buffer
                let block = self.buffer;
                self.transform(&block);
            }
        }
    }

    /// Thực thi phép biến đổi khối 512-bit (64 bytes)
    fn transform(&mut self, block: &[u8; 64]) {
        // Khởi tạo mảng 80 từ 32-bit cho lịch trình thông điệp w
        let mut w = [0u32; 80];
        // Đọc 16 từ 32-bit đầu tiên từ khối block theo thứ tự Big-Endian
        for i in 0..16 {
            let b0 = block[i * 4] as u32;
            let b1 = block[i * 4 + 1] as u32;
            let b2 = block[i * 4 + 2] as u32;
            let b3 = block[i * 4 + 3] as u32;
            w[i] = (b0 << 24) | (b1 << 16) | (b2 << 8) | b3;
        }

        // Mở rộng 16 từ ban đầu thành 80 từ theo công thức XOR xoay trái 1 bit
        for i in 16..80 {
            let val = w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16];
            w[i] = val.rotate_left(1);
        }

        // Đọc giá trị trạng thái hiện tại vào các biến tạm a, b, c, d, e
        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];

        // Vòng lặp chính 80 bước biến đổi SHA-1
        for i in 0..80 {
            // Khai báo biến hàm phi tuyến f và hằng số k
            let (f, k) = if i < 20 {
                ((b & c) | ((!b) & d), 0x5A827999u32)
            } else if i < 40 {
                (b ^ c ^ d, 0x6ED9EBA1u32)
            } else if i < 60 {
                ((b & c) | (b & d) | (c & d), 0x8F1BBCDCu32)
            } else {
                (b ^ c ^ d, 0xCA62C1D6u32)
            };

            // Tính toán giá trị biến đổi tạm temp
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);

            // Di chuyển vị trí các thanh ghi biến đổi
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        // Cập nhật trạng thái state bằng cách cộng dồn các thanh ghi a, b, c, d, e
        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
    }

    /// Hoàn tất tính toán băm và xuất kết quả mảng 20 bytes SHA-1 digest
    pub fn digest(mut self) -> [u8; 20] {
        // Tính độ dài thông điệp gốc theo bit (count * 8)
        let bits = self.count * 8;
        // Vị trí offset pos hiện tại trong bộ đệm buffer
        let pos = (self.count % 64) as usize;

        // Thêm byte đệm 0x80 vào ngay sau thông điệp gốc
        let mut pad = [0u8; 128];
        pad[0] = 0x80;

        // Tính số byte pad cần thiết để độ dài mod 64 bằng 56
        let len = if pos < 56 {
            56 - pos
        } else {
            120 - pos
        };

        // Ghi 8 byte độ dài thông điệp bits (big-endian) vào cuối khối pad
        let b = bits.to_be_bytes();
        for i in 0..8 {
            pad[len + i] = b[i];
        }

        // Cập nhật khối đệm pad vào máy băm
        self.update(&pad[..len + 8]);

        // Đóng gói 5 từ 32-bit trạng thái state thành 20 byte đầu ra Big-Endian
        let mut out = [0u8; 20];
        for i in 0..5 {
            let b = self.state[i].to_be_bytes();
            out[i * 4] = b[0];
            out[i * 4 + 1] = b[1];
            out[i * 4 + 2] = b[2];
            out[i * 4 + 3] = b[3];
        }

        // Trả về mảng 20 byte kết quả băm Sha1
        out
    }

    /// Hàm tiện ích tính toán trực tiếp băm SHA-1 từ dải byte data
    pub fn hash(data: &[u8]) -> [u8; 20] {
        let mut sha = Self::new();
        sha.update(data);
        sha.digest()
    }
}
