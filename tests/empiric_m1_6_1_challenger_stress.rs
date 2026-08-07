// ============================================================================
// XIANGTI ENGINE: THỬ NGHIỆM ĐO KIỂM XUNG ĐỘT BỘ NHỚ EMPIRICAL CHALLENGER (M1 ITERATION 6)
// ============================================================================
// Bộ kiểm thử thực nghiệm đối kháng kiểm tra tính đúng đắn của Buffer::allocate
// (lũy thừa 2, min 64 bytes, xử lý tràn số usize::MAX, căn lề 64-byte)
// và Buffer::pull (kiểm soát tranh chấp MPMC, zero data corruption, zero false Fault,
// zero dirty byte leak vào target buffer khi CAS thất bại).
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích tiếng Việt.
// ============================================================================

use std::sync::atomic::{AtomicUsize, Ordering}; // Nhập các kiểu nguyên tử đồng bộ
use std::sync::{Arc, Barrier}; // Nhập Arc và Barrier cho đồng bộ đa luồng
use std::thread; // Nhập module luồng
use xiangrust::gpu::buffer::{Buffer, Storable}; // Nhập Buffer và Storable
use xiangrust::gpu::status::Status; // Nhập enum Status

/// Cấu trúc phụ trợ phản chiếu bộ nhớ nội bộ của struct Buffer để kiểm thử boundary.
#[repr(C, align(64))]
struct Layout {
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

/// Kiểm thử 1: Xác minh tính đúng đắn của `Buffer::allocate` với lũy thừa 2, min 64 bytes, và tràn số.
#[test]
fn allocate() {
    // 1. Kiểm thử các kích thước hợp lệ
    let sizes = vec![1, 2, 3, 7, 8, 15, 16, 31, 32, 63, 64, 65, 100, 127, 128, 1000, 1024, 65535, 65536];
    for size in sizes {
        let buffer = Buffer::allocate(size, false).expect("Cấp phát kích thước hợp lệ phải thành công");
        assert!(buffer.capacity().is_power_of_two(), "Capacity phải là lũy thừa của 2");
        assert!(buffer.capacity() >= 64, "Capacity phải tối thiểu 64 bytes");
        assert!(buffer.capacity() >= size, "Capacity phải lớn hơn hoặc bằng kích thước yêu cầu");
        assert!(buffer.aligned(), "Bộ đệm phải được căn lề 64-byte");
        assert_eq!((buffer.pointer() as usize) % 64, 0, "Con trỏ bộ nhớ phải chia hết cho 64");
        assert_eq!(buffer.bytes(), size, "Kích thước bytes khởi tạo phải khớp");
    }

    // 2. Yêu cầu 0 byte phải trả về Status::Fault
    assert_eq!(Buffer::allocate(0, false).err(), Some(Status::Fault), "Kích thước 0 byte phải trả về Fault");

    // 3. Xử lý tràn số usize::MAX và cận biên lớn
    let overflows = vec![
        usize::MAX,
        usize::MAX / 2 + 1,
        usize::MAX - 63,
        1usize << (usize::BITS - 1),
        (1usize << (usize::BITS - 1)) + 1,
    ];
    for size in overflows {
        let result = Buffer::allocate(size, false);
        assert!(result.is_err(), "Cấp phát size tràn số phải trả về lỗi");
        let err = result.err().unwrap();
        assert!(err == Status::Fault || err == Status::Exhausted, "Lỗi trả về phải là Fault hoặc Exhausted");
    }
}

/// Kiểm thử 2: Xác minh thao tác wrapping tràn chỉ số usize::MAX không làm sai lệch offset hay hỏng dữ liệu.
#[test]
fn wrapping() {
    let mut buffer = Buffer::allocate(128, false).expect("Cấp phát buffer 128 bytes thành công");

    // Ép chỉ số head, tail, commit về gần kịch trần usize::MAX
    let pointer = (&mut buffer as *mut Buffer) as *mut Layout;
    unsafe {
        let max = usize::MAX - 10;
        (*pointer).head.store(max, Ordering::SeqCst);
        (*pointer).tail.store(max, Ordering::SeqCst);
        (*pointer).commit.store(max, Ordering::SeqCst);
    }

    let payload1 = vec![11u8, 22, 33, 44, 55, 66, 77, 88];
    buffer.push(&payload1).expect("Push qua biên usize::MAX phải thành công");

    let payload2 = vec![99u8, 100, 110, 120];
    buffer.push(&payload2).expect("Push tiếp theo sau khi wrap phải thành công");

    let mut target1 = vec![0u8; 8];
    buffer.pull(&mut target1).expect("Pull gói 1 qua biên wrap phải thành công");
    assert_eq!(target1, payload1, "Dữ liệu gói 1 đọc ra phải chính xác 100%");

    let mut target2 = vec![0u8; 4];
    buffer.pull(&mut target2).expect("Pull gói 2 phải thành công");
    assert_eq!(target2, payload2, "Dữ liệu gói 2 đọc ra phải chính xác 100%");

    let mut target3 = vec![0u8; 8];
    assert_eq!(buffer.pull(&mut target3).err(), Some(Status::Ready), "Buffer rỗng phải trả về Status::Ready");
}

/// Kiểm thử 3: Xác minh mảng target của caller không bị nhiễm dirty byte khi CAS trong `pull` thất bại.
#[test]
fn cleanliness() {
    let buffer = Arc::new(Buffer::allocate(256, false).expect("Cấp phát buffer thành công"));
    let payload = vec![0xA1u8, 0xB2, 0xC3, 0xD4, 0xE5, 0xF6, 0x17, 0x28];
    buffer.push(&payload).expect("Push gói dữ liệu mẫu thành công");

    let threads = 16;
    let barrier = Arc::new(Barrier::new(threads));
    let count = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();
    for _ in 0..threads {
        let b = Arc::clone(&buffer);
        let bar = Arc::clone(&barrier);
        let c = Arc::clone(&count);

        handles.push(thread::spawn(move || {
            let sentinel = 0xEEu8;
            let mut target = vec![sentinel; 8];

            bar.wait(); // Xuất phát đồng thời 100%

            let res = b.pull(&mut target);
            if res.is_ok() {
                c.fetch_add(1, Ordering::SeqCst);
                assert_eq!(target, vec![0xA1u8, 0xB2, 0xC3, 0xD4, 0xE5, 0xF6, 0x17, 0x28], "Luồng thắng CAS phải đọc đúng payload");
            } else {
                // Luồng thua CAS phải giữ nguyên 100% mảng target hoàn toàn sạch sẽ (sentinel values intact)
                assert_eq!(target, vec![sentinel; 8], "Target của luồng thất bại CAS phải không bị ghi bẩn!");
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(count.load(Ordering::SeqCst), 1, "Duy nhất 1 luồng được pull gói thành công");
}

/// Kiểm thử 4: Stress test MPMC đa luồng đẩy/rút cực hạn (16 producers, 16 consumers, 16,000 packets).
#[test]
fn concurrency() {
    let buffer = Arc::new(Buffer::allocate(65536, false).expect("Cấp phát buffer 64KB thành công"));
    let limit = 1000;
    let producers = 16;
    let consumers = 16;
    let total = limit * producers;

    let barrier = Arc::new(Barrier::new(producers + consumers));
    let count = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();

    // Khởi tạo các producer threads
    for producer in 0..producers {
        let b = Arc::clone(&buffer);
        let bar = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            bar.wait();
            for seq in 0..limit {
                let mut packet = vec![0u8; 16];
                packet[0..4].copy_from_slice(&(producer as u32).to_le_bytes());
                packet[4..8].copy_from_slice(&(seq as u32).to_le_bytes());
                let checksum = (producer as u64) ^ (seq as u64) ^ 0xCAFE_BABE_DADA_1234;
                packet[8..16].copy_from_slice(&checksum.to_le_bytes());

                loop {
                    match b.push(&packet) {
                        Ok(_) => break,
                        Err(Status::Full) => thread::yield_now(),
                        Err(e) => panic!("Push lỗi bất ngờ: {:?}", e),
                    }
                }
            }
        }));
    }

    // Khởi tạo các consumer threads
    for _ in 0..consumers {
        let b = Arc::clone(&buffer);
        let bar = Arc::clone(&barrier);
        let c = Arc::clone(&count);
        handles.push(thread::spawn(move || {
            bar.wait();
            let mut target = vec![0u8; 16];
            while c.load(Ordering::Acquire) < total {
                match b.pull(&mut target) {
                    Ok(_) => {
                        let producer = u32::from_le_bytes(target[0..4].try_into().unwrap()) as usize;
                        let seq = u32::from_le_bytes(target[4..8].try_into().unwrap()) as usize;
                        let checksum = u64::from_le_bytes(target[8..16].try_into().unwrap());
                        let expected = (producer as u64) ^ (seq as u64) ^ 0xCAFE_BABE_DADA_1234;
                        assert_eq!(checksum, expected, "Checksum của dữ liệu pull ra phải trùng khớp tuyệt đối!");
                        c.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(Status::Ready) => thread::yield_now(),
                    Err(e) => panic!("Pull lỗi bất ngờ: {:?}", e),
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(count.load(Ordering::SeqCst), total, "Tất cả 16,000 gói dữ liệu phải được xử lý thành công");
}

/// Kiểm thử 5: Tranh chấp kích thước target nhỏ hơn payload (Fault vs Retry under race condition).
#[test]
fn contention() {
    let buffer = Arc::new(Buffer::allocate(1024, false).expect("Cấp phát buffer thành công"));
    let payload = vec![0x77u8; 32];
    buffer.push(&payload).expect("Push payload 32 bytes thành công");

    // Target chỉ có 16 bytes (nhỏ hơn payload 32 bytes)
    let mut small = vec![0u8; 16];
    let res = buffer.pull(&mut small);
    assert_eq!(res.err(), Some(Status::Fault), "Target nhỏ hơn payload khi head không đổi phải trả về Fault");
}
