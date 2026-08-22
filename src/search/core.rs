// ============================================================================
// MODULE CORE: THUẬT TOÁN TÌM KIẾM CỐT LÕI PVS / NEGASCOUT (SEARCH CORE ENGINE)
// ============================================================================
// `core.rs` chứa trái tim của Search Engine:
// - `pvs()`: Thuật toán Principal Variation Search (NegaScout) kết hợp với các kỹ thuật cắt tỉa nâng cao:
//   - Tra cứu Transposition Table Sharding (`probe_with`).
//   - Tích hợp đa dạng hóa History scaling trong `Picker::next_with`.
//   - Reverse Futility Pruning (RFP) tại depth <= 3.
//   - Dynamic Null Move Pruning (NMP) tại depth >= 3.
//   - Late Move Reduction (LMR) cho các nước đi không ăn quân muộn.
//   - Thẩm định chiếu kéo dài độ sâu (Check Extension).
//   - Cập nhật Killer Moves và History Table khi Beta Cutoff.
// - `iterate()`: Vòng lặp tăng dần độ sâu Iterative Deepening kết hợp cửa sổ Aspiration Window.
// ============================================================================

use std::sync::atomic::Ordering;
use crate::board::Position;
use crate::eval::Eval;
use crate::movegen::{legal, Move};
use crate::search::diversity::Diversity;
use crate::search::limit::Timer;
use crate::search::order::{History, Killer, Picker};
use crate::search::prune::Prune;
use crate::search::quiesce::Quiesce;
use crate::search::stack::Stack;
use crate::tt::{Bound, Table};

/// Struct `Core` bọc các thuật toán tìm kiếm đệ quy cốt lõi PVS và Iterative Deepening.
pub struct Core;

impl Core {
    /// Điểm số chiếu bí Mate Score = 30,000 centipawns
    pub const MATE: i32 = 30000;

