use std::io::Write;
use xiangrust::board::Parser;
use xiangrust::book::opening::Book;

#[test]
fn debug_step_by_step() {
    println!("[DEBUG] 1. Starting test");
    std::io::stdout().flush().unwrap();

    let book = Book::default();
    println!("[DEBUG] 2. book.count = {}", book.count);
    std::io::stdout().flush().unwrap();

    println!("[DEBUG] 3. Calling Parser::parse(Parser::DEFAULT)");
    std::io::stdout().flush().unwrap();
    let pos = Parser::parse(Parser::DEFAULT);

    println!("[DEBUG] 4. pos.hash = {:#X}", pos.hash);
    std::io::stdout().flush().unwrap();

    println!("[DEBUG] 5. Calling Book::probe(&pos)");
    std::io::stdout().flush().unwrap();
    let probed = Book::probe(&pos);

    println!("[DEBUG] 6. probed = {:?}", probed);
    std::io::stdout().flush().unwrap();
}
