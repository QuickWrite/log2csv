use std::iter::{Peekable, Iterator};
use std::str::Chars;

#[derive(Debug, PartialEq, Eq)]
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
    Dot,
    OpenParen,
    CloseParen,
    Number,
    String,
    Identifier,
}

#[derive(Debug)]
struct Token<'a> {
    ttype: TokenType,
    sequence: &'a str,
    pos: usize,
}

impl<'a> Token<'a> {
    fn from(lexer: &Lexer<'a>, ttype: TokenType, from: usize, to: usize) -> Token<'a> {
        Token {
            ttype,
            sequence: &lexer.expression[(from - 1)..to],
            pos: from - 1
        }
    }

    fn from_char(lexer: &Lexer<'a>, ttype: TokenType, pos: usize) -> Token<'a> {
        Token::from(lexer, ttype, pos, pos)
    }
}

struct Lexer<'a> {
    expression: &'a str,
    chars: Peekable<Chars<'a>>,
    cursor: usize
}

impl Lexer<'_> {
    fn new(input: &str) -> Lexer<'_> {
        Lexer {
            expression: input,
            chars: input.chars().peekable(),
            cursor: 0,
        }
    }

    fn next_char(&mut self) -> char {
        let char = self.chars.next();

        if char.is_none() {
            return '\0';
        }

        self.cursor += 1;

        char.unwrap()
    }

    fn peek(&mut self) -> char {
        *self.chars.peek().unwrap_or(&'\0')
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        match self.next_char() {
            '(' => Some(Token::from_char(self, TokenType::OpenParen, self.cursor)),
            ')' => Some(Token::from_char(self, TokenType::CloseParen, self.cursor)),
            '^' => Some(Token::from_char(self, TokenType::Xor, self.cursor)),

            '<' => {
                Some(if self.peek() == '=' {
                    self.next_char();
                    Token::from(self, TokenType::LessThanEquals, self.cursor - 1, self.cursor)
                } else {
                    Token::from_char(self, TokenType::LessThan, self.cursor)
                })
            },
            '>' => {
                Some(if self.peek() == '=' {
                    self.next_char();
                    Token::from(self, TokenType::GreaterThanEquals, self.cursor - 1, self.cursor)
                } else {
                    Token::from_char(self, TokenType::GreaterThan, self.cursor)
                })
            },

            '!' => {
                Some(if self.peek() == '=' {
                    self.next_char();
                    Token::from(self, TokenType::NotEquals, self.cursor - 1, self.cursor)
                } else {
                    Token::from_char(self, TokenType::Bang, self.cursor)
                })
            }

            '&' => {
                if self.next_char() == '&' {
                    Some(Token::from(self, TokenType::And, self.cursor - 1, self.cursor))
                } else {
                    None // TODO: proper error handling
                }
            },
            '|' => {
                if self.next_char() == '|' {
                    Some(Token::from(self, TokenType::Or, self.cursor - 1, self.cursor))
                } else {
                    None // TODO: proper error handling
                }
            },
            '=' => {
                if self.next_char() == '=' {
                    Some(Token::from(self, TokenType::Equals, self.cursor - 1, self.cursor))
                } else {
                    None // TODO: proper error handling
                }
            },

            '"' => {
                let start = self.cursor; // points just after the opening quote

                loop {
                    let ch = self.next_char();
                    match ch {
                        '\\' => {
                            let _escaped = self.next_char();
                        }
                        '"' => {
                            break;
                        }
                        '\0' => {
                            break; // TODO: Raise an error.
                        }
                        _ => {
                            // ordinary character – continue scanning
                        }
                    }
                }

                Some(Token::from(self, TokenType::String, start + 1, self.cursor - 1))
            }

            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = self.cursor - 1;

                while {
                    let nxt = self.peek();
                    nxt.is_ascii_alphanumeric() || nxt == '_'
                } {
                    self.next_char();
                }

                Some(Token::from(self, TokenType::Identifier, start + 1, self.cursor))
            },

            c if c.is_ascii_digit() || (c == '.' && self.peek().is_ascii_digit()) => {
                let start = self.cursor - 1;
                let mut seen_dot = c == '.';

                while {
                    let nxt = self.peek();
                    if nxt.is_ascii_digit() {
                        true
                    } else if nxt == '.' && !seen_dot {
                        // First dot – part of a floating point literal.
                        seen_dot = true;
                        true
                    } else {
                        false
                    }
                } {
                    self.next_char();
                }

                Some(Token::from(self, TokenType::Number, start + 1, self.cursor))
            },

            // The dot has to be checked after the number to allow numbers like `.025`.
            '.' => Some(Token::from_char(self, TokenType::Dot, self.cursor)),

            '\0' => None,

            _ => None, // TODO: Currently nothing is done. Probably an error (and also consider whitespace).
        }
    }
}

/// Helper used in the tests / examples.
pub fn parse_expression(input: &str) {
    let iterator = Lexer::new(input);

    for i in iterator {
        print!("[{}] {:?} ({}), \n", i.pos, i.ttype, i.sequence);
    }
}
