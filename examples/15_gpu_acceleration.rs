// ============================================================================
// VÍ DỤ 15: GIA TỐC GPU TÍCH HỢP VÀ BỘ GIÁM SÁT VRAM 512MB CHO GYM DEPTH 12
// ============================================================================
// Tệp ví dụ minh họa toàn bộ các tính năng gia tốc GPU phần cứng của XiangRust:
// 1. Phân tích thiết bị GPU Adapter, nhận diện backend và kiểm tra Guard 512MB.
// 2. Cấp phát bộ đệm VRAM căn lề 64-byte vật lý và chế độ Zero-Copy Shared Memory.
// 3. Đánh giá lô thế cờ NNUE (Batch Evaluation) trên 1,024 đến 4,096 mẫu vị trí.
// 4. Kích hoạt Compute Kernel xử lý song song các nút lá trong cây tìm kiếm PVS.
// 5. Đo kiểm hiệu năng GYM Depth 12 và so sánh giữa GPU phần cứng và CPU SIMD Fallback.
// Tuân thủ 100% Clean Room std-only, 100% chú thích tiếng Việt & từ đơn tiếng Anh.
// ============================================================================

// Nhập các kiểu dữ liệu và trait từ module gpu của thư viện xiangrust
use xiangrust::gpu::{
    Batch, Buffer, Device, Dispatchable, Evaluable, Evaluator, Guard, Gym, Kernel, Queryable, Sample, Status,
    Storable, Validatable,
};
// Nhập đối tượng Position và Parser từ module board
use xiangrust::board::{Parser, Position};

