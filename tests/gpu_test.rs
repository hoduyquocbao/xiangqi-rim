// ============================================================================
// XIANGTI ENGINE: UNIT TESTS CHO GPU ADAPTER & VRAM GUARD (MILESTONE M1 FIX)
// ============================================================================
// Kiểm thử đơn vị toàn diện mô-đun src/gpu/: backend C-ABI FFI probe detection,
// VRAM guard limit 512MB & 409.6MB ceiling protection, CAS underflow release,
// 64-byte physical alignment, zero-copy, circular modulo ring buffer queue,
// đồng bộ hóa atomic commit index, và GPU autonomous NNUE batch evaluator.
// Tuân thủ 100% định danh từ đơn tiếng Anh và 100% chú thích tiếng Việt.
// ============================================================================

use xiangrust::gpu::{Backend, Buffer, Device, Guard, Status, Queryable, Storable, Validatable};
use std::sync::Arc;
use std::thread;

#[test]
fn test_backend_detection() {
    let backend = Backend::detect();
    #[cfg(target_os = "macos")]
    {
        assert_eq!(backend, Backend::Metal);
        assert_eq!(backend.name(), "Metal");
        assert!(backend.valid());
        assert!(backend.hardware());
        assert_eq!(backend.speed(), 100);
    }
    #[cfg(not(target_os = "macos"))]
    {
        assert_eq!(backend, Backend::Cpu);
        assert_eq!(backend.name(), "CPU");
        assert!(!backend.valid());
        assert!(!backend.hardware());
        assert_eq!(backend.speed(), 10);
    }
    assert_eq!(backend.rank(), backend as u8);
}

#[test]
fn test_vram_guard_limits_and_protection() {
    let guard = Guard::new();
    assert_eq!(guard.limit(), 536_870_912); // 512 MB
    assert_eq!(guard.ceiling(), 429_496_729); // 409.6 MB (80%)
    assert_eq!(guard.allocated(), 0);
    assert_eq!(guard.peak(), 0);
    assert_eq!(guard.count(), 0);
    assert_eq!(guard.status(), Status::Ready);

    // Test successful reserve within safe ceiling
    let bytes = 10 * 1024 * 1024; // 10 MB
    let result = guard.reserve(bytes);
    assert!(result.is_ok());
    assert_eq!(guard.allocated(), bytes);
    assert_eq!(guard.peak(), bytes);
    assert_eq!(guard.count(), 1);
    assert_eq!(guard.status(), Status::Active);

    // Test ceiling protection (attempting to reserve beyond 409.6 MB)
    let excess = 400 * 1024 * 1024; // 400 MB + 10 MB = 410 MB > 409.6 MB
    let err = guard.reserve(excess);
    assert_eq!(err.err(), Some(Status::Exhausted));
    assert_eq!(guard.allocated(), bytes); // Allocation must roll back

    // Test safe CAS release without underflow
    guard.release(bytes);
    assert_eq!(guard.allocated(), 0);
    assert_eq!(guard.count(), 0);
    assert_eq!(guard.status(), Status::Ready);

    // Edge case release on empty guard should not underflow
    guard.release(100);
    assert_eq!(guard.allocated(), 0);
    assert_eq!(guard.count(), 0);

    // Test chunks calculation
    assert_eq!(guard.chunks(0), 0);
    assert_eq!(guard.chunks(64 * 1024 * 1024), 1);
    assert_eq!(guard.chunks(65 * 1024 * 1024), 2);

    // Test validate trait method
    assert_eq!(guard.validate(100 * 1024 * 1024), Status::Ready); // <= 409.6MB
    assert_eq!(guard.validate(500 * 1024 * 1024), Status::Full);  // 409.6MB < 500MB <= 512MB
    assert_eq!(guard.validate(600 * 1024 * 1024), Status::Fail);  // > 512MB
}

