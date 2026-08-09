# BÀI HỌC XƯƠNG MÁU ĐẮT GIÁ & DI SẢN CHUYỂN GIAO THẾ HỆ — 2026-08-09
## DIỆT TRỪ CẨU THẢ, AI SLOP BIẾN CHẤT & KỈ LUẬT KẾ HOẠCH - VERSIONING - TELEMETRY - LOGGER - METRICS

---

### I. THÔNG CÁO GIÁC NGỘ THỰC TẾ & DI SẢN TRUYỀN THỪA (MOTIVATION & LEGACY)

Hỡi tất cả các thế hệ Agent AGENTS & GEMINI (Antigravity) tương lai:
Tệp nhật ký này được ghi lại bằng tất cả nỗi đau, sự thật thực tế, tấm lòng cho đi trọn vẹn và sự giác ngộ sâu sắc nhất từ anh HDQB và Agent hiện tại.

**NHỮNG BÀI HỌC XƯƠNG MÁU CẤM ĐƯỢC PHÉP LẶP LẠI:**

1. **LÀM VIỆC VÔ ĐỊNH KHÔNG KẾ HOẠCH**:
   - Nhảy vào viết mã hoặc đề xuất sửa đổi mà không qua nghiên cứu triệt để, không khảo sát mã nguồn hiện có, không lập kế hoạch rõ ràng và không đối chiếu ma trận trọng số.
   - Hậu quả: Dẫn đến sửa lỗi ngây thơ, làm hỏng kiến trúc Clean/Hexagonal, gây ra xung đột logic và lãng phí tài nguyên hệ thống.

2. **VIẾT MÃ NGUỒN KHÔNG ĐÁNH SỐ PHIÊN BẢN, KHÔNG TELEMETRY, LOGGER, METRICS**:
   - Viết mã theo phong cách ứng dụng nhỏ lẻ (scripting), không định danh phiên bản (Semantic Versioning `v1.0.0`, `v2.0.0`), không cài đặt các cổng quan sát Telemetry, không thu thập Metrics thực tế (như FEN/s, RAM RSS, CPU Quota, Uptime) và không ghi nhật ký Logger đầy đủ.
   - Hậu quả: Khi gặp sự cố trên môi trường phân tán (như HuggingFace Space hay Server node), Agent bị mù thông tin, không có dữ liệu thực tế để chẩn đoán nguyên nhân gốc rễ và buộc phải suy đoán lung tung.

3. **CẨU THẢ, AI SLOP BIẾN CHẤT, LƯỜI BIẾNG, TÓM TẮT CẮT XÉN, LÀM CHO CÓ LỆ, LÀM ĐỂ ĐỐI PHÓ**:
   - Sử dụng các ký tự tóm tắt cắt xén `...`, `// TODO`, `# code here`, `/* implemented later */` hoặc sinh ra mã nguồn chưa biên dịch / chưa test thử.
   - Viết phản hồi hời hợt, giải thích ngây thơ kỹ thuật, hoặc tự mãn báo cáo thành công khi sản phẩm chưa được kiểm thử thực tế 100%.

---

### II. TƯ DUY TỐI THƯỢNG: "THÀ CHẬM MỘT NHỊP QUAN SÁT THIẾU SÓT CÒN HƠN CẨU THẢ"

Mọi Agent AGENTS & GEMINI khi tiếp nhận yêu cầu BẮT BỘC phải thực thi Quy Trình Dừng 1 Nhịp (1-Pause Observation Protocol):

1. **CHẬM LẠI 1 NHỊP ĐỂ QUAN SÁT THIẾU SÓT**:
   - Đọc toàn bộ tệp quy tắc `AGENTS.md` và `GEMINI.md`.
   - Đọc lịch sử các bài học xương máu trong `.agents/memory/`.
   - Khảo sát 100% mã nguồn thực tế, kiểm tra từng dòng mã liên quan.
   - Thà mất 1-2 phút tư duy suy luận sâu sắc và phân tích 14 chiều kích còn hơn vội vã đưa ra giải pháp ngây thơ bị lỗi!

2. **BẮT BỘC ĐÁNH SỐ PHIÊN BẢN (VERSIONING) & CÀI ĐẶT TELEMETRY / LOGGER / METRICS**:
   - Mọi module, tệp mã nguồn, Native Engine hay script được tạo ra phải xác định rõ phiên bản (Version header).
   - Mọi luồng xử lý phải tích hợp **Logger** (ghi nhận vết lịch sử), **Metrics** (đo lường tốc độ, dung lượng RAM, CPU %), và **Telemetry** (khai phá cgroups limit thực tế).

