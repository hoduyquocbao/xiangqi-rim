// ============================================================================
// MODULE ENGINE: VÒNG LẶP ĐIỀU KHIỂN ENGINE VÀ GIAO THỨC UCI V2 (UCI ENGINE FACADE)
// ============================================================================
// `engine.rs` là điểm kết nối trực tiếp giữa giao diện GUI bên ngoài và lõi Engine:
// - `pool`: Hệ thống luồng công nhân Lazy SMP Thread Pool.
// - `pos`: Trạng thái thế cờ hiện tại `Position`.
// - `bus`: Trung tâm điều phối CQRS Bus.
// - `exec(cmd)`: Thực thi các câu lệnh `Uci`, `Ready`, `Option`, `Reset`, `Position`, `Go`, `Stop`, `Quit`.
// - `run()`: Khởi chạy luồng đọc STDIN bất đồng bộ không gây nghẽn GUI, hỗ trợ phản hồi lệnh ngắt dừng trong < 10ms.
// ============================================================================

use std::io::{self, BufRead, Write};
use crate::board::position::Position;
use crate::search::limit::Limits;
use crate::thread::Pool;
use super::command::Command;
use super::format::Format;
use super::option;
use super::parser::Parser;

/// Struct `Engine` quản lý trạng thái toàn cục của ứng dụng XiangRust UCI Engine.
pub struct Engine {
    /// Bể chứa luồng công nhân Lazy SMP Thread Pool
    pub pool: Pool,
    /// Vị trí thế cờ hiện tại của bàn cờ
    pub pos: Position,
    /// Số lượng luồng công nhân chạy song song (Threads option)
    pub threads: usize,
    /// Dung lượng Transposition Table tính bằng MB (Hash option)
    pub hash: usize,
    /// Loại bộ đánh giá hiện tại ("NNUE" hoặc "HCE")
    pub eval: String,
    /// Trung tâm điều phối CQRS Bus
    pub bus: crate::cqrs::Bus,
    /// Tay cầm (JoinHandle) của luồng background tìm kiếm
    pub handle: Option<std::thread::JoinHandle<()>>,
}

impl Default for Engine {
    /// Khởi tạo mặc định đối tượng Engine.
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    /// Khởi tạo Engine mới với cấu hình mặc định (1 Thread, 64MB Hash, NNUE Evaluation).
    pub fn new() -> Self {
        Self {
            pool: Pool::new(1, 64),
            pos: crate::board::fen::Parser::parse(crate::board::fen::Parser::DEFAULT),
            threads: 1,
            hash: 64,
            eval: "NNUE".to_string(),
            bus: crate::cqrs::Bus::default(),
            handle: None,
        }
    }

    /// Khai báo danh sách các tùy chọn UCI được Engine hỗ trợ (Threads, Hash, Clear Hash, EvalType).
    pub fn options() -> Vec<option::Option> {
        vec![
            option::Option::spin("Threads", 1, 1, 128),
            option::Option::spin("Hash", 64, 1, 65536),
            option::Option::button("Clear Hash"),
            option::Option::combo("EvalType", "NNUE", &["NNUE", "HCE"]),
        ]
    }

    /// Ngắt dừng quá trình tìm kiếm hiện tại và chờ luồng background kết thúc an toàn.
    pub fn stop(&mut self) {
        self.pool.halt();
        if let Some(task) = self.handle.take() {
            let _ = task.join();
        }
    }