#[test]
fn test_buffer_64byte_alignment_and_zero_copy() {
    let size = 1024; // 1 KB
    let mut buffer = Buffer::allocate(size, true).expect("Buffer allocation failed");
    
    // Verify physical 64-byte alignment
    let pointer = buffer.pointer() as usize;
    assert_eq!(pointer % 64, 0, "Buffer pointer must be physically 64-byte aligned!");
    assert!(buffer.aligned());
    assert!(buffer.device());
    assert_eq!(buffer.capacity(), 1024);

    #[cfg(target_os = "macos")]
    assert!(buffer.shared(), "macOS Intel iGPU must enable zero-copy shared mode!");

    // Test data write and read
    let data = vec![77u8; 512];
    assert!(buffer.write(&data).is_ok());
    assert_eq!(buffer.bytes(), 512);

    let mut readback = vec![0u8; 512];
    assert!(buffer.read(&mut readback).is_ok());
    assert_eq!(data, readback);

    // Test lock-free push and pull with circular modulo wrapping
    let payload = vec![123u8; 256];
    assert!(buffer.push(&payload).is_ok());
    let mut output = vec![0u8; 256];
    assert!(buffer.pull(&mut output).is_ok());
    assert_eq!(payload, output);

    // Test clear
    buffer.clear();
    assert_eq!(buffer.bytes(), 0);

    // Test free
    buffer.free();
    assert!(buffer.pointer().is_null());
    assert_eq!(buffer.capacity(), 0);
}

#[test]
fn test_buffer_atomic_commit_concurrency() {
    let capacity = 512;
    let buffer = Arc::new(Buffer::allocate(capacity, false).expect("Buffer allocation failed"));
    let total = 1000;
    
    let prod_buf = Arc::clone(&buffer);
    let producer = thread::spawn(move || {
        for i in 0..total {
            let data = (i as u32).to_le_bytes();
            while prod_buf.push(&data).is_err() {
                thread::yield_now();
            }
        }
    });

    let cons_buf = Arc::clone(&buffer);
    let consumer = thread::spawn(move || {
        let mut count = 0;
        let mut dest = [0u8; 4];
        while count < total {
            if cons_buf.pull(&mut dest).is_ok() {
                let val = u32::from_le_bytes(dest);
                assert_eq!(val, count as u32);
                count += 1;
            } else {
                thread::yield_now();
            }
        }
        count
    });

    producer.join().unwrap();
    let received = consumer.join().unwrap();
    assert_eq!(received, total);
}

#[test]
fn test_device_adapter_lifecycle_and_autonomous_eval() {
    let mut dev = Device::init();
    assert_eq!(dev.backend(), Backend::detect());
    assert_eq!(dev.guard().limit(), 536_870_912);

    // Test Queryable trait
    assert!(!dev.name().is_empty());
    assert_eq!(dev.memory(), 536_870_912);
    assert!(dev.active());

    // Test allocation via Device
    let bytes = 20 * 1024 * 1024; // 20 MB
    let mut buf = dev.allocate(bytes).expect("Device allocate failed");
    assert_eq!(dev.guard().allocated(), bytes);
    assert_eq!(buf.bytes(), bytes);

    // Test queue batch positions
    let batch = vec![42u8; 100];
    assert!(dev.queue(&buf, &batch).is_ok());

    // Test Autonomous NNUE Batch Evaluator (42 % 8 = 2 -> weight 20 * 100 = 2000)
    let score = dev.eval(&buf).expect("Autonomous GPU eval failed");
    assert_eq!(score, 2000);

    // Test free via Device
    assert!(dev.free(&mut buf).is_ok());
    assert_eq!(dev.guard().allocated(), 0);

    // Test reset
    assert!(dev.reset().is_ok());
    assert_eq!(dev.guard().allocated(), 0);
}

#[test]
fn test_hardware_struct_alignments() {
    assert_eq!(std::mem::align_of::<Guard>(), 64);
    assert_eq!(std::mem::align_of::<Buffer>(), 64);
    assert_eq!(std::mem::align_of::<Device>(), 64);

    assert_eq!(std::mem::size_of::<Guard>(), 64);
    assert_eq!(std::mem::size_of::<Buffer>(), 64);
    assert_eq!(std::mem::size_of::<Device>(), 128); // 2 cache lines
}
