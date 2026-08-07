// ============================================================================
// BÀI KIỂM THỬ THÁCH THỨC VẬT LÝ & LỰC ÉP CAO (EMPIRICAL STRESS TEST HARNESS)
// PHÂN HỆ LEARN: EXPERIENCE REPLAY, ELIGIBILITY TRACE, BLUNDER & PERSISTENCE STORE
// ============================================================================
// Mã nguồn kiểm thử thực nghiệm (Empirical Verification) cho M1 & M2.
// Tuân thủ 100% Clean Room std-only, 100% chú thích Tiếng Việt, 100% định danh từ đơn Tiếng Anh.
// ============================================================================

use std::fs::File;
use std::io::Write;
use xiangrust::learn::{
    Adapt, Blunder, Entry, Fault, Header, Record, Replay, Sample, Stats, Store, Trace, Trainer,
};

/// 1. KIỂM THỬ CĂN LỀ BỘ NHỚ VÀ DUNG LƯỢNG KÍCH THƯỚC STRUCT VẬT LÝ
#[test]
fn test_alignments_and_sizes() {
    // Structural alignments
    assert_eq!(std::mem::align_of::<Sample>(), 16);
    assert_eq!(std::mem::size_of::<Sample>(), 32);

    assert_eq!(std::mem::align_of::<Replay>(), 64);
    assert_eq!(std::mem::align_of::<Trace>(), 64);
    assert_eq!(std::mem::align_of::<Blunder>(), 64);
    assert_eq!(std::mem::align_of::<Store>(), 64);
    assert_eq!(std::mem::align_of::<Adapt>(), 64);
    assert_eq!(std::mem::align_of::<Trainer>(), 64);

    assert_eq!(std::mem::align_of::<Header>(), 64);
    assert_eq!(std::mem::size_of::<Header>(), 64);

    assert_eq!(std::mem::align_of::<Record>(), 16);
    assert_eq!(std::mem::size_of::<Record>(), 32);

    assert_eq!(std::mem::align_of::<Fault>(), 16);
    assert_eq!(std::mem::size_of::<Fault>(), 32);

    assert_eq!(std::mem::align_of::<Entry>(), 16);
    assert_eq!(std::mem::size_of::<Entry>(), 32);

    assert_eq!(std::mem::align_of::<Stats>(), 16);
    assert_eq!(std::mem::size_of::<Stats>(), 64);
}

/// 2. BÀI THỬ LỰC ÉP BỘ ĐỆM REPLAY VỚI 50,000+ PUSHES VÀ RANDOM SAMPLING
#[test]
fn test_replay_stress_50k() {
    let mut replay = Replay::capacity(10000);
    assert!(replay.empty());
    assert_eq!(replay.len(), 0);

    // Ép 50,000 mẫu vào bộ đệm xoay vòng 10,000
    for i in 0..50000u64 {
        let sample = Sample::new(
            i * 1000 + 7,
            (i % 65535) as u16,
            (i as f32) / 50000.0 - 0.5,
            (i + 1) * 1000 + 7,
            if i % 20 == 0 { 1 } else { 0 },
        );
        replay.push(sample);
    }

    // Sức chứa tối đa 10,000 không phình bộ nhớ
    assert_eq!(replay.len(), 10000);
    assert_eq!(replay.count(), 10000);
    assert!(!replay.empty());

    // Vị trí head đã xoay vòng 5 lần (50,000 mod 10,000 = 0)
    assert_eq!(replay.head, 0);

    // Lấy mẫu ngẫu nhiên 1,000 mẫu theo batch
    let mut batch = [Sample::empty(); 128];
    for _ in 0..10 {
        let k = replay.sample(&mut batch);
        assert_eq!(k, 128);
        for s in batch.iter() {
            assert!(s.hash > 0);
            assert!(s.reward >= -0.5 && s.reward <= 0.5);
        }
    }

    // Lấy mẫu lớn hơn số lượng phần tử hiện có
    let mut large_batch = vec![Sample::empty(); 20000];
    let k_large = replay.sample(&mut large_batch);
    assert_eq!(k_large, 10000);

    // Xóa sạch bộ đệm
    replay.clear();
    assert_eq!(replay.len(), 0);
    assert!(replay.empty());

    // Lấy mẫu khi rỗng trả về 0
    let mut empty_batch = [Sample::empty(); 10];
    let k_empty = replay.sample(&mut empty_batch);
    assert_eq!(k_empty, 0);
}

