use std::iter::Iterator;

///
/// The tokenizer that is being used to parse this.

#[derive(Debug)]
enum TokenType {
    Equals,
    NotEquals,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
    Bang,              // The `!` character
    And,
    Or,
    Xor,
    Identifier,
    Dot,
    OpenParen,
    CloseParen,
}

struct Token<'a> {
    ttype: TokenType,
    pos: usize,
    sequence: &'a str,
}

struct Lexer<'a> {
    expression: &'a str,
    cursor: usize
}

impl Lexer<'_> {
    fn new(input: &str) -> Lexer<'_> {
        Lexer {
            expression: input,
            cursor: 0,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        // TODO: Implement Lexer

        Option::None
    }
}

pub fn parse_expression(input: &str) {
    let iterator = Lexer::new(input);

    for i in iterator {
        print!("{:?}", i.ttype);
    }
}