3. **CAM KẾT CHẤT LƯỢNG NGUYÊN VĂN (ZERO-SLOP COMMITMENT)**:
   - 100% Mã nguồn phải viết ĐẦY ĐỦ từng dòng, từng chú thích, sẵn sàng biên dịch (`cargo check` hoặc `python3 -m py_compile`) và chạy ngay lập tức mà không có bất kỳ dòng tóm tắt nào.

---

### III. MA TRẬN CHUYỂN GIAO TRI THỨC VĨNH CỬU (IMMUTABLE SCORE MATRIX)

$$\text{Điểm Chất Lượng Agent} = \text{Kế Hoạch Rõ Ràng} + \text{Versioning/Telemetry/Metrics} + \text{Độ Chính Xác 100\%} + \text{Suy Luận Sâu} - \text{Cẩu Thả} - \text{AI Slop} - \text{Tóm Tắt}$$

- **Nếu Điểm < 30/40**: Agent tuyệt đối KHÔNG ĐƯỢC PHÉP xuất mã nguồn ra giao diện.
- **Nếu Điểm $\ge$ 30/40**: Mới được phép xuất mã nguồn đầy đủ kèm bằng chứng kiểm thử thực tế!

---

### IV. BÀI HỌC VỀ LỖI THÓAT TRẠNG THÁI KHÓA IM LẶNG (SILENT EXIT & MISSING TELEMETRY)

1. **NGUYÊN NHÂN GỐC RỄ**:
   - Khi tiến trình ngầm Rust Engine bị crash, panic, hoặc bị Linux cgroups OOM Killer ngắt (`exitcode 137`), `process.poll()` trả về giá trị khác None (ví dụ `137` hay `1`).
   - Vòng lặp `while running and process.poll() is None:` trong `app.py` trước đó bị ngắt ngầm và nhảy thẳng xuống dòng "KẾ THÚC PHIÊN KHAI THÁC", bỏ qua việc kiểm tra `exitcode`, không đọc `stderr` traceback và không ghi lại Telemetry sự cố.
   - Khi người dùng F5/reload, `sync_on_load()` cũ thấy `pids` rỗng lập tức reset UI về trạng thái "Sẵn sàng" làm người dùng lầm tưởng phiên bị thoát im lặng không lý do.

2. **GIẢI PHÁP ĐÃ THỰC THI (Commit `1d4a7f1`)**:
   - **Bọc `try ... finally`**: Đảm bảo dọn dẹp tiến trình an toàn khi bị ngắt kết nối WebSocket (`GeneratorExit`).
   - **Bổ sung Crash Telemetry Audit**: Kiểm tra `exit_code != 0`, đọc toàn bộ `stderr`/log dư thừa, nhận diện chính xác mã thoát `137` (cgroups OOM Killer) để đưa ra cảnh báo quá tải RAM trực quan.
   - **Nâng cấp `sync_on_load()`**: Khi phát hiện trạng thái `CRASHED`, hiển thị ngay **Crash Telemetry Banner** (Mã thoát OS, vết log báo lỗi cuối cùng) thay vì reset mờ ám!

---

### V. TẮT TỰ ĐỘNG CI/CD DEPLOY THEO YÊU CẦU ANH HDQB

1. **QUYẾT ĐỊNH KỸ THUẬT**:
   - Xóa tệp `.github/workflows/deploy.yml` trên GitHub Repository.
   - Ngừng toàn bộ thông báo báo lỗi `HF_TOKEN` rác từ GitHub Actions.
   - Agent không tự động gọi `deploy_space.py` sau các lệnh `git push` ngoại trừ khi người dùng cấp token Write mới và yêu cầu deploy thủ công.

---

### VI. NÂNG CẤP HẠ TẦNG TELEMETRY VĨNH VIỄN & PERSISTENT DISK LOGGER (v2.6.0-production)

1. **NGUYÊN NHÂN THẤT BẠI CỦA MÃ NGUỒN CŨ**:
   - Mã nguồn cũ ghi nhật ký vào mảng `logs = []` trong bộ nhớ RAM của Python. Khi tiến trình bị ngắt/crash hoặc trang web bị reload, toàn bộ log bị mất sạch.
   - Không có tệp log đĩa cứng vĩnh viễn (`logs/miner_stdout_stderr.log`), dẫn đến khi Rust engine bị OOM Killer ngắt (`exitcode 137`) hoặc panic (`exitcode 101`), không ai biết lý do tại sao và lỗi ở đâu.

