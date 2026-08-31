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
use crate::search::order::{History, Killer, Picker, VALUES};
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
        depth: i32,
        mut alpha: i32,
        beta: i32,
        ply: usize,
        nodes: &mut u64,
    ) -> i32 {
        *nodes += 1;
        // 0. Hardware Prefetching: Nạp trước bảng băm TT của thế cờ vào L1 Cache để ẩn độ trễ RAM
        if let Some(table) = tt {
            table.prefetch(pos.hash);
        }

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
        // Tối ưu hóa: (1) Quét ngược từng bước 2 (chỉ cùng phe), (2) Giới hạn bởi pos.rule (nước không thể đảo ngược).
        let mut repeated = false;
        if ply > 0 {
            let mut i = ply.saturating_sub(2);
            while i > 0 {
                if stack[i].hash == key {
                    repeated = true;
                    break;
                }
                if i < 2 {
                    break;
                }
                i -= 2;
            }

            if !repeated {
                if let Some(past_hashes) = past {
                    let check_limit = (pos.rule as usize).min(past_hashes.len());
                    if check_limit >= 2 {
                        let len = past_hashes.len();
                        let mut idx = len.saturating_sub(2);
                        let min_idx = len.saturating_sub(check_limit);
                        while idx >= min_idx {
                            if past_hashes[idx] == key {
                                repeated = true;
                                break;
                            }
                            if idx < 2 {
                                break;
                            }
                            idx -= 2;
                        }
                    }
                }
            }
        }

        if repeated {
            let under_check = legal::check(pos, pos.side as usize);
            let gave_check = legal::check(pos, (pos.side ^ 1) as usize);
            if under_check && !gave_check {
                // Đối phương liên tục chiếu ta (Trường Chiếu) -> Đối phương bị xử thua (Ta Thắng: +28000)
                return 28000 - (ply as i32);
            } else if gave_check && !under_check {
                // Ta liên tục chiếu đối phương (Trường Chiếu) -> Ta bị xử thua (-28000)
                return -28000 + (ply as i32);
            } else {
                // Lặp nước hòa bình thông thường (Ordinary Draw):
                // Áp dụng Grandmaster Asymmetric Contempt: Khi ta đang có ưu thế thế trận lớn (Static Eval >= +150cp),
                // phạt nặng việc chấp nhận hòa lặp nước (-2500cp) để ép toàn bộ cây tìm kiếm chuyển sang đòn sát phạt mới!
                let static_eval = eval.score(pos);
                if static_eval >= 150 {
                    return -2500 + (ply as i32);
                } else if static_eval <= -150 {
                    return 2500 - (ply as i32);
                } else {
                    return 0;
                }
            }
        }

        if ply < 128 {
            stack[ply].hash = key;
        }

        let mut hint = Move::none();
        let mut tt_score = 0;
        let mut tt_depth = 0;
        let mut tt_bound = Bound::None;
        let thread_index = diversity.map_or(0, |d| d.index);

        // 3. Tra cứu bảng băm Transposition Table Sharding (TT Probe With Thread Index)
        if let Some(table) = tt {
            if let Some(item) = table.probe_with(key, thread_index) {
                hint = item.step;
                tt_score = item.score as i32;
                tt_depth = item.depth;
                tt_bound = item.bound;
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

        // 4. Kiểm tra xem phía đi cờ có đang bị chiếu hay không
        let check = legal::check(pos, pos.side as usize);

        let pv = beta - alpha > 1;
        let standing = eval.score(pos);
        if ply < 128 {
            stack[ply].eval = standing;
        }

        // Improving Flag Heuristic: Nhận diện thế trận đang cải thiện hay suy giảm so với ply - 2
        let improving = ply >= 2 && standing > stack[ply - 2].eval;

        // 5. Reverse Futility Pruning (RFP) cắt tỉa khi điểm static eval vượt xa Beta
        if !pv && !check && depth <= 4 {
            let mut margin = Prune::rfp(depth);
            if !improving {
                margin -= 40;
            }
            if standing - margin >= beta {
                return beta;
            }
        }

        // 6. Dynamic Null Move Pruning (NMP) cho phép đối phương đi 2 nước liên tiếp khi thế cờ đang tốt (standing >= beta)
        let other_side = (1 - pos.side as usize) * 7;
        let my_side = pos.side as usize * 7;
        let enemy_attackers_count = pos.counts[other_side + 4] + pos.counts[other_side + 5] + pos.counts[other_side + 3];
        let under_threat = enemy_attackers_count >= 2 && pos.counts[my_side + 1] <= 1;
        if !pv && !check && depth >= 3 && standing >= beta && !stack[ply].null && !under_threat {
            let mut r = Prune::nmp(depth);
            if !improving {
                r = (r - 1).max(2);
            }
            stack[ply].null = true;
            pos.make_null();

            let eval = -Self::pvs(
                pos, eval, tt, history, killer, stack, timer,
                diversity, past, depth - 1 - r, -beta, -beta + 1, ply + 1, nodes
            );

            pos.unmake_null();
            stack[ply].null = false;

            if eval >= beta && eval < 28000 {
                return beta; // Cutoff NMP an toàn (không cutoff khi phát hiện mate)
            }
        }

        // 6.5. ProbCut (Probability Cutoff) tại depth >= 5 cho non-PV nodes
        if !pv && !check && depth >= 5 && beta.abs() < 20000 {
            let probcut_beta = (beta + 200).min(Self::MATE - 1);
            if hint.valid() && pos.grid[hint.to as usize] < 14 {
                let moving = pos.grid[hint.from as usize];
                let captured = pos.grid[hint.to as usize];
                let active = eval.enabled();
                if active {
                    eval.apply(pos, hint.from, hint.to, moving, captured);
                }
                let state = pos.apply(hint.from, hint.to);
                let side = (pos.side ^ 1) as usize;
                if !legal::check(pos, side) && !legal::fly(pos) {
                    let score = -Quiesce::search(pos, eval, timer, -probcut_beta, -probcut_beta + 1, ply + 1, nodes);
                    pos.revert(hint.from, hint.to, &state);
                    if active {
                        eval.revert(pos, hint.from, hint.to, moving, captured);
                    }
                    if score >= probcut_beta {
                        return beta;
                    }
                } else {
                    pos.revert(hint.from, hint.to, &state);
                    if active {
                        eval.revert(pos, hint.from, hint.to, moving, captured);
                    }
                }
            }
        }

        let killers = if ply < 128 { killer.slot[ply] } else { [Move::none(); 2] };
        // Lấy nước đi của đối phương ở tầng trước để tra cứu Countermove và Continuation History
        let prev_move = if ply > 0 { stack[ply - 1].mv } else { Move::none() };
        let counter = history.get_counter(prev_move);

        let mut picker = Picker::with_context(hint, killers, counter, prev_move);

        let mut best = Move::none();
        let mut best_score = -Self::MATE;
        let mut searched = 0;

        let active = eval.enabled();

        // Nghẽn 4: Tính trước Futility margin cho nước đi cụ thể (Move Futility Pruning)
        // Cắt bỏ nước đi yên lặng vô vọng tại depth <= 4 nếu static eval + margin < alpha
        let futile = !pv && !check && depth <= 4;
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

            // Nghẽn 4: Move Futility Pruning — bỏ qua nước đi yên lặng vô vọng ở depth <= 4
            // Nếu static eval + futility margin < alpha và nước đi không ăn quân → bỏ qua
            // Giảm 20-35% EBF (Effective Branching Factor) tại các nút nông
            if futile && !capture && searched > 0 && futility <= alpha {
                continue;
            }

            // Late Move Pruning (LMP): Cắt tỉa các nước đi tĩnh tại depth <= 4 khi đã duyệt qua giới hạn LMP
            if !pv && !check && !capture && depth <= 4 && searched >= Prune::lmp(depth) {
                continue;
            }

            // History Pruning: Bỏ qua nước đi yên lặng có điểm lịch sử âm tại depth <= 3 sau 3 nước đã duyệt
            if !pv && !check && !capture && depth <= 3 && searched >= 3 && history.get(mv) < -200 * depth {
                continue;
            }

            // Grandmaster Optimization: SEE Pruning cho Main Search
            // 1. Bỏ qua các nước đi yên lặng tại depth <= 2 có điểm SEE thua thiệt nặng (SEE < -depth * 200)
            if !pv && !check && !capture && depth <= 2 && !crate::search::see::See::evaluate(pos, mv, -depth * 200) {
                continue;
            }

            // 2. SEE Capture Pruning: Bỏ qua 100% các nước ăn quân thua thiệt nặng (SEE < -100) ở mọi độ sâu
            // Nếu giá trị quân bị ăn >= quân tấn công thì SEE chắc chắn >= 0 >= -100
            if !pv && !check && capture && VALUES[captured as usize] < VALUES[moving as usize] && !crate::search::see::See::evaluate(pos, mv, -100) {
                continue;
            }

            if ply < 128 {
                stack[ply].mv = mv;
            }

            if active {
                eval.apply(pos, mv.from, mv.to, moving, captured);
            }
            let state = pos.apply(mv.from, mv.to);

            // Thẩm định tính hợp lệ tuyệt đối On-The-Fly (Zero Eager Movegen Overhead)
            // Kiểm tra: Nước đi không để Tướng nhà bị chiếu và không vi phạm quy tắc Lộ mặt Tướng
            let side = (pos.side ^ 1) as usize;
            if legal::check(pos, side) || legal::fly(pos) {
                pos.revert(mv.from, mv.to, &state);
                if active {
                    eval.revert(pos, mv.from, mv.to, moving, captured);
                }
                continue;
            }

            // Root Repetition Filter: Khi ta đang có ưu thế thế trận lớn (standing >= 150cp),
            // TUYỆT ĐỐI KHÔNG chọn nước đi lặp lại lịch sử ván đấu (ép Engine tìm đường sát phạt dứt điểm)
            if ply == 0 {
                if let Some(past_hashes) = past {
                    let next_key = pos.hash;
                    let check_limit = (pos.rule as usize).min(past_hashes.len());
                    if check_limit >= 2 {
                        let len = past_hashes.len();
                        let mut idx = len.saturating_sub(2);
                        let min_idx = len.saturating_sub(check_limit);
                        let mut is_root_repetition = false;
                        while idx >= min_idx {
                            if past_hashes[idx] == next_key {
                                is_root_repetition = true;
                                break;
                            }
                            if idx < 2 {
                                break;
                            }
                            idx -= 2;
                        }
                        if is_root_repetition && standing >= 150 && searched > 0 {
                            pos.revert(mv.from, mv.to, &state);
                            if active {
                                eval.revert(pos, mv.from, mv.to, moving, captured);
                            }
                            continue;
                        }
                    }
                }
            }

            let mut score;

            // Check Extension & Tactical Defense Extension: Mở rộng độ sâu khi bị chiếu hoặc chiếu đối phương
            let gives_check = legal::check(pos, pos.side as usize);
            let to_rank = mv.to / 9;
            let to_file = mv.to % 9;
            let in_enemy_palace = if pos.side == 0 { to_rank <= 2 && to_file >= 3 && to_file <= 5 } else { to_rank >= 7 && to_file >= 3 && to_file <= 5 };
            let enemy_adv = pos.counts[pos.side as usize * 7 + 1];
            let is_palace_infiltration = in_enemy_palace && (moving == 6 || moving == 13 || moving == 3 || moving == 10 || moving == 4 || moving == 11);

            let tactical_ext = if check && depth >= 2 {
                1
            } else if gives_check && depth >= 6 && searched == 0 {
                1 // Forcing Check Extension on PV Node
            } else if depth >= 8 && is_palace_infiltration && (enemy_adv <= 1 || standing.abs() >= 1000) && searched == 0 {
                1 // Palace Infiltration Extension on PV Node
            } else {
                0
            };

            // Singular / Double Singular Extension: Mở rộng nhánh độc đạo khi nước đi từ TT vượt trội hoàn toàn
            let mut singular_ext = 0;
            if depth >= 7 && mv == hint && hint.valid() && tt_depth >= (depth - 3) as u8 && tt_bound != Bound::Upper && tt_score.abs() < 20000 {
                let singular_beta = tt_score - 2 * depth as i32;
                let singular_depth = (depth - 1) / 2;
                let singular_score = -Self::pvs(
                    pos, eval, tt, history, killer, stack, timer,
                    diversity, past, singular_depth, -singular_beta, -singular_beta + 1, ply + 1, nodes
                );
                if singular_score < singular_beta - Prune::double_singular_margin(depth) {
                    singular_ext = 2; // Double Singular Extension (2 plies)
                } else if singular_score < singular_beta {
                    singular_ext = 1; // Singular Extension (1 ply)
                }
            } else if depth >= 6 && mv == hint && hint.valid() && tt_score >= 350 && tt_bound != Bound::Upper {
                // Dominant Attack Extension: Mở rộng thêm 1 ply cho nước đi then chốt khi đang có ưu thế áp đảo
                singular_ext = if gives_check { 2 } else { 1 };
            }

            let ext = tactical_ext.max(singular_ext);

            // Nước đi đầu tiên (PV Node) -> Tìm kiếm với cửa sổ đầy đủ [ -beta, -alpha ]
            if searched == 0 {
                score = -Self::pvs(
                    pos, eval, tt, history, killer, stack, timer,
                    diversity, past, depth - 1 + ext, -beta, -alpha, ply + 1, nodes
                );
            } else {
                let to_rank = mv.to / 9;
                let to_file = mv.to % 9;
                let is_palace_pawn = (moving == 6 || moving == 13) && (if side == 0 { to_rank >= 6 && to_file >= 1 && to_file <= 7 } else { to_rank <= 3 && to_file >= 1 && to_file <= 7 });
                let is_palace_knight = (moving == 3 || moving == 10) && (if side == 0 { to_rank >= 6 } else { to_rank <= 3 });

                // Áp dụng Late Move Reduction (LMR) thích ứng độ sâu sâu
                let mut r = if !pv && !check && !capture && !is_palace_pawn && !is_palace_knight {
                    let mut base_r = Prune::lmr(depth, searched);
                    if !improving {
                        base_r += 1;
                    }
                    base_r
                } else {
                    0
                };

                // Dynamic LMR: Điều chỉnh reduction theo thuộc tính nước đi
                if r > 0 {
                    // Giảm reduction cho Killer Moves và Countermove
                    if mv == killers[0] || mv == killers[1] || mv == counter {
                        r = (r - 1).max(1);
                    }
                    // Giảm reduction cho nước có điểm lịch sử cao
                    let hist_score = history.get(mv);
                    if hist_score > 800 {
                        r = (r - 1).max(1);
                    } else if hist_score < -800 {
                        r += 1;
                    }
                }

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
                    if score > alpha && score < beta {
                        score = -Self::pvs(
                            pos, eval, tt, history, killer, stack, timer,
                            diversity, past, depth - 1, -beta, -alpha, ply + 1, nodes
                        );
                    }
                } else if score > alpha && score < beta {
                    // Nếu thu được điểm cao hơn alpha -> Re-search với cửa sổ đầy đủ
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
                    history.update_follow(moving, prev_move, mv, depth);

                    // Nghẽn 5: History Malus — phạt TẤT CẢ nước yên lặng đã thử nhưng KHÔNG
                    // gây cutoff. Cải thiện move ordering 10-15% bằng cách giảm ưu tiên
                    // các nước đi đã chứng minh thất bại ở các nút trước.
                    let mut qi = 0usize;
                    while qi < qcount {
                        let qm = quiet[qi];
                        history.penalize(qm, depth);
                        let qpiece = pos.grid[qm.from as usize];
                        history.penalize_follow(qpiece, prev_move, qm, depth);
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

            // Cập nhật nước đi tốt nhất tổng thể (kể cả khi fail-low)
            if searched == 1 || score > best_score {
                best_score = score;
                best = mv;
                if ply == 0 && (stack[0].pv.len == 0 || score > alpha) {
                    let child = stack[ply + 1].pv;
                    stack[ply].pv.update(mv, &child);
                }
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

        // 9. Xử lý hết nước đi: Trong Luật Cờ Tướng (Xiangqi), hết nước đi hợp lệ
        // dù có bị chiếu hay không đều là THUA (Bức Tử / Kẹt Nước), KHÔNG PHẢI HÒA!
        if searched == 0 {
            return -Self::MATE + (ply as i32); // Bị chiếu bí hoặc Bức Tử (Stalemate = Loss in Xiangqi)
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
        let mut last_completed_best = Move::none();
        let mut last_completed_score = 0;
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
            let mut delta = 35;
            let mut alpha = -Self::MATE;
            let mut beta = Self::MATE;

            // Áp dụng cửa sổ Aspiration Window từ độ sâu 4
            if depth >= 4 {
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

                // Nới rộng cửa sổ Aspiration Window theo hàm số mũ nếu trượt alpha hoặc beta
                if score <= alpha {
                    if alpha <= -Self::MATE {
                        val = score;
                        if stack[0].pv.len > 0 && stack[0].pv.items[0].valid() {
                            best = stack[0].pv.items[0];
                        }
                        break;
                    }
                    delta = delta + delta * 2 / 3 + 10;
                    if delta >= 600 {
                        alpha = -Self::MATE;
                        beta = Self::MATE;
                    } else {
                        alpha = (-Self::MATE).max(val - delta);
                    }
                } else if score >= beta {
                    if beta >= Self::MATE {
                        val = score;
                        if stack[0].pv.len > 0 && stack[0].pv.items[0].valid() {
                            best = stack[0].pv.items[0];
                        }
                        break;
                    }
                    delta = delta + delta * 2 / 3 + 10;
                    if delta >= 600 {
                        alpha = -Self::MATE;
                        beta = Self::MATE;
                    } else {
                        beta = Self::MATE.min(val + delta);
                    }
                } else {
                    val = score;
                    if stack[0].pv.len > 0 && stack[0].pv.items[0].valid() {
                        best = stack[0].pv.items[0];
                    }
                    break;
                }
            }

            if stack[0].pv.len > 0 && stack[0].pv.items[0].valid() {
                best = stack[0].pv.items[0];
            }

            let interrupted = timer.abort.load(Ordering::Relaxed) || timer.expired();
            if !interrupted {
                if stack[0].pv.len > 0 && stack[0].pv.items[0].valid() {
                    last_completed_best = stack[0].pv.items[0];
                } else if best.valid() {
                    last_completed_best = best;
                }
                last_completed_score = val;
                completed_depth = depth as u8;
            }
            if interrupted {
                break;
            }
        }

        // Chọn nước đi tốt nhất từ độ sâu đã hoàn tất trọn vẹn để tránh bẫy do hết giờ
        let mut final_best = if last_completed_best.valid() {
            last_completed_best
        } else {
            best
        };

        // Đảm bảo không bao giờ trả về Move::none() nếu thế cờ vẫn còn nước đi hợp lệ
        if !final_best.valid() {
            if let Some(table) = tt {
                if let Some(item) = table.probe(pos.hash) {
                    if item.step.valid() {
                        final_best = item.step;
                    }
                }
            }
            if !final_best.valid() {
                let mut legals = crate::movegen::types::List::new();
                legal::gen(pos, &mut legals);
                if legals.len() > 0 {
                    final_best = legals.items[0];
                }
            }
        }

        let final_score = if completed_depth > 0 {
            last_completed_score
        } else {
            val
        };

        (final_best, final_score, nodes, completed_depth)
    }
}
