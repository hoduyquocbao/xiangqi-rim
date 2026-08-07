// ============================================================================
// VÍ DỤ 14: HUẤN LUYỆN HỌC MÁY TĂNG CƯỜNG ONLINE & BỘ QUẢN LÝ TRAINER
// ============================================================================
// Chương trình ví dụ minh họa tiến trình tự đấu học thích ứng (Online Learning):
// 1. Khởi tạo đối tượng Trainer tích hợp Replay, Trace, Blunder, Store, Adapt.
// 2. Chạy chuỗi 10 ván tự đấu tự ghi nhận mẫu transition vào Replay buffer.
// 3. Cập nhật vết điều kiện Eligibility Trace & sai số TD(lambda) delta_t.
// 4. Nhận diện các nước đi sai lầm (Blunder >= 200cp) và tích lũy Penalty bias.
// 5. Kiểm thử lưu và nạp trí nhớ kinh nghiệm nhị phân (Persistence Store) xuống ổ đĩa.
// 6. Thống kê và so sánh sự cải thiện tỷ lệ thắng (Win Rate) qua từng giai đoạn.
// 100% chú thích tiếng Việt từng dòng & 100% định danh đơn từ tiếng Anh.
// ============================================================================

use xiangrust::learn::Trainer;

fn main() {
    // In tiêu đề trang trí khởi đầu chương trình minh họa học máy online
    println!("============================================================");
    println!("  XIANGRUST AI ENGINE - VÍ DỤ 14: ONLINE LEARNING & TRAINER ");
    println!("============================================================");

    // 1. KHỞI TẠO CẤU HÌNH VÀ BỘ QUẢN LÝ HUẤN LUYỆN (TRAINER INITIALIZATION)
    let depth = 3u8;
    let limit = 20u32;
    let count = 10u32;

    println!("\n[1] KHỞI TẠO CẤU HÌNH HUẤN LUYỆN ONLINE:");
    println!(" -> Độ sâu tìm kiếm (depth): {}", depth);
    println!(" -> Giới hạn nước đi/ván (limit): {}", limit);
    println!(" -> Số ván tự đấu (count): {}", count);

    // Khởi tạo trình quản lý Trainer
    let mut trainer = Trainer::new(depth, limit);

    println!("\n[2] BẮT ĐẦU CHUỖI VÁN TỰ ĐẤU HỌC TÍCH LŨY (10 VÁN)...");
    println!("------------------------------------------------------------");
    println!(" Ván | Số nước | Số mẫu Replay | Blunders | TD-Error | Kết quả ");
    println!("------------------------------------------------------------");

    // 2. CHẠY VÒNG LẶP HUẤN LUYỆN TỰ ĐẤU QUA 10 VÁN (TRAINING LOOP)
    for game in 1..=count {
        // Thực thi 1 ván tự đấu và thu thập dữ liệu học
        let stats = trainer.step(game);

        println!(
            " {:2}  |   {:2}    |     {:4}     |    {:2}    |  {:+.4}  | {}",
            game, stats.moves, stats.samples, stats.blunders, stats.delta, stats.label
        );
    }
    println!("------------------------------------------------------------");

    // 3. KIỂM THỬ LƯU VÀ NẠP BỘ NHỚ KINH NGHIỆM TRÊN Ổ ĐĨA (PERSISTENCE STORAGE)
    let path = "/tmp/xiangrust_experience.bin";
    println!("\n[3] LƯU & NẠP TRÍ NHỚ KINH NGHIỆM Persistence Store:");
    match trainer.save(path) {
        Ok(_) => println!(" -> Lưu tệp kinh nghiệm nhị phân thành công: {}", path),
        Err(err) => println!(" -> Lỗi lưu tệp: {:?}", err),
    }

    match trainer.load(path) {
        Ok(loaded) => println!(" -> Nạp thành công {} mẫu kinh nghiệm từ tệp nhị phân!", loaded),
        Err(err) => println!(" -> Lỗi nạp tệp: {:?}", err),
    }

    // Dọn dẹp tệp đĩa tạm sau khi kiểm thử xong
    let _ = std::fs::remove_file(path);

    // 4. TỔNG HỢP VÀ ĐÁNH GIÁ HIỆU QUẢ HỌC TÍCH LŨY (LEARNING PROGRESS REPORT)
    let samples = trainer.replay.count();
    let blunders = trainer.blunder.count();
    let early = trainer.wins(1, 5);
    let late = trainer.wins(6, 10);

    println!("\n[4] BÁO CÁO TỔNG HỢP THỐNG KÊ HỌC MÁY ONLINE:");
    println!(" -> Tổng số mẫu kinh nghiệm tích lũy (Replay): {}", samples);
    println!(" -> Tổng số nước đi sai lầm đã bị phạt (Blunders): {}", blunders);
    println!(" -> Số ván thắng giai đoạn đầu (Ván 1 - 5)   : {} / 5", early);
    println!(" -> Số ván thắng giai đoạn sau (Ván 6 - 10)  : {} / 5", late);

    if late > early {
        println!(" -> ĐÁNH GIÁ: Tỷ lệ thắng CẢI THIỆN RÕ RỆT nhờ tích lũy TD(lambda) & Blunder Bias!");
    } else {
        println!(" -> ĐÁNH GIÁ: Engine duy trì phong độ ổn định qua các epoch tự đấu!");
    }

    println!("\n=> HOÀN THÀNH CHƯƠNG TRÌNH VÍ DỤ 14 ONLINE LEARNING & TRAINER!");
}
