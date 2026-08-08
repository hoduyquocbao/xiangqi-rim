fn main() {
    println!("============================================================");
    println!(" JRCP 3.0 × 64GB RAM ELITE DATA MINER");
    println!("============================================================");

    let total_games: usize = std::env::var("GAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(100000);
    let depth: u8 = std::env::var("DEPTH").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
    let num_threads: usize = std::env::var("THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(12);
    let tt_mb: usize = std::env::var("TT_MB").ok().and_then(|v| v.parse().ok()).unwrap_or(2048);
    let sieve_mb: usize = std::env::var("SIEVE_MB").ok().and_then(|v| v.parse().ok()).unwrap_or(32768);
    let base_seed: u64 = std::env::var("SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let output_path: String = std::env::var("OUTPUT").unwrap_or_else(|_| format!("data/hf_space/jrcp3_ram64g_{}_{}.jsonl", base_seed, stamp));

    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        let _ = fs::create_dir_all(parent);
    }

    let tt_total_gb = (tt_mb as f64 * num_threads as f64) / 1024.0;
    let sieve_gb = sieve_mb as f64 / 1024.0;
    let search_overhead_gb = num_threads as f64 * 50.0 / 1024.0;
    let total_ram_gb = tt_total_gb + sieve_gb + search_overhead_gb + 2.0;

    println!("⚡ Cấu hình JRCP 3.0 × 64GB RAM:");
    println!("   Target Games  : {}", total_games);
    println!("   Search Depth  : {}", depth);
    println!("   CPU Threads   : {}", num_threads);
    println!("   TT RAM        : {} MB/thread × {} = {:.1} GB", tt_mb, num_threads, tt_total_gb);
    println!("   Sieve Bitset  : {} MB = {:.1} GB", sieve_mb, sieve_gb);
    println!("   Tổng RAM      : {:.1} GB", total_ram_gb);
    println!("   Output Path   : {}", output_path);
    println!("------------------------------------------------------------");

    let sieve = Arc::new(Sieve::new(sieve_mb));
    let ram_buffer = Arc::new(Buffer::new());

    let games_completed = Arc::new(AtomicUsize::new(0));
    let samples_mined = Arc::new(AtomicUsize::new(0));
    let dupes_filtered = Arc::new(AtomicUsize::new(0));
    let stop_signal = Arc::new(AtomicBool::new(false));

    let start_time = Instant::now();

    // Monitor Thread
    let monitor_completed = games_completed.clone();
    let monitor_samples = samples_mined.clone();
    let monitor_dupes = dupes_filtered.clone();
    let monitor_stop = stop_signal.clone();
    let monitor_buffer = ram_buffer.clone();
    let monitor_path = output_path.clone();

    let monitor_handle = thread::spawn(move || {
        let mut last_samples = 0;
        let mut last_time = Instant::now();
        while !monitor_stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(3));
            let current_games = monitor_completed.load(Ordering::Relaxed);
            let current_samples = monitor_samples.load(Ordering::Relaxed);
            let current_dupes = monitor_dupes.load(Ordering::Relaxed);
            let now = Instant::now();
            let elapsed_sec = now.duration_since(start_time).as_secs_f64();
            let delta_sec = now.duration_since(last_time).as_secs_f64();

            let total_speed = current_samples as f64 / elapsed_sec.max(0.1);
            let instant_speed = (current_samples.saturating_sub(last_samples)) as f64 / delta_sec.max(0.1);
            let pct = (current_games as f64 / total_games.max(1) as f64) * 100.0;
            
            println!(
                "[MINING STREAMING {}/{}] ({:.1}%) | Samples: {} | Dupes: {} | Speed: {:.1} FEN/s (Instant: {:.1})",
                current_games, total_games, pct, current_samples, current_dupes, total_speed, instant_speed
            );

            let flushed = monitor_buffer.flush(&monitor_path);
            if flushed > 0 {
                println!("   💾 Flushed {} dòng xuống đĩa", flushed);
            }

            last_samples = current_samples;
            last_time = now;

            if current_games >= total_games {
                break;
            }
        }
    });

    let mut handles = Vec::with_capacity(num_threads);

    for thread_id in 0..num_threads {
        let games_counter = games_completed.clone();
        let samples_counter = samples_mined.clone();
        let dupes_counter = dupes_filtered.clone();
        let stop_flag = stop_signal.clone();
        let thread_sieve = sieve.clone();
        let thread_buffer = ram_buffer.clone();
        let mut thread_seed = base_seed.wrapping_add((thread_id as u64 + 1) * 12345678910111213);

        handles.push(thread::spawn(move || {
            let mut search = Search::new_boxed(tt_mb);
            let loaded = search.auto_load();
            if thread_id == 0 {
                if loaded {
                    println!("✅ Thread 0: NNUE weights loaded!");
                } else {
                    println!("⚠️ Thread 0: NNUE weights NOT found!");
                }
            }

            let evaluator = Eval::new();
            let mut limits = Limits::new();
            limits.depth = depth;

            let mut local_buffer: Vec<String> = Vec::with_capacity(2000);
            let mut local_count: usize = 0;
            let mut local_dupes: usize = 0;

            while !stop_flag.load(Ordering::Relaxed) {
                let current_game_idx = games_counter.fetch_add(1, Ordering::SeqCst);
                if current_game_idx >= total_games {
                    games_counter.fetch_sub(1, Ordering::SeqCst);
                    break;
                }

                let mut pos = Parser::parse(Parser::DEFAULT);

                thread_seed ^= thread_seed << 13;
                thread_seed ^= thread_seed >> 7;
                thread_seed ^= thread_seed << 17;
                let use_book = (thread_seed % 2) == 0;

                if use_book {
                    let mut book_steps = 0u8;
                    while book_steps < 12 {
                        if let Some(mv) = xiangrust::book::Book::probe(&pos) {
                            pos.apply(mv.from, mv.to);
                            book_steps += 1;
                        } else {
                            break;
                        }
                    }
                    let extra = 2 + (thread_seed as usize % 3);
                    for _ in 0..extra {
                        let mut moves = movegen::List::new();
                        movegen::legal(&mut pos, &mut moves);
                        if moves.len() == 0 { break; }
                        thread_seed ^= thread_seed << 13;
                        thread_seed ^= thread_seed >> 7;
                        thread_seed ^= thread_seed << 17;
                        let m = moves.items[(thread_seed as usize) % moves.len()];
                        pos.apply(m.from, m.to);
                    }
                } else {
                    for _ in 0..6 {
                        let mut moves = movegen::List::new();
                        movegen::legal(&mut pos, &mut moves);
                        if moves.len() == 0 { break; }
                        thread_seed ^= thread_seed << 13;
                        thread_seed ^= thread_seed >> 7;
                        thread_seed ^= thread_seed << 17;
                        let m = moves.items[(thread_seed as usize) % moves.len()];
                        pos.apply(m.from, m.to);
                    }
                }

                let mut ply: u32 = 0;
                let max_plies: u32 = 200;
                let mut pgn = String::new();
                let mut move_outcome = "draw";

                while ply < max_plies && !stop_flag.load(Ordering::Relaxed) {
                    let zobrist_key = pos.hash;
                    let is_unique = thread_sieve.insert(zobrist_key);
                    
                    let result = search.go(&pos, &limits);
                    if !result.best.valid() { break; }

                    let encoded = Format::encode(result.best);
                    let score = result.score;
                    let search_depth = result.depth;
                    let nodes = result.nodes;

                    if is_unique && ply >= 2 && score.abs() < 29000 {
                        let fen = Serializer::export(&pos);
                        let turn = if pos.side == 0 { "Đỏ" } else { "Đen" };
                        let phase = if ply < 20 { "opening" } else if ply < 50 { "midgame" } else { "endgame" };
                        let index = ply as usize;

                        let (red_inv, black_inv) = inventory(&pos);
                        let red_material = material(&pos, 0);
                        let black_material = material(&pos, 1);
                        let red_king_score = safety(&pos, 0);
                        let black_king_score = safety(&pos, 1);
                        let center = control(&pos);
                        let open_f = files(&pos);
                        let tact_pats = patterns(&pos);
                        let strat = strategy(phase, index);
                        let (red_dev, red_tot) = development(&pos, 0);
                        let (black_dev, black_tot) = development(&pos, 1);
                        let best_trans = translate(&pos, result.best);
                        let annotated_board = annotate(&fen);

                        let red_count = pos.grid.iter().filter(|&&p| p >= 1 && p <= 7).count();
                        let black_count = pos.grid.iter().filter(|&&p| p >= 8 && p <= 14).count();

                        let (advantages, disadvantages, positives_list, negatives_list) = risk(
                            &pos, pos.side, score, red_count, black_count
                        );

                        let mut legal_moves = movegen::List::new();
                        let mut alt_pos = pos.clone();
                        movegen::legal(&mut alt_pos, &mut legal_moves);
                        
                        let mut candidates_json: Vec<String> = Vec::new();
                        let mut candidates_for_compare: Vec<(String, i32, String, String)> = Vec::new();
                        
                        let best_intent = intent(&pos, result.best);
                        candidates_json.push(format!(
                            "{{\"move\": {:?}, \"notation\": {:?}, \"centipawn\": {}, \"intent\": {:?}, \"pros\": {}, \"cons\": {}, \"patterns\": {}}}",
                            encoded, best_trans, score, best_intent,
                            array(&advantages), array(&disadvantages), array(&tact_pats)
                        ));
                        candidates_for_compare.push((encoded.clone(), score, best_intent.clone(), best_trans.clone()));

                        let mut alt_count = 0;
                        for i in 0..legal_moves.len() {
                            let alt = legal_moves.items[i];
                            let alt_uci = Format::encode(alt);
                            if alt_uci == encoded { continue; }
                            if alt_count >= 2 { break; }
                            
                            let state = alt_pos.apply(alt.from, alt.to);
                            let alt_score = -evaluator.score(&alt_pos);
                            alt_pos.revert(alt.from, alt.to, &state);
                            
                            let alt_trans = translate(&pos, alt);
                            let alt_intent = intent(&pos, alt);
                            
                            candidates_json.push(format!(
                                "{{\"move\": {:?}, \"notation\": {:?}, \"centipawn\": {}, \"intent\": {:?}, \"pros\": [], \"cons\": [], \"patterns\": []}}",
                                alt_uci, alt_trans, alt_score, alt_intent
                            ));
                            candidates_for_compare.push((alt_uci, alt_score, alt_intent, alt_trans));
                            alt_count += 1;
                        }

                        let comp_str = compare(&candidates_for_compare, &encoded, score);

                        let thought = format!(
                            "<thought>\n\
                             [1/14] KIỂM KÊ QUÂN CỜ:\n  Đỏ: {}\n  Đen: {}\n\
                             [2/14] TƯƠNG QUAN VẬT CHẤT:\n  Đỏ: {}cp | Đen: {}cp | Chênh lệch: {}cp\n\
                             [3/14] AN TOÀN TƯỚNG:\n  Đỏ: {}/100 | Đen: {}/100\n\
                             [4/14] KHỐNG CHẾ TRUNG LỘ:\n  {}\n\
                             [5/14] MẪU CHIẾN THUẬT:\n  {}\n\
                             [6/14] GIAI ĐOẠN & CHIẾN LƯỢC:\n  Giai đoạn: {} (nước thứ {})\n  Chiến lược: {}\n\
                             [7/14] PHÂN TÍCH ƯU THẾ:\n  {}\n\
                             [8/14] PHÂN TÍCH BẤT LỢI:\n  {}\n\
                             [9/14] PHÂN TÍCH TÍCH CỰC:\n  {}\n\
                             [10/14] PHÂN TÍCH TIÊU CỰC:\n  {}\n\
                             [11/14] ĐÁNH GIÁ CANDIDATES ({} ứng viên):\n  Best: {} ({}cp) — {}\n\
                             [12/14] SO SÁNH & CHỌN BESTMOVE:\n  {}\n\
                             [13/14] CENTIPAWN TỔNG HỢP: {}cp\n\
                             [14/14] XÁC MINH: {} khớp regex ^[a-i][0-9][a-i][0-9]$ ✓\n\
                             </thought>",
                            red_inv, black_inv,
                            red_material, black_material, red_material - black_material,
                            red_king_score, black_king_score,
                            center,
                            if tact_pats.is_empty() { "Không phát hiện".to_string() } else { tact_pats.join(", ") },
                            phase, index, strat,
                            if advantages.is_empty() { "Thế cân bằng".to_string() } else { advantages.join("; ") },
                            if disadvantages.is_empty() { "Không có bất lợi rõ rệt".to_string() } else { disadvantages.join("; ") },
                            if positives_list.is_empty() { "Thế trận ổn định".to_string() } else { positives_list.join("; ") },
                            if negatives_list.is_empty() { "Không có rủi ro đáng kể".to_string() } else { negatives_list.join("; ") },
                            candidates_json.len(), encoded, score, best_trans,
                            comp_str,
                            score,
                            encoded
                        );

                        let assistant = format!(
                            "{{\"thought\": {:?}, \"board_analysis\": {{\"red_inventory\": {:?}, \"black_inventory\": {:?}, \"red_count\": {}, \"black_count\": {}, \"red_material\": {}, \"black_material\": {}, \"balance\": {}}}, \"position_assessment\": {{\"red_king_safety\": {}, \"black_king_safety\": {}, \"center_control\": {:?}, \"open_files\": {}, \"phase\": {:?}, \"phase_strategy\": {:?}}}, \"tactical_patterns\": {}, \"risk_assessment\": {{\"advantages\": {}, \"disadvantages\": {}, \"positives\": {}, \"negatives\": {}}}, \"candidates\": [{}], \"comparison\": {:?}, \"bestmove\": {:?}, \"explanation\": {:?}, \"centipawn_eval\": {}}}",
                            thought,
                            red_inv, black_inv,
                            red_count,
                            black_count,
                            red_material, black_material, red_material - black_material,
                            red_king_score, black_king_score,
                            center, array(&open_f), phase, strat,
                            array(&tact_pats),
                            array(&advantages), array(&disadvantages),
                            array(&positives_list), array(&negatives_list),
                            candidates_json.join(", "),
                            comp_str,
                            encoded,
                            best_trans,
                            score
                        );

                        let user = format!(
                            "Trạng thái bàn cờ tướng hiện tại:\n\n1. Bàn Cờ 2D:\n{}\n\n2. FEN:\n{}\n\n3. PGN:\n{}\n\nLượt {} đi.",
                            annotated_board, fen, pgn, turn
                        );

                        let stamp_sec = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

                        let sample = format!(
                            "{{\"messages\": [{{\"role\": \"system\", \"content\": {:?}}}, {{\"role\": \"user\", \"content\": {:?}}}, {{\"role\": \"assistant\", \"content\": {:?}}}], \"move\": {:?}, \"eval\": {}, \"outcome\": {:?}, \"phase\": {:?}, \"depth\": {}, \"nodes\": {}, \"stamp\": {}}}",
                            SYSTEM, user, assistant, encoded, score, move_outcome, phase, search_depth, nodes, stamp_sec
                        );

                        local_buffer.push(sample);
                        local_count += 1;

                        if local_buffer.len() >= 500 {
                            thread_buffer.push(std::mem::take(&mut local_buffer));
                            local_buffer = Vec::with_capacity(2000);
                            samples_counter.fetch_add(local_count, Ordering::Relaxed);
                            dupes_counter.fetch_add(local_dupes, Ordering::Relaxed);
                            local_count = 0;
                            local_dupes = 0;
                        }
                    } else if !is_unique {
                        local_dupes += 1;
                    }

                    if score.abs() > 29000 {
                        move_outcome = if score > 0 { "win" } else { "loss" };
                        break;
                    }

                    if pgn.len() > 0 { pgn.push(' '); }
                    pgn.push_str(&encoded);
                    pos.apply(result.best.from, result.best.to);
                    ply += 1;
                }

                if !local_buffer.is_empty() {
                    thread_buffer.push(std::mem::take(&mut local_buffer));
                    local_buffer = Vec::with_capacity(2000);
                    samples_counter.fetch_add(local_count, Ordering::Relaxed);
                    dupes_counter.fetch_add(local_dupes, Ordering::Relaxed);
                    local_count = 0;
                    local_dupes = 0;
                }
            }

            if !local_buffer.is_empty() {
                thread_buffer.push(local_buffer);
                samples_counter.fetch_add(local_count, Ordering::Relaxed);
                dupes_counter.fetch_add(local_dupes, Ordering::Relaxed);
            }
        }));
    }

    for h in handles {
        let _ = h.join();
    }

    stop_signal.store(true, Ordering::SeqCst);
    let _ = monitor_handle.join();

    let final_flushed = ram_buffer.flush(&output_path);
    
    println!("============================================================");
    println!(" 🏆 HOÀN THÀNH PHIÊN MINING JRCP 3.0!");
    println!("   Tệp dữ liệu đầu ra: {}", output_path);
    println!("============================================================");
}