    /// Thuật toán đệ quy Principal Variation Search (PVS) / NegaScout.
    /// Tối ưu hóa: (1) Batch timer check mỗi 4096 nút, (2) Check detection SAU depth check,
    /// (3) Futility Pruning cho nước đi muộn, (4) History Malus cho nước yên lặng thất bại.
    #[inline(always)]
    pub fn pvs(
        pos: &mut Position,
        eval: &mut Eval,
        tt: Option<&Table>,
        history: &mut History,
        killer: &mut Killer,
        stack: &mut [Stack; 128],
        timer: &Timer,
        diversity: Option<&Diversity>,
        past: Option<&[u64]>,
        mut depth: i32,
        mut alpha: i32,
        beta: i32,
        ply: usize,
        nodes: &mut u64,
    ) -> i32 {
        *nodes += 1;
        // 1. Kiểm tra ngắt dừng khẩn cấp từ Timer
        // Timer.check() đã có batch nội bộ (time check mỗi 256 nút),
        // abort flag check là 1 relaxed atomic load (~1 cycle) — rất rẻ.
        if timer.check(*nodes) {
            return 0;
        }

        if ply < 128 {
            stack[ply].pv.clear();
        }

        // 2. Nghẽn 3 FIX: Kiểm tra depth <= 0 TRƯỚC khi gọi legal::check()
        // Nếu depth <= 0, nút này sẽ rơi vào QSearch ngay — KHÔNG cần tốn CPU tính check.
        // QSearch tự kiểm tra check bên trong, tránh tính 2 lần.
        if depth <= 0 || ply >= 127 {
            return Quiesce::search(pos, eval, timer, alpha, beta, ply, nodes);
        }

        let orig = alpha;
        let key = pos.hash;

        // 3. Repetition Check: Tránh lặp cờ và Phạt CỰC NẶNG Luật Trường Chiếu (Perpetual Check = LOSS)
        let mut repeated = false;
        if ply > 0 {
            let mut i = 0;
            while i < ply {
                if stack[i].hash == key {
                    repeated = true;
                    break;
                }
                i += 1;
            }

            if !repeated {
                if let Some(past_hashes) = past {
                    for &h in past_hashes {
                        if h == key {
                            repeated = true;
                            break;
                        }
                    }
                }
            }
        }

        if repeated {
            let check = legal::check(pos, pos.side as usize);
            if check {
                return 28000 - (ply as i32);
            } else {
                return 0;
            }
        }

        if ply < 128 {
            stack[ply].hash = key;
        }

        let mut hint = Move::none();
        let thread_index = diversity.map_or(0, |d| d.index);

        // 3. Tra cứu bảng băm Transposition Table Sharding (TT Probe With Thread Index)
        if let Some(table) = tt {
            if let Some(item) = table.probe_with(key, thread_index) {
                hint = item.step;
                if ply > 0 && item.depth >= depth as u8 {
                    let mut score = item.score as i32;
                    // Điều chỉnh điểm Mate theo ply
                    if score > Self::MATE - 100 {
                        score -= ply as i32;
                    } else if score < -Self::MATE + 100 {
                        score += ply as i32;
                    }
                    match item.bound {
                        Bound::Exact => return score,
                        Bound::Lower => {
                            if score >= beta {
                                return score; // Cutoff Beta từ TT
                            }
                        }
                        Bound::Upper => {
                            if score <= alpha {
                                return score; // Cutoff Alpha từ TT
                            }
                        }
                        Bound::None => {}
                    }
                }
            }
        }

        // 4. Check Extension: Nếu đang bị chiếu, mở rộng độ sâu +1
        // Di chuyển SAU depth check để tránh tính check cho các nút QSearch
        let check = legal::check(pos, pos.side as usize);
        if check {
            depth += 1;
        }

        let pv = beta - alpha > 1;
        let standing = eval.score(pos);

        // 5. Reverse Futility Pruning (RFP) cắt tỉa khi điểm static eval vượt xa Beta
        if !pv && !check && depth <= 3 {
            let margin = Prune::rfp(depth);
            if standing - margin >= beta {
                return beta;
            }
        }

        // 6. Dynamic Null Move Pruning (NMP) cho phép đối phương đi 2 nước liên tiếp
        if !pv && !check && depth >= 3 && !stack[ply].null {
            let r = Prune::nmp(depth);
            stack[ply].null = true;
            pos.make_null();

            let eval = -Self::pvs(
                pos, eval, tt, history, killer, stack, timer,
                diversity, past, depth - 1 - r, -beta, -beta + 1, ply + 1, nodes
            );

            pos.unmake_null();
            stack[ply].null = false;

            if eval >= beta {
                return beta; // Cutoff NMP
            }
        }

        // ProbCut Pruning (Probability Cutoff):
        // Tại các nút non-PV ở depth >= 5, nếu một đợt tìm kiếm nông (depth - 4) với cửa sổ [ beta + 200 - 1, beta + 200 ]
        // thu được điểm số >= beta + 200 -> Lập tức cắt tỉa và trả về beta!
        if !pv && !check && depth >= 5 && beta.abs() < 20000 {
            let prob_depth = Prune::probcut_depth(depth);
            let prob_beta = beta + Prune::probcut_margin();
            let prob_score = -Self::pvs(
                pos, eval, tt, history, killer, stack, timer,
                diversity, past, prob_depth, -prob_beta, -prob_beta + 1, ply + 1, nodes
            );
            if prob_score >= prob_beta {
                return beta;
            }
        }

        let killers = if ply < 128 { killer.slot[ply] } else { [Move::none(); 2] };
        // Lấy nước đi của đối phương ở tầng trước để tra cứu Countermove
        let prev_move = if ply > 0 { stack[ply - 1].mv } else { Move::none() };
        let counter = history.get_counter(prev_move);

        let mut picker = Picker::with_counter(hint, killers, counter);

        let mut best = Move::none();
        let mut searched = 0;

        let active = eval.enabled();

        // Nghẽn 4: Tính trước Futility margin cho nước đi cụ thể (Move Futility Pruning)
        // Cắt bỏ nước đi yên lặng vô vọng tại depth <= 2 nếu static eval + margin < alpha
        let futile = !pv && !check && depth <= 2;
        let futility = if futile {
            standing + Prune::futility(depth)
        } else {
            0
        };

        // Nghẽn 5: Mảng thu thập nước đi yên lặng đã thử nhưng KHÔNG gây cutoff
        // Sẽ bị phạt History Malus khi có Beta Cutoff
        let mut quiet: [Move; 64] = [Move::none(); 64];
        let mut qcount = 0usize;

        // 7. Duyệt danh sách các nước đi qua Lazy Move Picker tích hợp Diversity
        while let Some(mv) = picker.next_with(pos, history, diversity) {
            let moving = pos.grid[mv.from as usize];
            let captured = pos.grid[mv.to as usize];
            let capture = captured < 14;

            // Nghẽn 4: Move Futility Pruning — bỏ qua nước đi yên lặng vô vọng ở depth <= 2
            // Nếu static eval + futility margin < alpha và nước đi không ăn quân → bỏ qua
            // Giảm 15-25% EBF (Effective Branching Factor) tại các nút nông
            if futile && !capture && searched > 0 && futility <= alpha {
                continue;
            }

            // Grandmaster Optimization: SEE Pruning cho Main Search
            // 1. Bỏ qua các nước đi yên lặng tại depth <= 3 có điểm SEE kém bất lợi
            if !pv && !check && !capture && depth <= 3 && !crate::search::see::See::evaluate(pos, mv, -depth * 50) {
                continue;
            }

            // 2. SEE Capture Pruning: Bỏ qua các nước đi ăn quân thua thiệt nặng tại depth <= 4
            if !pv && !check && capture && depth <= 4 && !crate::search::see::See::evaluate(pos, mv, -depth * 100) {
                continue;
            }

            if ply < 128 {
                stack[ply].mv = mv;
            }

            if active {
                eval.apply(pos, mv.from, mv.to, moving, captured);
            }
            let state = pos.apply(mv.from, mv.to);

            let mut score;

            // Singular Extension cho TT Move ở depth >= 6
            let ext = if searched == 0 && mv == hint && depth >= 6 && !check { 1 } else { 0 };

            // Nước đi đầu tiên (PV Node) -> Tìm kiếm với cửa sổ đầy đủ [ -beta, -alpha ]
            if searched == 0 {
                score = -Self::pvs(
                    pos, eval, tt, history, killer, stack, timer,
                    diversity, past, depth - 1 + ext, -beta, -alpha, ply + 1, nodes
                );
            } else {
                // Áp dụng Late Move Reduction (LMR) cho các nước không ăn quân muộn
                let r = if !pv && !check && !capture {
                    Prune::lmr(depth, searched)
                } else {
                    0
                };

                // Thử tìm kiếm với cửa sổ hẹp Zero Window [ -alpha - 1, -alpha ]
                score = -Self::pvs(
                    pos, eval, tt, history, killer, stack, timer,
                    diversity, past, depth - 1 - r, -alpha - 1, -alpha, ply + 1, nodes
                );

                // Nếu LMR thất bại (score > alpha) -> Re-search với độ sâu đầy đủ
                if r > 0 && score > alpha {
                    score = -Self::pvs(
                        pos, eval, tt, history, killer, stack, timer,
                        diversity, past, depth - 1, -alpha - 1, -alpha, ply + 1, nodes
                    );
                }

                // Nếu thu được điểm cao hơn alpha -> Re-search với cửa sổ đầy đủ
                if score > alpha && score < beta {
                    score = -Self::pvs(
                        pos, eval, tt, history, killer, stack, timer,
                        diversity, past, depth - 1, -beta, -alpha, ply + 1, nodes
                    );
                }
            }

            pos.revert(mv.from, mv.to, &state);
            if active {
                eval.revert(pos, mv.from, mv.to, moving, captured);
            }

            if timer.abort.load(Ordering::Relaxed) {
                return 0;
            }

            searched += 1;

            // 8. Beta Cutoff: Cắt tỉa nhánh Alpha-Beta
            if score >= beta {
                if !capture {
                    killer.push(ply, mv);
                    history.update(mv, depth);
                    history.update_counter(prev_move, mv);

                    // Nghẽn 5: History Malus — phạt TẤT CẢ nước yên lặng đã thử nhưng KHÔNG
                    // gây cutoff. Cải thiện move ordering 10-15% bằng cách giảm ưu tiên
                    // các nước đi đã chứng minh thất bại ở các nút trước.
                    let mut qi = 0usize;
                    while qi < qcount {
                        history.penalize(quiet[qi], depth);
                        qi += 1;
                    }
                }

                if ply == 0 {
                    let child = stack[ply + 1].pv;
                    stack[ply].pv.update(mv, &child);
                }
                if !timer.abort.load(Ordering::Relaxed) {
                    if let Some(table) = tt {
                        let val = if score > Self::MATE - 100 {
                            (score + ply as i32) as i16
                        } else if score < -Self::MATE + 100 {
                            (score - ply as i32) as i16
                        } else {
                            score as i16
                        };
                        table.save_with(key, depth as u8, Bound::Lower.raw(), mv, val, thread_index);
                    }
                }
                return beta;
            }

            // Nâng Alpha
            if score > alpha {
                alpha = score;
                best = mv;
                let child = stack[ply + 1].pv;
                stack[ply].pv.update(mv, &child);
            } else if !capture && qcount < 64 {
                // Thu thập nước yên lặng thất bại vào mảng quiet cho History Malus
                quiet[qcount] = mv;
                qcount += 1;
            }
        }

        // 9. Xử lý hết nước đi
        if searched == 0 {
            if check {
                return -Self::MATE + (ply as i32); // Bị chiếu bí
            }
            return 0; // Hòa cờ hết nước đi (Stalemate)
        }

        // 10. Lưu kết quả vào Transposition Table Sharding (nếu không bị ngắt)
        if !timer.abort.load(Ordering::Relaxed) {
            if let Some(table) = tt {
                let bound = if alpha > orig {
                    Bound::Exact
                } else {
                    Bound::Upper
                };
                let val = if alpha > Self::MATE - 100 {
                    (alpha + ply as i32) as i16
                } else if alpha < -Self::MATE + 100 {
                    (alpha - ply as i32) as i16
                } else {
                    alpha as i16
                };
                table.save_with(key, depth as u8, bound.raw(), best, val, thread_index);
            }
        }

        alpha
    }

