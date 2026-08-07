// ============================================================================
// MODULE PARSER: BỘ PHÂN TÍCH CÚ PHÁP VĂN BẢN CHUẨN GIAO THỨC UCI (UCI PARSER)
// ============================================================================
// `parser.rs` chịu trách nhiệm bóc tách và phân tích các dòng lệnh văn bản STDIN từ GUI:
// - `parse(line)`: Phân tích 1 dòng lệnh văn bản đầu vào thành enum `Command`.
// - `option(words)`: Phân tích cú pháp lệnh `setoption name <name> value <value>`.
// - `position(words)`: Phân tích cú pháp lệnh `position [startpos|fen <fen>] [moves ...]`.
// - `go(words)`: Phân tích các cờ tham số thời gian và độ sâu của lệnh `go`.
// ============================================================================

use super::command::Command;

/// Struct `Parser` chứa các phương thức tĩnh phân tích cú pháp chuỗi văn bản UCI.
pub struct Parser;

impl Parser {
    /// Phân tích một dòng văn bản đầu vào `line` thành đối tượng câu lệnh `Command`.
    pub fn parse(line: &str) -> Command {
        let text = line.trim();
        if text.is_empty() {
            return Command::Invalid;
        }

        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return Command::Invalid;
        }

        match words[0] {
            "uci" => Command::Uci,
            "isready" => Command::Ready,
            "ucinewgame" => Command::Reset,
            "stop" => Command::Stop,
            "quit" => Command::Quit,
            "setoption" => Self::option(&words[1..]),
            "position" => Self::position(&words[1..]),
            "go" => Self::go(&words[1..]),
            _ => Command::Invalid,
        }
    }

    /// Phân tích các từ tiếp theo của câu lệnh `setoption`.
    fn option(words: &[&str]) -> Command {
        let mut name = String::new();
        let mut value = String::new();
        let mut mode = 0u8;

        for &word in words {
            if word == "name" {
                mode = 1;
                continue;
            } else if word == "value" {
                mode = 2;
                continue;
            }

            if mode == 1 {
                if !name.is_empty() {
                    name.push(' ');
                }
                name.push_str(word);
            } else if mode == 2 {
                if !value.is_empty() {
                    value.push(' ');
                }
                value.push_str(word);
            }
        }

        Command::Option { name, value }
    }

    /// Phân tích các từ tiếp theo của câu lệnh `position`.
    fn position(words: &[&str]) -> Command {
        if words.is_empty() {
            return Command::Invalid;
        }

        let mut fen = String::new();
        let mut moves = Vec::new();
        let mut mode = 0u8;

        let mut index = 0usize;
        if words[0] == "startpos" {
            fen = crate::board::fen::Parser::DEFAULT.to_string();
            index = 1;
        } else if words[0] == "fen" {
            mode = 1;
            index = 1;
        }

        while index < words.len() {
            let word = words[index];
            if word == "moves" {
                mode = 2;
                index += 1;
                continue;
            }

            if mode == 1 {
                if !fen.is_empty() {
                    fen.push(' ');
                }
                fen.push_str(word);
            } else if mode == 2 {
                moves.push(word.to_string());
            }
            index += 1;
        }

        Command::Position { fen, moves }
    }

    /// Phân tích các từ tiếp theo của câu lệnh `go`.
    fn go(words: &[&str]) -> Command {
        let mut depth = 0u8;
        let mut nodes = 0u64;
        let mut infinite = false;
        let mut span = 0u64;
        let mut red = 0u64;
        let mut black = 0u64;
        let mut gain = 0u64;
        let mut extra = 0u64;

        let mut index = 0usize;
        while index < words.len() {
            match words[index] {
                "depth" => {
                    if index + 1 < words.len() {
                        depth = words[index + 1].parse().unwrap_or(0);
                        index += 1;
                    }
                }
                "movetime" => {
                    if index + 1 < words.len() {
                        span = words[index + 1].parse().unwrap_or(0);
                        index += 1;
                    }
                }
                "nodes" => {
                    if index + 1 < words.len() {
                        nodes = words[index + 1].parse().unwrap_or(0);
                        index += 1;
                    }
                }
                "infinite" => {
                    infinite = true;
                }
                "wtime" => {
                    if index + 1 < words.len() {
                        red = words[index + 1].parse().unwrap_or(0);
                        index += 1;
                    }
                }
                "btime" => {
                    if index + 1 < words.len() {
                        black = words[index + 1].parse().unwrap_or(0);
                        index += 1;
                    }
                }
                "winc" => {
                    if index + 1 < words.len() {
                        gain = words[index + 1].parse().unwrap_or(0);
                        index += 1;
                    }
                }
                "binc" => {
                    if index + 1 < words.len() {
                        extra = words[index + 1].parse().unwrap_or(0);
                        index += 1;
                    }
                }
                _ => {}
            }
            index += 1;
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unit test phân tích đúng các cú pháp lệnh uci, isready, setoption, position, go
    #[test]
    fn parsing() {
        assert_eq!(Parser::parse("uci"), Command::Uci);
        assert_eq!(Parser::parse("isready"), Command::Ready);
        assert_eq!(Parser::parse("ucinewgame"), Command::Reset);
        assert_eq!(Parser::parse("stop"), Command::Stop);
        assert_eq!(Parser::parse("quit"), Command::Quit);

        if let Command::Option { name, value } = Parser::parse("setoption name Threads value 4") {
            assert_eq!(name, "Threads");
            assert_eq!(value, "4");
        } else {
            panic!("MUST parse setoption!");
        }

        if let Command::Position { fen, moves } = Parser::parse("position startpos moves h2e2") {
            assert!(!fen.is_empty());
            assert_eq!(moves.len(), 1);
            assert_eq!(moves[0], "h2e2");
        } else {
            panic!("MUST parse position!");
        }

        if let Command::Go { depth, span, .. } = Parser::parse("go depth 8 movetime 1000") {
            assert_eq!(depth, 8);
            assert_eq!(span, 1000);
        } else {
            panic!("MUST parse go!");
        }
    }
}

