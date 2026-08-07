// Thử nghiệm thực nghiệm độc lập và stress-test hiệu năng đa luồng CQRS Bus & Queue
// Tác giả: challenger_m6_1 (M6 CQRS Bus Stress & Alignment Challenger)
// 100% chú thích Tiếng Việt, 100% định danh mã nguồn từ đơn tiếng Anh (Single-Word English Identifiers).

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use xiangrust::cqrs::{Bus, Command, Event, Item, Kind, Query, Queue, Store};

/// Kiểm tra căn lề bộ nhớ 64-byte để triệt tiêu False Sharing giữa các luồng
#[test]
fn align() {
    assert_eq!(std::mem::align_of::<Bus>(), 64);
    assert_eq!(std::mem::align_of::<Queue>(), 64);
    assert_eq!(std::mem::align_of::<Store>(), 64);
    assert_eq!(std::mem::align_of::<Item>(), 64);
}

/// Thử nghiệm đơn luồng các thao tác CQRS Bus, Queue, Store
#[test]
fn single() {
    let bus = Bus::new(1024, 65536);
    assert!(bus.send(Command::Stop));
    assert!(bus.emit(Event::Ready));
    let res = bus.ask(Query::Stats);
    assert!(res.is_some());

    let first = bus.poll();
    assert!(first.is_some());
    assert_eq!(first.unwrap().kind, Kind::Command);

    let second = bus.poll();
    assert!(second.is_some());
    assert_eq!(second.unwrap().kind, Kind::Event);

    assert_eq!(bus.store.len(), 3);
}

/// Stress test 16 luồng đồng thời gửi thông điệp (send, emit, ask) tới Bus
#[test]
fn producers() {
    let bus = Arc::new(Bus::new(65536, 131072));
    let threads = 16;
    let count = 1000;

    println!("\n=== [CHALLENGER M6] STRESS TEST 16 LUỒNG ĐỒNG THỜI GỬI THÔNG ĐIỆP ===");
    let start = Instant::now();
    let mut handles = Vec::with_capacity(threads);

    for _ in 0..threads {
        let bus = Arc::clone(&bus);
        let handle = thread::spawn(move || {
            for i in 0..count {
                if i % 3 == 0 {
                    bus.send(Command::Stop);
                } else if i % 3 == 1 {
                    bus.emit(Event::Ready);
                } else {
                    bus.ask(Query::Eval);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed = start.elapsed();
    let total = (threads * count) as u64;
    let rate = if elapsed.as_millis() > 0 {
        (total * 1000) / (elapsed.as_millis() as u64)
    } else {
        total * 1000
    };

    println!(
        "Luồng: {} | Tổng thông điệp: {} | Thời gian: {:?} | Tốc độ: {} msg/sec | Store len: {} | Queue len: {}",
        threads,
        total,
        elapsed,
        rate,
        bus.store.len(),
        bus.queue.len()
    );

    assert_eq!(bus.store.len(), total as usize, "Số lượng thông điệp ghi nhận trong Store không khớp!");
}

/// Stress test kiểm tra race condition trực tiếp trên Queue với 16 luồng push đồng thời
#[test]
fn queue() {
    let queue = Arc::new(Queue::new(65536));
    let threads = 16;
    let count = 1000;

    println!("\n=== [CHALLENGER M6] KIỂM TRA RACE CONDITION THỰC NGHIỆM TRÊN QUEUE (16 LUỒNG PUSH) ===");
    let mut handles = Vec::with_capacity(threads);

    for t in 0..threads {
        let queue = Arc::clone(&queue);
        let handle = thread::spawn(move || {
            let mut pushed = 0;
            for i in 0..count {
                let item = Item {
                    id: (t * count + i) as u64,
                    stamp: 0,
                    kind: Kind::Command,
                    data: format!("{}:{}", t, i),
                };
                if queue.push(item) {
                    pushed += 1;
                }
            }
            pushed
        });
        handles.push(handle);
    }

    let mut total = 0;
    for handle in handles {
        total += handle.join().unwrap();
    }

    let qlen = queue.len();
    println!("Push thành công (theo return true): {}", total);
    println!("Queue len thực tế: {}", qlen);

    let mut popped = 0;
    while queue.pop().is_some() {
        popped += 1;
    }
    println!("Tổng số item pop() thành công: {}", popped);

    assert_eq!(total, threads * count, "Toàn bộ push() phải trả về true vì capacity đủ lớn!");
    assert_eq!(qlen, total, "CHÚ Ý: Queue len ({}) KHÔNG KHỚP với tổng số item đã push ({})! Race condition trong Queue::push!", qlen, total);
    assert_eq!(popped, total, "CHÚ Ý: Số item pop() được ({}) KHÔNG KHỚP với tổng số item đã push ({})!", popped, total);
}

/// Stress test 16 luồng vừa gửi vừa nhận thông điệp (Duplex producers & consumers)
#[test]
fn duplex() {
    let bus = Arc::new(Bus::new(65536, 131072));
    let producers = 8;
    let consumers = 8;
    let count = 2000;

    println!("\n=== [CHALLENGER M6] DUPLEX STRESS TEST 8 LUỒNG PUSH + 8 LUỒNG POP ===");
    let start = Instant::now();
    let mut prod_handles = Vec::with_capacity(producers);
    let mut cons_handles = Vec::with_capacity(consumers);

    for _ in 0..producers {
        let bus = Arc::clone(&bus);
        let handle = thread::spawn(move || {
            for _ in 0..count {
                bus.send(Command::Stop);
                bus.emit(Event::Ready);
            }
        });
        prod_handles.push(handle);
    }

    for _ in 0..consumers {
        let bus = Arc::clone(&bus);
        let handle = thread::spawn(move || {
            let mut popped = 0;
            let timeout = Instant::now();
            while timeout.elapsed() < Duration::from_millis(500) {
                if bus.poll().is_some() {
                    popped += 1;
                }
                thread::yield_now();
            }
            popped
        });
        cons_handles.push(handle);
    }

    for handle in prod_handles {
        handle.join().unwrap();
    }

    let mut total_popped = 0;
    for handle in cons_handles {
        total_popped += handle.join().unwrap();
    }

    let elapsed = start.elapsed();
    println!(
        "Hoàn thành Duplex test trong {:?} | Store len: {} | Queue len: {} | Consumers popped: {}",
        elapsed,
        bus.store.len(),
        bus.queue.len(),
        total_popped
    );

    assert_eq!(bus.store.len(), (producers * count * 2) as usize);
}

/// Test Event Sourcing Store record, fetch, clear
#[test]
fn store() {
    let store = Arc::new(Store::new(10000));
    let threads = 16;
    let count = 500;
    let mut handles = Vec::with_capacity(threads);

    for t in 0..threads {
        let store = Arc::clone(&store);
        let handle = thread::spawn(move || {
            for i in 0..count {
                let item = Item {
                    id: (t * count + i) as u64,
                    stamp: 100,
                    kind: Kind::Event,
                    data: String::from("test"),
                };
                store.record(item);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(store.len(), threads * count);
    let items = store.fetch();
    assert_eq!(items.len(), threads * count);

    store.clear();
    assert_eq!(store.len(), 0);
}