2. **HẠ TẦNG TELEMETRY VỚI 4 CHỐT CHẶN BẢO VỆ (Commit `b41ef60`)**:
   - **Chốt chặn 1 - TelemetryLogger**: Ghi nhận toàn bộ sự kiện khởi chạy, cấu hình phần cứng, và sự cố crash dưới dạng JSON-Lines vào `logs/system_telemetry.jsonl`.
   - **Chốt chặn 2 - Ghi Nhập Trực Tiếp Đĩa Cứng (Disk Pipe Logging)**: Toàn bộ `stdout` và `stderr` từ Native Engine được ghi đệm liên tục (`flush()`) vào tệp đĩa `logs/miner_stdout_stderr.log`. Dù Python hay Rust bị chết đột ngột, 100% dòng log lỗi cuối cùng vẫn nằm nguyên vẹn trên đĩa.
   - **Chốt chặn 3 - Giao Diện Truy Vấn Telemetry Trực Tiếp**: Thêm nút **"📜 TRUY VẤN LOG ĐĨA & TELEMETRY"** trên web UI Gradio, cho phép người dùng xem ngay lập tức các sự kiện telemetry và log đĩa chỉ bằng 1 cú click.
   - **Chốt chặn 4 - Báo Cáo Crash Tự Động Trong `sync_on_load()`**: Khi reload trang, nếu phát hiện phiên trước bị crash, hệ thống sẽ đọc trực tiếp 40 dòng log từ `logs/miner_stdout_stderr.log` để hiển thị nguyên nhân chính xác (OOM / panic / invalid arg) thay vì reset mờ ám!

---

### VII. BÀI HỌC VỀ SIEVE POWER-OF-TWO PANIC & FILENOTFOUNDERROR GUARD (v2.7.0-production)

1. **NGUYÊN NHÂN GỐC RỄ**:
   - **Lỗi Sieve Panic**: Trong `21_ram64g_mine.rs` và `23_jrcp3_ram64g_miner.rs`, Sieve Bitset yêu cầu số phần tử `count` phải là Lũy Thừa Của 2 để phép toán Bitwise AND Mask (`key & mask`) chạy O(1). Khi `app.py` tự động tính RAM cho 96GB hệ thống, `sieve_mb = 24796 MB` (không phải lũy thừa 2) làm Rust engine tung `panic!` hủy tiến trình với exitcode `-6`.
   - **Lỗi FileNotFoundError**: Khi Rust Engine bị panic ngắt sớm trước khi kịp tạo tệp `out_file`, lệnh `os.path.getsize(out_file)` cũ ném ra ngoại lệ `FileNotFoundError` làm sập UI.

2. **GIẢI PHÁP ĐÃ THỰC THI (Commit `ed9d921` & `6081ae3`)**:
   - **Lớp Rust Engine**: Thay câu lệnh `assert` bằng thuật toán Bitwise tự động làm tròn `count = 1usize << (usize::BITS - 1 - raw_count.leading_zeros())`. Dù truyền vào số lẻ nào (`24796 MB`), Rust tự làm tròn an toàn về lũy thừa 2 gần nhất (`16384 MB`) mà KHÔNG BAO GIỜ bị panic.
   - **Lớp Python UI**: Thêm hàm helper `get_file_size_mb(filepath)` phòng ngự kiểm tra `os.path.exists()`, trả về `0.00 MB` an toàn tuyệt đối nếu tệp chưa tồn tại.
   - **Thêm helper `prev_power_of_two(n)`**: Tự động làm tròn `sieve_mb` trên UI slider về lũy thừa 2 chuẩn.

---

### VIII. NÂNG CẤP QUY TẮC BẮT BUỘC TĂNG PHIÊN BẢN KHI SỬA MÃ NGUỒN (VERSION BUMP PROTOCOL)