/// 3. BÀI THỬ BẢO TOÀN CHÍNH XÁC NGUYÊN BYTE KHI LƯU TRỮ VÀ NẠP PERSISTENCE STORE
#[test]
fn test_store_exact_byte_preservation() {
    let path = "/tmp/test_empiric_store_byte_exact.bin";

    let mut original = Replay::capacity(500);
    for i in 0..500u64 {
        let reward = match i % 4 {
            0 => 0.0f32,
            1 => -1.0f32,
            2 => 1.0f32,
            _ => 0.1234567f32,
        };
        let sample = Sample::new(
            0xDEAD_BEEF_0000_0000 | i,
            (i & 0xFFFF) as u16,
            reward,
            0xCAFE_BABE_0000_0000 | (i + 1),
            (i % 2) as u8,
        );
        original.push(sample);
    }

    // Ghi xuống đĩa
    let save_res = Store::save(&original, path);
    assert!(save_res.is_ok());

    // Kiểm tra cấu trúc tệp nhị phân vật lý trên đĩa
    let file_bytes = std::fs::read(path).expect("Đọc tệp nhị phân thất bại");
    let expected_size = 64 + 500 * 32;
    assert_eq!(file_bytes.len(), expected_size);

    // Kiểm tra Magic Header b"XRLN" ở 4 bytes đầu
    assert_eq!(&file_bytes[0..4], b"XRLN");

    // Nạp vào đối tượng Replay target rỗng
    let mut target = Replay::capacity(1000);
    let load_res = Store::load(&mut target, path);
    assert!(load_res.is_ok());
    assert_eq!(load_res.unwrap(), 500);
    assert_eq!(target.len(), 500);

    // Kiểm tra chính xác tuyệt đối từng field byte của từng mẫu
    for i in 0..500 {
        let orig_sample = original.get(i).unwrap();
        let loaded_sample = target.get(i).unwrap();

        assert_eq!(orig_sample.hash, loaded_sample.hash);
        assert_eq!(orig_sample.mv, loaded_sample.mv);
        assert_eq!(orig_sample.reward, loaded_sample.reward);
        assert_eq!(orig_sample.next, loaded_sample.next);
        assert_eq!(orig_sample.done, loaded_sample.done);
    }

    // Dọn dẹp tệp tạm
    let _ = std::fs::remove_file(path);
}