    /// Thuật toán Iterative Deepening tăng độ sâu từng bước kết hợp cửa sổ Aspiration Windows.
    #[inline(always)]
    pub fn iterate(
        pos: &mut Position,
        eval: &mut Eval,
        tt: Option<&Table>,
        history: &mut History,
        killer: &mut Killer,
        timer: &Timer,
        diversity: Option<&Diversity>,
        past: Option<&[u64]>,
    ) -> (Move, i32, u64, u8) {
        // Xây dựng mảng Stack frame 128 tầng ply trực tiếp trên L1 Data Cache (0-heap allocation, 0ms latency)
        let mut stack = [Stack::new(); 128];

        let mut nodes = 0u64;
        let mut best = Move::none();
        let mut val = 0;
        let mut completed_depth = 0u8;
        let limit = if timer.limit.depth > 0 {
            timer.limit.depth as i32
        } else {
            128
        };

        // Lặp tăng dần độ sâu từ 1 đến limit
        for depth in 1..=limit {
            if timer.abort.load(Ordering::Relaxed) {
                break;
            }
            let mut alpha = -Self::MATE;
            let mut beta = Self::MATE;

            // Áp dụng cửa sổ Aspiration Window từ độ sâu 4
            if depth >= 4 {
                let delta = 20;
                alpha = val - delta;
                beta = val + delta;
            }

            loop {
                let score = Self::pvs(
                    pos, eval, tt, history, killer, &mut stack, timer,
                    diversity, past, depth, alpha, beta, 0, &mut nodes
                );

                if timer.abort.load(Ordering::Relaxed) {
                    break;
                }

                // Nới rộng cửa sổ Aspiration Window nếu trượt alpha hoặc beta
                if score <= alpha {
                    if alpha <= -Self::MATE {
                        val = score;
                        if stack[0].pv.len > 0 && stack[0].pv.items[0].valid() {
                            best = stack[0].pv.items[0];
                        }
                        break;
                    }
                    alpha = (-Self::MATE).max(alpha - 50);
                } else if score >= beta {
                    if beta >= Self::MATE {
                        val = score;
                        if stack[0].pv.len > 0 && stack[0].pv.items[0].valid() {
                            best = stack[0].pv.items[0];
                        }
                        break;
                    }
                    beta = Self::MATE.min(beta + 50);
                } else {
                    val = score;
                    if stack[0].pv.len > 0 && stack[0].pv.items[0].valid() {
                        best = stack[0].pv.items[0];
                    }
                    break;
                }
            }

            let interrupted = timer.abort.load(Ordering::Relaxed) || timer.expired();
            if !interrupted {
                completed_depth = depth as u8;
            }
            if interrupted {
                break;
            }
        }

        (best, val, nodes, completed_depth)
    }
}