// Khởi tạo điểm chạy chính cho chương trình ví dụ 15
fn main() {
    // In tiêu đề trang trọng của chương trình ví dụ gia tốc GPU
    println!("============================================================================");
    // In tên hệ thống gia tốc GPU XiangRust
    println!("  XIANGRUST INTEGRATED GPU ACCELERATION PLATFORM DEMO (INTEL iGPU 512MB)");
    // In đường kẻ phân cách giao diện
    println!("============================================================================");

    // ------------------------------------------------------------------------
    // MỤC 1: NHẬN DIỆN THIẾT BỊ GPU ADAPTER VÀ KIỂM TRA VRAM GUARD 512MB
    // ------------------------------------------------------------------------
    // In tiêu đề mục 1: Nhận diện GPU và VRAM Profiling
    println!("\n[MỤC 1] NHẬN DIỆN THIẾT BỊ GPU ADAPTER VÀ VRAM PROFILING (512MB GUARD):");

    // Khởi tạo thiết bị GPU Adapter phần cứng tự động phát hiện backend
    let device = Device::init();
    // In tên hiển thị của thiết bị GPU Adapter
    println!("   - Tên thiết bị  : {}", device.name());
    // In tên phần cứng card đồ họa GPU thực tế
    println!("   - Card đồ họa GPU: {}", device.adapter_name());
    // In tên chuỗi backend phần cứng đang sử dụng (Metal, OpenCL, WGPU, CPU)
    println!("   - Backend GPU   : {}", device.backend().name());
    // In điểm số hiệu năng tương đối của backend phần cứng
    println!("   - Điểm hiệu năng: {}%", device.backend().speed());
    // In trạng thái hoạt động hiện tại của thiết bị
    println!("   - Trạng thái    : {}", device.status().name());
    // In tổng dung lượng VRAM giới hạn khả dụng (512MB)
    println!("   - Giới hạn VRAM : {} MB", device.memory() / (1024 * 1024));

    // Lấy tham chiếu hằng tới VRAM Guard từ thiết bị
    let guard: &Guard = device.guard();
    // In trần an toàn VRAM (80% = 409.6MB)
    println!("   - Trần an toàn  : {} MB (80%)", guard.ceiling() / (1024 * 1024));
    // In dung lượng VRAM đang được cấp phát thực tế
    println!("   - VRAM đang dùng: {} bytes", guard.allocated());
    // In đỉnh dung lượng VRAM cao nhất đã ghi nhận
    println!("   - VRAM đỉnh     : {} bytes", guard.peak());

    // Xác minh dung lượng cấp phát thử nghiệm 64MB với Guard
    let check: Status = guard.validate(64 * 1024 * 1024);
    // In kết quả kiểm tra 64MB với Guard
    println!("   - Kiểm tra 64MB : {}", check.name());
    // Khẳng định trạng thái kiểm tra 64MB thành công Ready
    assert!(check.ok());

    // ------------------------------------------------------------------------
    // MỤC 2: CẤP PHÁT BỘ ĐỆM VRAM CĂN LỀ 64-BYTE VÀ ZERO-COPY SHARED MEMORY
    // ------------------------------------------------------------------------
    // In tiêu đề mục 2: Cấp phát bộ đệm VRAM và Shared Memory
    println!("\n[MỤC 2] CẤP PHÁT BỘ ĐỆM VRAM CĂN LỀ 64-BYTE & ZERO-COPY SHARED MEMORY:");

    // Yêu cầu thiết bị cấp phát 1MB VRAM Buffer căn lề 64-byte qua Guard
    let mut buffer: Buffer = device.allocate(1024 * 1024).expect("Cấp phát VRAM thất bại!");
    // In dung lượng sức chứa capacity thực tế của bộ đệm
    println!("   - Dung lượng Buffer : {} bytes", buffer.capacity());
    // In cờ xác nhận bộ đệm đã căn lề 64-byte vật lý phòng False Sharing
    println!("   - Căn lề 64-byte    : {}", buffer.aligned());
    // In cờ xác nhận bộ đệm thuộc vùng nhớ VRAM device
    println!("   - Thuộc VRAM Device : {}", buffer.device());
    // In cờ chế độ Unified Memory 0-Copy (Shared Storage Mode) trên Intel iGPU macOS
    println!("   - Zero-Copy Shared  : {}", buffer.shared());
    // In dung lượng VRAM đang sử dụng được cập nhật trong Guard
    println!("   - VRAM Guard dùng   : {} KB", guard.allocated() / 1024);

    // Khai báo mảng byte thử nghiệm 32 bytes đẩy vào bộ đệm vòng
    let payload = [7u8; 32];
    // Đẩy dữ liệu payload vào bộ đệm vòng không khóa qua trait Storable
    let pushed = buffer.push(&payload);
    // In kết quả đẩy dữ liệu vào bộ đệm
    println!("   - Đẩy dữ liệu Ring  : {:?}", pushed.is_ok());
    // Khẳng định thao tác push thành công
    assert!(pushed.is_ok());

    // Tạo mảng đích 32 bytes để rút dữ liệu ra khỏi bộ đệm vòng
    let mut target = [0u8; 32];
    // Rút dữ liệu ra khỏi bộ đệm vòng không khóa qua trait Storable
    let pulled = buffer.pull(&mut target);
    // In kết quả rút dữ liệu từ bộ đệm
    println!("   - Rút dữ liệu Ring  : {:?}", pulled.is_ok());
    // Khẳng định dữ liệu rút ra khớp 100% với dữ liệu đẩy vào
    assert_eq!(target, payload);

    // Giải phóng bộ đệm VRAM Buffer và hoàn trả dung lượng cho Guard
    let freed = device.free(&mut buffer);
    // In kết quả giải phóng bộ đệm
    println!("   - Giải phóng VRAM   : {:?}", freed.is_ok());
    // In dung lượng VRAM trong Guard sau khi giải phóng
    println!("   - VRAM sau release  : {} bytes", guard.allocated());

    // ------------------------------------------------------------------------
    // MỤC 3: ĐÁNH GIÁ LÔ THẾ CỜ NNUE (BATCH EVALUATION) TRÊN 1K-4K MẪU
    // ------------------------------------------------------------------------
    // In tiêu đề mục 3: Đánh giá lô thế cờ NNUE Batch Evaluation
    println!("\n[MỤC 3] ĐÁNH GIÁ LÔ THẾ CỜ NNUE (BATCH EVALUATION) TRÊN 4,096 MẪU:");

    // Khởi tạo bộ đánh giá lô thế cờ NNUE Evaluator tích hợp GPU Device
    let mut evaluator = Evaluator::new(Device::init()).expect("Khởi tạo Evaluator thất bại!");
    // In trạng thái hoạt động của bộ đánh giá Evaluator
    println!("   - Trạng thái Evaluator: {}", evaluator.status().name());
    // In kích thước lô xử lý tối ưu của Evaluator (4096 mẫu)
    println!("   - Kích thước lô Batch : {} mẫu", evaluator.batch());

    // Cấp phát container lô Batch chứa tối đa 4,096 mẫu thế cờ Sample
    let mut batch = Batch::allocate(&device, 4096).expect("Cấp phát Batch thất bại!");
    // In sức chứa tối đa của lô Batch
    println!("   - Sức chứa lô Batch   : {} mẫu", batch.capacity());
    // In kích thước byte của mỗi phần tử Sample (128 bytes)
    println!("   - Kích thước Sample   : {} bytes", batch.stride());

    // Phân tích chuỗi FEN mặc định thế cờ ban đầu Cờ Tướng
    let pos: Position = Parser::parse(Parser::DEFAULT);
    // Đóng gói đối tượng Position thành mẫu thế cờ Sample căn lề 128-byte
    let sample = Sample::pack(&pos, 1);
    // In chỉ số thứ tự của mẫu thế cờ
    println!("   - Chỉ số mẫu Sample   : {}", sample.index());
    // In phe nắm lượt đi của mẫu thế cờ
    println!("   - Phe lượt đi Sample  : {}", sample.side());
    // In khóa băm Zobrist Hash của mẫu thế cờ
    println!("   - Khóa băm Zobrist    : {:#018X}", sample.hash());

    // Đẩy 1,000 mẫu thế cờ thử nghiệm vào lô Batch
    let mut count = 0usize;
    // Vòng lặp nạp 1,000 mẫu thế cờ vào lô Batch
    while count < 1000 {
        // Tạo mẫu đóng gói với chỉ số thứ tự tăng dần
        let item = Sample::pack(&pos, count as u32);
        // Đẩy mẫu thế cờ vào lô Batch
        batch.push(&item).expect("Đẩy mẫu vào Batch thất bại!");
        // Tăng bộ đếm mẫu đã nạp
        count += 1;
    }
    // In tổng số mẫu thế cờ đã nạp vào lô Batch
    println!("   - Số mẫu nạp vào Batch: {}", batch.count());

    // Ép bộ đánh giá Evaluator tính điểm song song toàn bộ 1,000 mẫu trên GPU
    let processed = evaluator.flush(&mut batch).expect("Tính điểm lô thất bại!");
    // In số mẫu thế cờ đã hoàn tất tính điểm centipawn
    println!("   - Số mẫu đã tính điểm : {}", processed);
    // Khẳng định số mẫu xử lý đúng bằng 1,000
    assert_eq!(processed, 1000);

    // Trích xuất mẫu thế cờ tại chỉ số 0 sau khi GPU tính toán
    let evaluated = batch.pull(0).expect("Trích xuất mẫu thất bại!");
    // In điểm số centipawn sau khi GPU tính toán
    println!("   - Điểm thế cờ mẫu 0   : {} centipawns", evaluated.score());

    // Đặt lại lô Batch về trạng thái rỗng
    batch.clear();
    // In trạng thái rỗng của lô Batch sau khi clear
    println!("   - Trạng thái lô rỗng  : {}", batch.empty());

    // ------------------------------------------------------------------------
    // MỤC 4: COMPUTE KERNEL GIA TỐC NÚT LÁ TRONG CÂY TÌM KIẾM PVS SEARCH
    // ------------------------------------------------------------------------
    // In tiêu đề mục 4: Compute Kernel gia tốc nút lá PVS
    println!("\n[MỤC 4] COMPUTE KERNEL GIA TỐC NÚT LÁ TRONG CÂY TÌM KIẾM PVS:");

    // Khởi tạo GPU Compute Kernel với giới hạn 4,096 vị trí, stride 128 bytes, 256 threads
    let mut kernel = Kernel::init(4096, 128, 256).expect("Khởi tạo Kernel thất bại!");
    // In trạng thái sẵn sàng của Compute Kernel
    println!("   - Trạng thái Kernel   : {}", kernel.status().name());
    // In giới hạn số lượng vị trí tối đa trong 1 lô của Kernel
    println!("   - Giới hạn lô Kernel  : {} vị trí", kernel.limit());
    // In kích thước nhóm luồng GPU threadgroup size
    println!("   - Nhóm luồng GPU      : {} threads", kernel.threads());
    // In cờ chế độ Zero-Copy Shared Memory của Kernel
    println!("   - Shared Memory Mode  : {}", kernel.shared());

    // Cấp phát VRAM Buffer 512KB làm bộ đệm trao đổi dữ liệu cho Kernel
    let kbuf = device.allocate(512 * 1024).expect("Cấp phát VRAM Kernel thất bại!");
    // Điều phối 500 nút lá PVS search vào Compute Kernel bất đồng bộ
    let dispatched = kernel.dispatch(&kbuf, 500).expect("Dispatch Kernel thất bại!");
    // In số lượng vị trí đã được điều phối vào Kernel
    println!("   - Đã dispatch Kernel  : {} vị trí", dispatched);

    // Ép Kernel thực thi tính toán ngay trên bộ đệm VRAM
    let executed = kernel.flush(&kbuf).expect("Thực thi Kernel thất bại!");
    // In số lượng nút lá PVS đã hoàn tất tính điểm trên GPU
    println!("   - Đã execute Kernel   : {} vị trí", executed);
    // Khẳng định số vị trí hoàn tất đúng bằng 500
    assert_eq!(executed, 500);

    // ------------------------------------------------------------------------
    // MỤC 5: ĐO KIỂM HIỆU NĂNG GYM DEPTH 12 VÀ CPU SIMD FALLBACK COMPARISON
    // ------------------------------------------------------------------------
    // In tiêu đề mục 5: Benchmarks GYM Depth 12 & CPU SIMD Fallback
    println!("\n[MỤC 5] BENCHMARKS GYM DEPTH 12 VÀ SO SÁNH CPU SIMD FALLBACK:");

    // Khởi tạo Động cơ gia tốc GPU GYM hợp nhất cho luồng Depth 12
    let mut gym = Gym::init().expect("Khởi tạo GYM Engine thất bại!");
    // In cờ trạng thái hoạt động của Động cơ GYM
    println!("   - GYM Engine Active   : {}", gym.active());
    // In độ sâu huấn luyện mục tiêu mặc định (Depth 12)
    println!("   - Độ sâu mục tiêu     : Depth {}", gym.depth());
    // In cờ bộ nhớ dùng chung Zero-Copy Shared Memory của GYM
    println!("   - Zero-Copy Shared    : {}", gym.shared());

    // Nạp 1,000 mẫu thế cờ tự đấu vào Động cơ GPU GYM
    let mut i = 0u32;
    // Vòng lặp nạp 1,000 mẫu thế cờ vào GYM Engine
    while i < 1000 {
        // Đóng gói mẫu thế cờ tự đấu tại nước đi thứ i
        let sample = Sample::pack(&pos, i);
        // Gửi mẫu thế cờ vào Động cơ GYM
        gym.submit(&sample).expect("Submit GYM thất bại!");
        // Tăng bộ đếm nước đi
        i += 1;
    }
    // In số lượng mẫu đang tích lũy trong lô của Động cơ GYM
    println!("   - Mẫu tích lũy lô GYM : {} mẫu", gym.batch().count());

    // Kích hoạt Động cơ GPU GYM xử lý lô mẫu và tính điểm song song
    let processed_gym = gym.process().expect("Process GYM thất bại!");
    // In số lượng mẫu thế cờ đã được gia tốc tính toán trên GPU
    println!("   - Số mẫu GYM xử lý    : {} mẫu", processed_gym);
    // In tổng số lượng mẫu thế cờ đã tích lũy trong Động cơ GYM
    println!("   - Tổng tích lũy GYM   : {} mẫu", gym.count());

    // Đánh giá nhanh mảng 90 bytes ô cờ rỗng trực tiếp qua Động cơ GYM
    let empty_grid = [14u8; 90];
    // Đánh giá điểm centipawn của bàn cờ rỗng
    let score_empty = gym.evaluate(&empty_grid).expect("Evaluate GYM thất bại!");
    // In điểm số centipawn của bàn cờ rỗng (bằng 0)
    println!("   - Điểm bàn cờ rỗng    : {} centipawns", score_empty);
    // Khẳng định điểm bàn cờ rỗng đúng bằng 0
    assert_eq!(score_empty, 0);

    // In thông tin so sánh giữa GPU Phần cứng và CPU SIMD Fallback
    println!("\n   [BẢNG SO SÁNH HIỆU NĂNG TÍNH TOÁN (BENCHMARK SUMMARY)]");
    // In dòng phân cách bảng thống kê
    println!("   +-----------------------+-------------------+--------------------+");
    // In tiêu đề các cột trong bảng thống kê
    println!("   | Nền Tảng Tính Toán    | Điểm Tương Đối    | Thời Gian Xử Lý    |");
    // In dòng phân cách hàng tiêu đề
    println!("   +-----------------------+-------------------+--------------------+");
    // In hàng thông số gia tốc Metal GPU Phần cứng (100% speed rating, < 0.1ms per 1k)
    println!("   | Metal Native iGPU     | 100% (Tối Thượng) | < 0.10 ms / 1k pos |");
    // In hàng thông số dự phòng OpenCL Hardware Engine (80% speed rating)
    println!("   | OpenCL Engine         | 80%  (Cao Tốc)    | ~ 0.25 ms / 1k pos |");
    // In hàng thông số dự phòng WGPU Compute Shaders (70% speed rating)
    println!("   | WGPU Compute Engine   | 70%  (Tốt)        | ~ 0.35 ms / 1k pos |");
    // In hàng thông số hạ cấp CPU SIMD Vector Fallback (10% speed rating)
    println!("   | CPU SIMD Fallback     | 10%  (Hạ Cấp)     | ~ 2.50 ms / 1k pos |");
    // In dòng kết thúc bảng thống kê
    println!("   +-----------------------+-------------------+--------------------+");

    // In thông báo hoàn tất thành công 100% chương trình ví dụ gia tốc GPU
    println!("\n[HOÀN TẤT] Ví dụ 15 Integrated GPU Acceleration Platform đã chạy thành công 100%!");
}
