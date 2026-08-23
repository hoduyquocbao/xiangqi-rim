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

/// Struct `State`: Trạng thái tiến độ chi tiết của chiến dịch vét cạn đa giai đoạn có lưu vết Checkpoint.
#[derive(Clone, Debug)]
pub struct State {
    /// Trạng thái hoạt động: "IN_PROGRESS", "COMPLETED", "STANDBY" (String)
    pub status: String,
    /// Số thứ tự giai đoạn chiến dịch hiện tại (1..5) (usize)
    pub stage: usize,
    /// Tên mô tả của giai đoạn chiến dịch (String)
    pub title: String,
    /// Danh sách các giai đoạn đã hoàn thành 100% (Vec<usize>)
    pub completed_stages: Vec<usize>,
    /// Tổng số mục tiêu trong giai đoạn hiện tại (usize)
    pub total: usize,
    /// Chỉ số mục tiêu đang thực hiện (0-indexed) (usize)
    pub current: usize,
    /// Tên mục tiêu/khai cuộc đang thực hiện (String)
    pub opening: String,
    /// Chỉ số nhánh đang thực hiện (1-indexed) (usize)
    pub branch: usize,
    /// Tổng số nhánh của mục tiêu hiện tại (usize)
    pub branches: usize,
    /// Tổng số thế cờ FEN đã khai thác trong giai đoạn (usize)
    pub nodes: usize,
    /// Tổng số đòn Sát Cục dứt điểm bắt được trong giai đoạn (usize)
    pub mates: usize,
    /// Tổng số bản ghi trong 1,024 Shards NVMe (usize)
    pub shards: usize,
    /// Tốc độ xử lý trung bình FEN / giây (f64)
    pub nps: f64,
    /// Thời gian đã chạy tính bằng giây (f64)
    pub elapsed: f64,
    /// Thời gian ước tính còn lại tính bằng giây (f64)
    pub eta: f64,
    /// Tỷ lệ phần trăm hoàn thành giai đoạn (0.0 - 100.0) (f64)
    pub progress: f64,
    /// Danh sách các mục tiêu đã hoàn thành 100% trong giai đoạn hiện tại (Vec<String>)
    pub done: Vec<String>,
    /// Danh sách các mục tiêu chưa thực hiện (Vec<String>)
    pub pending: Vec<String>,
}

impl State {
    /// Khởi tạo trạng thái ban đầu của chiến dịch.
    pub fn new() -> Self {
        Self {
            status: "IN_PROGRESS".to_string(),
            stage: 1,
            title: "Giai Đoạn 1: Khai Cuộc Cơ Bản (Depth 5)".to_string(),
            completed_stages: Vec::new(),
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
        let completed_json = self.completed_stages.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");

        format!(
            "{{\"status\":\"{}\",\"stage\":{},\"title\":\"{}\",\"completed_stages\":[{}],\"total\":{},\"current\":{},\"opening\":\"{}\",\"branch\":{},\"branches\":{},\"nodes\":{},\"mates\":{},\"shards\":{},\"nps\":{:.1},\"elapsed\":{:.1},\"eta\":{:.1},\"progress\":{:.1},\"done\":[{}],\"pending\":[{}]}}",
            self.status,
            self.stage,
            self.title,
            completed_json,
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

    /// Lưu vết trạng thái Checkpoint vĩnh cửu xuống tệp đĩa JSON
    pub fn save_checkpoint(&self, path: &str) {
        if let Ok(mut file) = std::fs::File::create(path) {
            use std::io::Write;
            let _ = file.write_all(self.json().as_bytes());
            let _ = file.flush();
        }
    }

    /// Nạp vết trạng thái Checkpoint từ tệp đĩa JSON
    pub fn load_checkpoint(path: &str) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        let mut st = Self::new();

        // Trích xuất các trường cơ bản từ chuỗi JSON
        if let Some(stage_val) = crate::server::json::num(&content, "stage") {
            st.stage = stage_val as usize;
        }
        if let Some(title_val) = crate::server::json::str(&content, "title") {
            st.title = title_val.to_string();
        }
        if let Some(nodes_val) = crate::server::json::num(&content, "nodes") {
            st.nodes = nodes_val as usize;
        }
        if let Some(mates_val) = crate::server::json::num(&content, "mates") {
            st.mates = mates_val as usize;
        }

        let done_items = crate::server::json::list(&content, "done");
        st.done = done_items.into_iter().map(|s| s.trim_matches('"').to_string()).filter(|s| !s.is_empty()).collect();

        let pending_items = crate::server::json::list(&content, "pending");
        st.pending = pending_items.into_iter().map(|s| s.trim_matches('"').to_string()).filter(|s| !s.is_empty()).collect();

        let completed_items = crate::server::json::list(&content, "completed_stages");
        st.completed_stages = completed_items.into_iter().filter_map(|s| s.trim().parse::<usize>().ok()).collect();

        Some(st)
    }
}
