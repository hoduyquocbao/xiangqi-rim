// Empirical verification and stress test suite for Milestone M4 GPU Acceleration Platform.

use std::mem::align_of;
use std::sync::Arc;
use std::thread;

use xiangrust::gpu::{
    Batch, Buffer, Device, Evaluator, Guard, Gym, Kernel, Queryable, Sample, Status,
    Storable,
};
use xiangrust::board::Parser;

#[test]
fn test_m4_gpu_struct_alignments() {
    assert_eq!(align_of::<Device>(), 64, "Device struct alignment must be 64 bytes");
    assert_eq!(align_of::<Buffer>(), 64, "Buffer struct alignment must be 64 bytes");
    assert_eq!(align_of::<Evaluator>(), 64, "Evaluator struct alignment must be 64 bytes");
    assert_eq!(align_of::<Gym>(), 64, "Gym struct alignment must be 64 bytes");
    assert_eq!(align_of::<Guard>(), 64, "Guard struct alignment must be 64 bytes");
    assert_eq!(align_of::<Kernel>(), 64, "Kernel struct alignment must be 64 bytes");
}

#[test]
fn test_m4_gpu_device_and_guard_safety() {
    let device = Device::init();
    assert!(device.memory() == 536_870_912); // 512 MB
    let guard = device.guard();
    assert_eq!(guard.limit(), 536_870_912);
    assert_eq!(guard.ceiling(), 429_496_729); // 409.6 MB

    // Test buffer allocation within guard limits
    let buf_res = device.allocate(1024 * 1024);
    assert!(buf_res.is_ok());
    let mut buf = buf_res.unwrap();
    assert!(buf.aligned());
    assert_eq!(buf.capacity(), 1024 * 1024);

    // Test freeing buffer
    let free_res = device.free(&mut buf);
    assert!(free_res.is_ok());
    assert_eq!(guard.allocated(), 0);

    // Test exceeding ceiling
    let overflow_res = device.allocate(500 * 1024 * 1024);
    assert!(overflow_res.is_err());
    assert_eq!(overflow_res.err(), Some(Status::Exhausted));
}

#[test]
fn test_m4_gpu_continuous_execution_zero_panics() {
    let pos = Parser::parse(Parser::DEFAULT);
    let sample = Sample::pack(&pos, 1);

    // Stress test 100 iterations of Gym submit and process
    let mut gym = Gym::init().expect("Gym init failed");
    assert!(gym.active());

    for i in 0..100 {
        let item = Sample::pack(&pos, i as u32);
        gym.submit(&item).expect("Submit failed");
    }
    let count = gym.process().expect("Process failed");
    assert_eq!(count, 100);
    assert_eq!(gym.count(), 100);

    // Test fallback CPU batch evaluation
    let mut batch = Batch::allocate(gym.device(), 100).expect("Batch allocate failed");
    for i in 0..50 {
        let item = Sample::pack(&pos, i as u32);
        batch.push(&item).expect("Push to batch failed");
    }
    gym.evaluator().fallback(&mut batch, 50).expect("Fallback failed");
    assert_eq!(batch.count(), 50);
}

#[test]
fn test_m4_gpu_multi_threaded_buffer_ring_stress() {
    let buffer = Arc::new(Buffer::allocate(131072, false).expect("Buffer allocation failed")); // 128 KB
    let threads = 8;
    let iterations = 1_000;
    let mut handles = Vec::with_capacity(threads);

    for thread_id in 0..threads {
        let buf = buffer.clone();
        let handle = thread::spawn(move || {
            let payload = [(thread_id as u8) + 1; 32];
            for _ in 0..iterations {
                let push_res = buf.push(&payload);
                if push_res.is_ok() {
                    let mut target = [0u8; 32];
                    let _ = buf.pull(&mut target);
                }
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().expect("Thread panicked");
    }
}
