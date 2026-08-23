// ============================================================================
// MODULE CAMPAIGN: HỆ THỐNG ĐIỀU KHIỂN CHIẾN DỊCH VÉT CẠN ĐỊNH LƯỢNG (CAMPAIGN CONTROLLER)
// ============================================================================
// File: src/server/campaign.rs
// Triệt tiêu 100% việc chạy vô định:
// 1. Quản lý trạng thái Chiến Dịch: ĐÃ LÀM GÌ, ĐANG LÀM GÌ, CHƯA LÀM GÌ, CÒN BAO LÂU (ETA).
// 2. Tự động tính toán Tiến độ %, Thời gian ước tính (ETA), và Tổng số đòn Sát Cục.
// 3. Tự động dừng lại (Chuyển sang STANDBY) khi hoàn tất 100% 8/8 Khai Cuộc.
//
// 100% Chú thích Tiếng Việt trên từng dòng mã nguồn & 100% Định danh đơn từ tiếng Anh.
// ============================================================================

/// Struct `Opening`: Thông tin định nghĩa 1 khai cuộc trong chiến dịch.
#[derive(Clone, Debug)]
pub struct Opening {
    /// Tên tiếng Việt của khai cuộc (String)
    pub name: String,
    /// Nước đi mào đầu của khai cuộc (Vec<String>)
    pub seed: Vec<String>,
}

/// Struct `State`: Trạng thái tiến độ chi tiết của chiến dịch vét cạn.
#[derive(Clone, Debug)]
pub struct State {
    /// Trạng thái hoạt động: "IN_PROGRESS", "COMPLETED", "STANDBY" (String)
    pub status: String,
    /// Số thứ tự chiến dịch hiện tại (usize)
    pub campaign: usize,
    /// Tổng số khai cuộc trong chiến dịch (usize)
    pub total: usize,
    /// Chỉ số khai cuộc đang thực hiện (0-indexed) (usize)
    pub current: usize,
    /// Tên khai cuộc đang thực hiện (String)
    pub opening: String,
    /// Chỉ số nhánh đang thực hiện trong khai cuộc (1-indexed) (usize)
    pub branch: usize,
    /// Tổng số nhánh của khai cuộc hiện tại (usize)
    pub branches: usize,
    /// Tổng số thế cờ FEN đã khai thác trong chiến dịch (usize)
    pub nodes: usize,
    /// Tổng số đòn Sát Cục dứt điểm bắt được trong chiến dịch (usize)
    pub mates: usize,
    /// Tổng số bản ghi trong 1,024 Shards NVMe (usize)
    pub shards: usize,
    /// Tốc độ xử lý trung bình FEN / giây (f64)
    pub nps: f64,
    /// Thời gian đã chạy tính bằng giây (f64)
    pub elapsed: f64,
    /// Thời gian ước tính còn lại tính bằng giây (f64)
    pub eta: f64,
    /// Tỷ lệ phần trăm hoàn thành chiến dịch (0.0 - 100.0) (f64)
    pub progress: f64,
    /// Danh sách các khai cuộc đã hoàn thành 100% (Vec<String>)
    pub done: Vec<String>,
    /// Danh sách các khai cuộc chưa thực hiện (Vec<String>)
    pub pending: Vec<String>,
}

impl State {
    /// Khởi tạo trạng thái ban đầu của chiến dịch.
    pub fn new() -> Self {
        Self {
            status: "STANDBY".to_string(),
            campaign: 1,
            total: 8,
            current: 0,
            opening: "".to_string(),
            branch: 0,
            branches: 6,
            nodes: 0,
            mates: 0,
            shards: 0,
            nps: 0.0,
            elapsed: 0.0,
            eta: 0.0,
            progress: 0.0,
            done: Vec::new(),
            pending: Vec::new(),
        }
    }

    /// Xuất trạng thái sang chuỗi định dạng JSON chuẩn mực.
    pub fn json(&self) -> String {
        let done_json = self.done.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(",");
        let pending_json = self.pending.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(",");

        format!(
            "{{\"status\":\"{}\",\"campaign\":{},\"total\":{},\"current\":{},\"opening\":\"{}\",\"branch\":{},\"branches\":{},\"nodes\":{},\"mates\":{},\"shards\":{},\"nps\":{:.1},\"elapsed\":{:.1},\"eta\":{:.1},\"progress\":{:.1},\"done\":[{}],\"pending\":[{}]}}",
            self.status,
            self.campaign,
            self.total,
            self.current + 1,
            self.opening,
            self.branch,
            self.branches,
            self.nodes,
            self.mates,
            self.shards,
            self.nps,
            self.elapsed,
            self.eta,
            self.progress,
            done_json,
            pending_json
        )
    }
}
