use xiangrust::board::Parser;
use xiangrust::selfplay::Fen;

#[test]
fn simple_fen() {
    let pos = Parser::parse(Parser::DEFAULT);
    let text = Fen::export(&pos);
    assert_eq!(text, Parser::DEFAULT);
}