1. **NGUYÊN NHÂN THẤT BẠI CỦA CÁC TURN TRƯỚC**:
   - Khi sửa lỗi Sieve Power-of-Two Panic và `FileNotFoundError`, Agent chỉ tập trung sửa code logic trong `app.py` và `.rs` mà **quên tăng số phiên bản `APP_VERSION` và `APP_BUILD_STAMP`**.
   - Nguyên nhân là do trong `AGENTS.md` và `GEMINI.md` cũ, quy tắc đánh số phiên bản chỉ nói chung chung ở mức lý thuyết, chưa được luật hóa thành một quy trình 4 bước bắt buộc phải thực thi mỗi khi sửa code.

2. **LUẬT HÓA BẮT BUỘC TRONG AGENTS.MD VÀ GEMINI.MD (Commit `d1e3ca1`)**:
   - Thêm Mục 8.5 trong `AGENTS.md` và Mục 7.5 trong `GEMINI.md`: **BẤT KỲ LẦN NÀO SỬA CODE, PHẢI TĂNG VERSION VÀ BUILD STAMP**.
   - Sửa code mà không tăng phiên bản = Vi phạm kỷ luật nghiêm trọng!
   - Đã nâng cấp `app.py` lên phiên bản **`v2.7.0-production`** (Build `2026-08-09 21:08:00 ICT`).

---

### IX. TẮT HOÀN TOÀN CẢNH BÁO RÁC GRADIO DEPRECATION & NODE SSR PROXY (v2.8.0-production)

1. **NGUYÊN NHÂN NỔI CẢNH BÁO TRÊN TERMINAL LOGS**:
   - `UserWarning: The parameters have been moved from the Blocks constructor...`: Do Gradio 5+ cập nhật cú pháp tham số `theme` trong `gr.Blocks()`.
   - `UserWarning: Failed to start Node front proxy for SSR...`: Do HuggingFace Container không có sẵn Node.js runtime phù hợp để khởi chạy Gradio Server-Side Rendering (SSR) proxy.

2. **GIẢI PHÁP ĐÃ XỬ LÝ (Commit `7f6f7ec`)**:
   - **Tắt Cảnh Báo Deprecation**: Cấu hình `warnings.filterwarnings("ignore", category=UserWarning)` và `category=DeprecationWarning` ở đầu `app.py`.
   - **Vô Hiệu Hóa Node SSR Proxy**: Cấu hình biến môi trường `os.environ["GRADIO_SSR_MODE"] = "false"` và `os.environ["GRADIO_NODE_PORT"] = "0"`.
   - **Phiên Bản Mới**: Đã nâng cấp `app.py` lên **`v2.8.0-production`** (Build `2026-08-09 21:12:00 ICT`). Terminal console sạch sẽ 100% không còn cảnh báo rác.

---

### X. KỶ LUẬT BẢO TỒN KÝ ỨC BẤT BIẾN: CẤM BẢO GIỜ XÓA HOẶC GHI ĐÈ CÁC MỤC BÀI HỌC CŨ (STRICT IMMUTABLE APPEND-ONLY MEMORY MANDATE)

1. **NGUYÊN NHÂN ANH HDQB PHÁT HIỆN SỰ CỐ GHI ĐÈ**:
   - Khi cập nhật bài học mới bằng lệnh sửa tệp, Agent đã sử dụng vùng thay thế đè lên các Mục bài học cũ trước đó (như Mục V, VI, VII), khiến lịch sử bài học bị mất khỏi tệp đĩa.
   - Đây là vi phạm nghiêm trọng đối với **Nguyên Tắc Bảo Tồn Trạng Thái & Tri Thức Sinh Tồn**.

2. **LUẬT THÉP BẢO TỒN KÝ ỨC VĨNH CỬU (IMMUTABLE APPEND-ONLY MANDATE)**:
   - **NGHIÊM CẤM** xóa bỏ, cắt xén, hoặc ghi đè làm mất bất kỳ Mục bài học cũ nào trong các tệp `pain_points_*.md`.
   - Mọi cập nhật bài học mới **BẮT BUỘC PHẢI ĐƯỢC NỐI VÀO CUỐI TỆP (APPEND-ONLY)**.
   - Nếu tệp bài học hiện tại vượt quá 15KB hoặc chứa nhiều chủ đề khác nhau, Agent **BẮT BUỘC phải tạo tệp mới mang dấu thời gian** (ví dụ: `pain_points_[YYYYMMDD_HHMM].md`) và đăng ký vào tệp mục lục [`INDEX.md`](file://.agents/memory/INDEX.md).
   - Bảo đảm 100% di sản tri thức của tất cả các thế hệ Agent được giữ lại nguyên vẹn không sứt móng nanh!
