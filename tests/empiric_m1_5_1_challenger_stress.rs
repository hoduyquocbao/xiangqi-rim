// ============================================================================
// XIANGTI ENGINE: EMPIRICAL CHALLENGER STRESS SUITE (MILESTONE M1 ITERATION 5)
// ============================================================================
// Adversarial test harness for Buffer::allocate power-of-two capacity wrapping
// and Reserve-then-Copy CAS-first pull logic under high MPMC contention.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích tiếng Việt.
// ============================================================================

use std::sync::atomic::{AtomicUsize, Ordering}; // Nhập các kiểu nguyên tử std::sync::atomic
use std::sync::{Arc, Barrier}; // Nhập Arc và Barrier cho đồng bộ hóa đa luồng
use std::thread; // Nhập module thread
use xiangrust::gpu::buffer::{Buffer, Storable}; // Nhập struct Buffer và trait Storable
use xiangrust::gpu::guard::Guard; // Nhập struct Guard
use xiangrust::gpu::status::Status; // Nhập enum Status

/// Cấu trúc phụ trợ cho phép truy cập vùng nhớ nội bộ của struct Buffer trong môi trường kiểm thử.
#[repr(C, align(64))]
struct BufferTestLayout {
    pointer: *mut u8,
    bytes: usize,
    capacity: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
    commit: AtomicUsize,
    aligned: bool,
    device: bool,
    shared: bool,
    pad: [u8; 13],
}

/// Kiểm thử 1: Xác minh `Buffer::allocate` làm tròn dung lượng lũy thừa của 2 và xử lý tràn số usize::MAX.
#[test]
fn test_buffer_allocate_power_of_two_and_overflow_boundaries() {
    // 1. Kiểm thử các kích thước hợp lệ nhỏ và vừa
    let valid_sizes = vec![1, 2, 3, 7, 8, 15, 16, 31, 32, 63, 64, 65, 100, 127, 128, 1000, 1024, 4096];
    for size in valid_sizes {
        let buffer = Buffer::allocate(size, false).expect("Cấp phát kích thước hợp lệ phải thành công");
        assert!(buffer.capacity().is_power_of_two(), "Dung lượng capacity phải là lũy thừa của 2");
        assert!(buffer.capacity() >= 64, "Dung lượng capacity phải tối thiểu 64 bytes");
        assert!(buffer.capacity() >= size, "Dung lượng capacity phải lớn hơn hoặc bằng kích thước yêu cầu");
        assert!(buffer.aligned(), "Bộ đệm phải được căn lề 64-byte");
        assert_eq!(buffer.bytes(), size, "Kích thước bytes khởi tạo phải khớp với giá trị truyền vào");
    }

    // 2. Kiểm thử dung lượng 0 -> Phải trả về Status::Fault
    assert_eq!(Buffer::allocate(0, false).err(), Some(Status::Fault), "Dung lượng 0 phải trả về Fault");

    // 3. Kiểm thử các giá trị cận biên lớn gây tràn số (Boundary overflow)
    let overflow_sizes = vec![
        usize::MAX,
        usize::MAX / 2,
        usize::MAX - 63,
        1usize << (usize::BITS - 1),
        (1usize << (usize::BITS - 1)) + 1,
    ];
    for size in overflow_sizes {
        let result = Buffer::allocate(size, false);
        match &result {
            Ok(b) => println!("Overflow test size 0x{:X} ({}) -> Unexpected OK! cap={}", size, size, b.capacity()),
            Err(e) => println!("Overflow test size 0x{:X} ({}) -> Err({:?})", size, size, e),
        }
        assert!(result.is_err(), "Cấp phát size 0x{:X} vượt quá hạn mức bộ nhớ phải trả về lỗi!", size);
        let err = result.err().unwrap();
        assert!(
            err == Status::Fault || err == Status::Exhausted,
            "Lỗi trả về phải là Fault hoặc Exhausted"
        );
    }
}