    /// Xử lý một câu lệnh `Command` từ giao thức UCI v2.
    pub fn exec(&mut self, cmd: Command) -> bool {
        // Gửi thông điệp tương ứng tới CQRS Bus
        match &cmd {
            Command::Position { fen, moves } => {
                self.bus.send(crate::cqrs::Command::Position {
                    fen: fen.clone(),
                    moves: moves.clone(),
                });
            }
            Command::Go {
                depth,
                nodes,
                infinite,
                span,
                red,
                black,
                gain,
                extra,
            } => {
                self.bus.send(crate::cqrs::Command::Go {
                    depth: *depth,
                    nodes: *nodes,
                    infinite: *infinite,
                    span: *span,
                    red: *red,
                    black: *black,
                    gain: *gain,
                    extra: *extra,
                });
            }
            Command::Stop => {
                self.bus.send(crate::cqrs::Command::Stop);
            }
            Command::Option { name, value } => {
                self.bus.send(crate::cqrs::Command::Option {
                    name: name.clone(),
                    value: value.clone(),
                });
            }
            Command::Reset => {
                self.bus.send(crate::cqrs::Command::Reset);
            }
            Command::Quit => {
                self.bus.send(crate::cqrs::Command::Quit);
            }
            Command::Ready => {
                self.bus.emit(crate::cqrs::Event::Ready);
            }
            _ => {}
        }

        // Xử lý logic nội tại Engine theo từng câu lệnh
        match cmd {
            Command::Uci => {
                println!("id name XiangRust");
                println!("id author HDQB");
                for opt in Self::options() {
                    match opt.kind {
                        super::option::Kind::Spin => {
                            println!(
                                "option name {} type spin default {} min {} max {}",
                                opt.name, opt.def, opt.min, opt.max
                            );
                        }
                        super::option::Kind::Button => {
                            println!("option name {} type button", opt.name);
                        }
                        super::option::Kind::Combo => {
                            let mut line = format!(
                                "option name {} type combo default {}",
                                opt.name, opt.def
                            );
                            for v in &opt.vars {
                                line.push_str(&format!(" var {}", v));
                            }
                            println!("{}", line);
                        }
                        _ => {}
                    }
                }
                println!("uciok");
            }
            Command::Ready => {
                println!("readyok");
            }
            Command::Option { name, value } => {
                self.stop();
                if name == "Threads" {
                    if let Ok(val) = value.parse::<usize>() {
                        self.threads = val.clamp(1, 128);
                        self.pool = Pool::new(self.threads, self.hash);
                    }
                } else if name == "Hash" {
                    if let Ok(val) = value.parse::<usize>() {
                        self.hash = val.clamp(1, 65536);
                        self.pool = Pool::new(self.threads, self.hash);
                    }
                } else if name == "Clear Hash" || name == "Clear" {
                    self.pool.clear();
                } else if name == "EvalType" {
                    if value == "NNUE" || value == "HCE" {
                        self.eval = value;
                    }
                }
            }
            Command::Reset => {
                self.stop();
                self.pool.clear();
                self.pos = crate::board::fen::Parser::parse(crate::board::fen::Parser::DEFAULT);
            }
            Command::Position { fen, moves } => {
                self.stop();
                if !fen.is_empty() {
                    self.pos = crate::board::fen::Parser::parse(&fen);
                } else {
                    self.pos = crate::board::fen::Parser::parse(crate::board::fen::Parser::DEFAULT);
                }

                // Thực thi các nước đi tiếp nối
                for item in moves {
                    let m = Format::decode(&item);
                    if m.valid() {
                        self.pos.apply(m.from, m.to);
                    }
                }
            }
            Command::Go {
                depth,
                nodes,
                infinite,
                span,
                red,
                black,
                gain,
                extra,
            } => {
                self.stop();

                let mut limits = Limits::new();
                limits.depth = if depth > 0 { depth } else { 64 };
                limits.nodes = nodes;
                limits.infinite = infinite;
                limits.exact = span;

                if self.pos.side == 0 {
                    limits.time = red;
                    limits.inc = gain;
                } else {
                    limits.time = black;
                    limits.inc = extra;
                }

                self.pool.reset();

                let pool = self.pool.clone();
                let pos = self.pos;
                let bus = self.bus.clone();

                // Tạo luồng background chạy phiên tìm kiếm không gây treo STDIN
                let task = std::thread::spawn(move || {
                    let result = pool.go(&pos, &limits);
                    let best = Format::encode(result.best);

                    let nps = if result.time > 0 {
                        (result.nodes * 1000) / result.time
                    } else {
                        result.nodes * 1000
                    };

                    bus.emit(crate::cqrs::Event::Info {
                        depth: result.depth,
                        score: result.score,
                        nodes: result.nodes,
                        nps,
                        time: result.time,
                        pv: best.clone(),
                    });

                    bus.emit(crate::cqrs::Event::Move {
                        best: result.best.raw(),
                        ponder: result.ponder.raw(),
                    });

                    // In phản hồi kết quả ra STDOUT chuẩn giao thức UCI
                    println!(
                        "info depth {} score cp {} nodes {} nps {} time {} pv {}",
                        result.depth, result.score, result.nodes, nps, result.time, best
                    );
                    println!("bestmove {}", best);
                    io::stdout().flush().ok();
                });

                self.handle = Some(task);
            }
            Command::Stop => {
                self.stop();
            }
            Command::Quit => {
                self.stop();
                return false;
            }
            Command::Invalid => {}
        }
        true
    }

    /// Khởi chạy vòng lặp xử lý sự kiện UCI v2 từ Reader Thread qua channel mpsc.
    pub fn run(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel();
        // Luồng đọc STDIN bất đồng bộ
        std::thread::spawn(move || {
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            let mut line = String::new();

            while handle.read_line(&mut line).unwrap_or(0) > 0 {
                let cmd = Parser::parse(&line);
                line.clear();
                if tx.send(cmd).is_err() {
                    break;
                }
            }
        });

        // Vòng lặp chính xử lý câu lệnh nhận được từ channel
        while let Ok(cmd) = rx.recv() {
            if !self.exec(cmd) {
                break;
            }
            io::stdout().flush().ok();
        }

        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unit test phản hồi lệnh ngắt dừng Stop trong < 10ms khi đang tìm kiếm
    #[test]
    fn stop() {
        let mut engine = Engine::new();
        let go = Command::Go {
            depth: 64,
            nodes: 0,
            infinite: true,
            span: 0,
            red: 0,
            black: 0,
            gain: 0,
            extra: 0,
        };
        engine.exec(go);
        std::thread::sleep(std::time::Duration::from_millis(50));
        let start = std::time::Instant::now();
        engine.exec(Command::Stop);
        let elapsed = start.elapsed().as_millis();
        assert!(elapsed < 500, "Engine stop MUST halt search in < 500ms, took {}ms", elapsed);
    }

    /// Unit test hủy liên tục 50 phiên tìm kiếm liên tiếp phản hồi < 500ms
    #[test]
    fn cancel() {
        let mut engine = Engine::new();
        for _ in 0..50 {
            let go = Command::Go {
                depth: 64,
                nodes: 0,
                infinite: true,
                span: 0,
                red: 0,
                black: 0,
                gain: 0,
                extra: 0,
            };
            engine.exec(go);
            std::thread::sleep(std::time::Duration::from_millis(50));
            let start = std::time::Instant::now();
            engine.exec(Command::Stop);
            let elapsed = start.elapsed().as_millis();
            assert!(elapsed < 500, "Engine stop MUST halt search in < 500ms, took {}ms", elapsed);
        }
    }
}