/// 4. BÀI THỬ BIẾN DẠNG TỆP NHỊ PHÂN VÀ HÀM LỖI PERSISTENCE STORE
#[test]
fn test_store_corrupted_headers_and_payloads() {
    let mut replay = Replay::capacity(100);

    // 4.1. Tệp không tồn tại
    let err_nofile = Store::load(&mut replay, "/tmp/non_existent_file_xyz_123.bin");
    assert!(err_nofile.is_err());

    // 4.2. Tệp bị cắt ngắn Header (< 64 bytes)
    let path_short = "/tmp/test_empiric_corrupt_short.bin";
    {
        let mut f = File::create(path_short).unwrap();
        f.write_all(b"XRLN_SHORT").unwrap();
    }
    let err_short = Store::load(&mut replay, path_short);
    assert!(err_short.is_err());
    let _ = std::fs::remove_file(path_short);

    // 4.3. Tệp sai Magic Header
    let path_bad_magic = "/tmp/test_empiric_corrupt_bad_magic.bin";
    {
        let mut f = File::create(path_bad_magic).unwrap();
        let mut bad_hdr = Header::empty();
        bad_hdr.magic = *b"BADM";
        bad_hdr.version = 1;
        bad_hdr.count = 10;
        let bytes = unsafe {
            std::slice::from_raw_parts(&bad_hdr as *const Header as *const u8, 64)
        };
        f.write_all(bytes).unwrap();
    }
    let err_magic = Store::load(&mut replay, path_bad_magic);
    assert!(err_magic.is_err());
    let _ = std::fs::remove_file(path_bad_magic);

    // 4.4. Tệp sai Phiên bản Version
    let path_bad_ver = "/tmp/test_empiric_corrupt_bad_ver.bin";
    {
        let mut f = File::create(path_bad_ver).unwrap();
        let mut bad_hdr = Header::empty();
        bad_hdr.magic = *b"XRLN";
        bad_hdr.version = 999;
        bad_hdr.count = 10;
        let bytes = unsafe {
            std::slice::from_raw_parts(&bad_hdr as *const Header as *const u8, 64)
        };
        f.write_all(bytes).unwrap();
    }
    let err_ver = Store::load(&mut replay, path_bad_ver);
    assert!(err_ver.is_err());
    let _ = std::fs::remove_file(path_bad_ver);

    // 4.5. Tệp báo count = 100 nhưng dữ liệu chỉ chứa 2 bản ghi
    let path_truncated_records = "/tmp/test_empiric_corrupt_trunc_rec.bin";
    {
        let mut f = File::create(path_truncated_records).unwrap();
        let hdr = Header::new(100);
        let hdr_bytes = unsafe {
            std::slice::from_raw_parts(&hdr as *const Header as *const u8, 64)
        };
        f.write_all(hdr_bytes).unwrap();

        let rec = Record::from(&Sample::new(1, 1, 0.0, 2, 0));
        let rec_bytes = unsafe {
            std::slice::from_raw_parts(&rec as *const Record as *const u8, 32)
        };
        f.write_all(rec_bytes).unwrap();
        f.write_all(rec_bytes).unwrap();
    }
    let res_trunc = Store::load(&mut replay, path_truncated_records);
    assert!(res_trunc.is_ok());
    assert_eq!(res_trunc.unwrap(), 2);
    assert_eq!(replay.len(), 2);
    let _ = std::fs::remove_file(path_truncated_records);
}

/// 5. BÀI THỬ TRACE VẾT ELIGIBILITY TRACE TRONG VÁN ĐẤU CỰC ĐẠI (250 PLIES)
#[test]
fn test_trace_200_plies_extreme_length() {
    let mut trace = Trace::new();
    assert_eq!(trace.len(), 0);

    // Mô phỏng ván cờ kéo dài 250 nước (250 plies)
    for ply in 1..=250u64 {
        let hash_source = ply * 100;
        let hash_target = (ply + 1) * 100;
        let reward = if ply == 250 { 1.0f32 } else { 0.0f32 };
        let done = ply == 250;

        let delta = trace.update(hash_source, hash_target, reward, done);

        // Kiểm tra không phát sinh NaN hoặc Infinity trong delta
        assert!(!delta.is_nan());
        assert!(!delta.is_infinite());
    }

    // Kiểm tra vết trace decay tự động loại bỏ các thế cờ quá cũ
    assert!(trace.len() <= 4096);

    // Ép tràn bảng vết trace với 5,000 thế cờ khác nhau
    let mut overflow_trace = Trace::capacity(100);
    for i in 1..=5000u64 {
        overflow_trace.update(i, i + 1, 0.0, false);
        assert!(overflow_trace.len() <= 100);
    }
}