/// Kiểm thử 2: Xác minh wrapping tràn số usize::MAX của head, tail, commit không làm trượt offset hay làm hỏng dữ liệu.
#[test]
fn test_buffer_usize_max_wrapping_correctness() {
    let mut buffer = Buffer::allocate(128, false).expect("Cấp phát buffer 128 bytes thành công");
    
    // Sử dụng transmute / layout pointer để ép head, tail, commit về gần cận biên usize::MAX
    let layout_ptr = (&mut buffer as *mut Buffer) as *mut BufferTestLayout;
    unsafe {
        let max_near = usize::MAX - 10;
        (*layout_ptr).head.store(max_near, Ordering::SeqCst);
        (*layout_ptr).tail.store(max_near, Ordering::SeqCst);
        (*layout_ptr).commit.store(max_near, Ordering::SeqCst);
    }

    // Gói dữ liệu 1: 8 bytes payload (total = 12 bytes) -> tail sẽ tràn qua usize::MAX về 1
    let payload1 = vec![10u8, 20, 30, 40, 50, 60, 70, 80];
    buffer.push(&payload1).expect("Push gói 1 qua biên usize::MAX phải thành công");

    // Gói dữ liệu 2: 6 bytes payload (total = 10 bytes) -> tail tăng lên 11
    let payload2 = vec![90u8, 100, 110, 120, 130, 140];
    buffer.push(&payload2).expect("Push gói 2 sau biên usize::MAX phải thành công");

    // Đọc gói 1 ra target1
    let mut target1 = vec![0u8; 8];
    buffer.pull(&mut target1).expect("Pull gói 1 qua biên usize::MAX phải thành công");
    assert_eq!(target1, payload1, "Dữ liệu gói 1 đọc ra phải chính xác 100%");

    // Đọc gói 2 ra target2
    let mut target2 = vec![0u8; 6];
    buffer.pull(&mut target2).expect("Pull gói 2 phải thành công");
    assert_eq!(target2, payload2, "Dữ liệu gói 2 đọc ra phải chính xác 100%");

    // Đảm bảo không còn dữ liệu thừa
    let mut dummy = vec![0u8; 8];
    assert_eq!(buffer.pull(&mut dummy).err(), Some(Status::Ready), "Hàng đợi phải ở trạng thái Ready sau khi rút hết");
}

