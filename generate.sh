#!/bin/bash
TARGET="/Users/hdqb/workspaces/xiangqi-rim/examples/23_jrcp3_ram64g_miner.rs"

cat << 'HEADER' > "$TARGET"
// EXAMPLE 23: BỘ MINING DỮ LIỆU JRCP 3.0 × 64GB RAM
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use xiangrust::board::{Parser, Serializer};
use xiangrust::eval::Eval;
use xiangrust::movegen;
use xiangrust::search::{Limits, Search};
use xiangrust::uci::Format;

HEADER

# const SYSTEM (lines 16-224 of 22)
sed -n '16,224p' /Users/hdqb/workspaces/xiangqi-rim/examples/22_jrcp3_miner.rs >> "$TARGET"
echo "" >> "$TARGET"

# const VALUE, NAME (226-227 of 22)
sed -n '226,227p' /Users/hdqb/workspaces/xiangqi-rim/examples/22_jrcp3_miner.rs >> "$TARGET"
echo "" >> "$TARGET"

# struct Sieve (lines 52-97 of 21)
sed -n '50,97p' /Users/hdqb/workspaces/xiangqi-rim/examples/21_ram64g_mine.rs >> "$TARGET"
echo "" >> "$TARGET"

# struct Buffer (lines 103-167 of 21)
sed -n '103,167p' /Users/hdqb/workspaces/xiangqi-rim/examples/21_ram64g_mine.rs >> "$TARGET"
echo "" >> "$TARGET"

# 8 HÀM PHÂN TÍCH (lines 229-660 of 22)
sed -n '229,660p' /Users/hdqb/workspaces/xiangqi-rim/examples/22_jrcp3_miner.rs >> "$TARGET"
echo "" >> "$TARGET"

# compare signature fix (22_jrcp3_miner.rs signature is `fn compare(candidates: &[(String, i32, String, String)], best: &str, best_score: i32) -> String`)
# But in our usage we will pass JSON string list or something? The prompt says "compare() function trong 22 nhận &[(String, i32, String)] — cần điều chỉnh signature nếu cần"
# Wait, let's fix it later. I will just run bash and then use sed or edit.