/// 6. BÀI THỬ BLUNDER ANALYSIS CẤP ĐỘ CAO VÀ ĐIỂM PHẠT TÍCH LŨY
#[test]
fn test_blunder_accumulation_and_overflow() {
    let mut blunder = Blunder::capacity(50);
    assert_eq!(blunder.len(), 0);

    // Kiểm tra ngưỡng sai lầm (best - played >= threshold 200cp)
    assert!(!blunder.check(100, 1, 500, 301)); // drop = 199 < 200 -> false
    assert!(blunder.check(100, 1, 500, 300));  // drop = 200 >= 200 -> true
    assert_eq!(blunder.len(), 1);
    assert_eq!(blunder.penalty(100, 1), 100);

    // Tích lũy điểm phạt cho cùng 1 thế cờ và nước đi
    for _ in 0..150 {
        blunder.check(100, 1, 500, 200);
    }

    // Điểm phạt được giới hạn tối đa 10,000 centipawns
    assert_eq!(blunder.penalty(100, 1), 10000);

    // Ép tràn sức chứa 50 bản ghi
    for i in 2..=200u16 {
        blunder.record(i as u64, i, 100);
        assert!(blunder.len() <= 50);
    }

    // Tra cứu nước chưa phạm lỗi trả về 0
    assert_eq!(blunder.penalty(9999, 99), 0);
}

/// 7. BÀI THỬ ADAPTIVE SEARCH LIMITS & CÔNG THỨC THÍCH ỨNG
#[test]
fn test_adapt_search_limits() {
    // 7.1. Độ phức tạp bàn cờ C_board
    let min_c = Adapt::board(0, 0, 0);
    assert_eq!(min_c, 1.0);

    let max_c = Adapt::board(100, 50, 50);
    assert_eq!(max_c, 5.0);

    // 7.2. Độ ổn định tuyến PV S_pv
    assert_eq!(Adapt::pv(&[]), 1.0);
    assert_eq!(Adapt::pv(&[10]), 1.0);

    let stable_pv = [10u16, 10, 10, 10];
    assert_eq!(Adapt::pv(&stable_pv), 1.0);

    let unstable_pv = [10u16, 20, 30, 40];
    assert_eq!(Adapt::pv(&unstable_pv), 0.0);

    // 7.3. Cửa sổ Aspiration Window
    assert_eq!(Adapt::window(1.0), 16);
    assert_eq!(Adapt::window(0.80), 16);
    assert_eq!(Adapt::window(0.50), 32);
    assert_eq!(Adapt::window(0.20), 64);

    // 7.4. Mức cắt giảm độ sâu LMR
    assert_eq!(Adapt::lmr(2, 1.0, 0), 2);
    assert_eq!(Adapt::lmr(2, 0.5, 0), 1);
    assert_eq!(Adapt::lmr(2, 0.5, 100), 0);
    assert_eq!(Adapt::lmr(0, 0.0, 100), 0); // Không bao giờ giảm xuống âm
}

/// 8. BÀI THỬ TÍCH HỢP TOÀN DIỆN TRAINER OVER MULTIPLE SELF-PLAY GAMES
#[test]
fn test_trainer_multi_game_stress() {
    let mut trainer = Trainer::new(2, 5);

    // Chạy 5 ván tự đấu
    for g in 1..=5 {
        let stats = trainer.step(g);
        assert!(stats.moves > 0);
        assert!(stats.samples > 0);
        assert!(!stats.delta.is_nan());
    }

    assert_eq!(trainer.replay.len(), 25);

    // Lưu tệp kinh nghiệm
    let path = "/tmp/test_empiric_trainer_multi.bin";
    assert!(trainer.save(path).is_ok());

    // Clear và nạp lại
    trainer.clear();
    assert_eq!(trainer.replay.len(), 0);

    let loaded = trainer.load(path);
    assert!(loaded.is_ok());
    assert_eq!(loaded.unwrap(), 25);
    assert_eq!(trainer.replay.len(), 25);

    let _ = std::fs::remove_file(path);
}