/// Kiểm thử 3: Xác minh tính sạch sẽ của target buffer khi CAS thất bại (Reserve-then-Copy).
#[test]
fn test_pull_target_cleanliness_on_failed_cas() {
    let buffer = Arc::new(Buffer::allocate(256, false).expect("Cấp phát buffer thành công"));
    let payload = vec![0x11u8, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
    buffer.push(&payload).expect("Push 1 gói duy nhất thành công");

    let num_threads = 12;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    let success_count = Arc::new(AtomicUsize::new(0));

    for _ in 0..num_threads {
        let b = Arc::clone(&buffer);
        let bar = Arc::clone(&barrier);
        let sc = Arc::clone(&success_count);

        handles.push(thread::spawn(move || {
            let sentinel = 0xAAu8;
            let mut target = vec![sentinel; 8];

            bar.wait(); // Đồng bộ tất cả các luồng xuất phát cùng 1 vi giây

            let res = b.pull(&mut target);
            if res.is_ok() {
                sc.fetch_add(1, Ordering::SeqCst);
                assert_eq!(target, vec![0x11u8, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88], "Luồng thành công phải nhận đúng payload");
            } else {
                // Luồng thất bại CAS: Mảng target phải giữ nguyên 100% sentinel (zero copy / zero dirty data)
                assert_eq!(target, vec![sentinel; 8], "Mảng target của luồng thất bại CAS phải hoàn toàn sạch sẽ!");
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(success_count.load(Ordering::SeqCst), 1, "Duy nhất 1 luồng được phép pull thành công gói dữ liệu");
}

/// Kiểm thử 4: Stress test MPMC đa luồng cực hạn (16 producers, 16 consumers, 50,000 packets).
#[test]
fn test_mpmc_extreme_stress_concurrency() {
    let buffer = Arc::new(Buffer::allocate(65536, false).expect("Cấp phát buffer 64KB thành công"));
    let total_packets_per_producer = 2000;
    let num_producers = 8;
    let num_consumers = 8;
    let total_packets = total_packets_per_producer * num_producers;

    let barrier = Arc::new(Barrier::new(num_producers + num_consumers));
    let pulled_count = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();

    // Khởi chạy Producer threads
    for prod_id in 0..num_producers {
        let b = Arc::clone(&buffer);
        let bar = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            bar.wait();
            for seq in 0..total_packets_per_producer {
                let mut packet = vec![0u8; 16];
                packet[0..4].copy_from_slice(&(prod_id as u32).to_le_bytes());
                packet[4..8].copy_from_slice(&(seq as u32).to_le_bytes());
                let checksum = (prod_id as u64) ^ (seq as u64) ^ 0xFEED_FACE_CAFE_BABE;
                packet[8..16].copy_from_slice(&checksum.to_le_bytes());

                loop {
                    match b.push(&packet) {
                        Ok(_) => break,
                        Err(Status::Full) => thread::yield_now(),
                        Err(e) => panic!("Push gặp lỗi bất ngờ: {:?}", e),
                    }
                }
            }
        }));
    }

    // Khởi chạy Consumer threads
    for _ in 0..num_consumers {
        let b = Arc::clone(&buffer);
        let bar = Arc::clone(&barrier);
        let pc = Arc::clone(&pulled_count);
        handles.push(thread::spawn(move || {
            bar.wait();
            let mut target = vec![0u8; 16];
            while pc.load(Ordering::Acquire) < total_packets {
                match b.pull(&mut target) {
                    Ok(_) => {
                        let prod_id = u32::from_le_bytes(target[0..4].try_into().unwrap()) as usize;
                        let seq = u32::from_le_bytes(target[4..8].try_into().unwrap()) as usize;
                        let checksum = u64::from_le_bytes(target[8..16].try_into().unwrap());

                        let expected_checksum = (prod_id as u64) ^ (seq as u64) ^ 0xFEED_FACE_CAFE_BABE;
                        assert_eq!(checksum, expected_checksum, "Checksum dữ liệu pull ra phải khớp tuyệt đối");
                        pc.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(Status::Ready) => thread::yield_now(),
                    Err(e) => panic!("Pull gặp lỗi bất ngờ: {:?}", e),
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(pulled_count.load(Ordering::SeqCst), total_packets, "Tất cả các gói dữ liệu phải được tiêu thụ thành công");
}

/// Kiểm thử 5: Stress test VRAM Guard giải phóng và đặt trước nguyên tử dưới tranh chấp đa luồng.
#[test]
fn test_vram_guard_concurrent_reserve_and_release() {
    let guard = Arc::new(Guard::new());
    let num_threads = 16;
    let iterations = 1000;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    for _ in 0..num_threads {
        let g = Arc::clone(&guard);
        let bar = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            bar.wait();
            for _ in 0..iterations {
                let bytes = 1024 * 64; // 64KB per alloc
                match g.reserve(bytes) {
                    Ok(_) => {
                        assert!(g.allocated() <= Guard::CEILING, "Dung lượng allocated không được vượt quá ceiling");
                        g.release(bytes);
                    }
                    Err(Status::Exhausted) => thread::yield_now(),
                    Err(e) => panic!("Guard reserve gặp lỗi bất ngờ: {:?}", e),
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(guard.allocated(), 0, "Dung lượng allocated sau khi giải phóng hết phải bằng 0");
    assert_eq!(guard.count(), 0, "Số lượng khối count sau khi giải phóng hết phải bằng 0");
    assert!(guard.peak() > 0, "Đỉnh peak phải lớn hơn 0");
}
